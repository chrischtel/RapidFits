use anyhow::*;
use std::result::Result::{Err as StdErr, Ok as StdOk};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use tauri::WebviewWindow;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use wgpu::util::DeviceExt;

pub struct FitsRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    texture: Option<Arc<wgpu::Texture>>,
    width: u32,
    height: u32,

    pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
}

impl FitsRenderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            texture: None,
            width: 0,
            height: 0,
            pipeline: None,
            bind_group: None,
            uniform_buffer: None,
        }
    }
    pub fn create_pipeline(
        &mut self,
        surface_format: wgpu::TextureFormat,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Result<()> {
        // 1. Load WGSL shader
        let shader_source = include_str!("shader.wgsl");
        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("FITS Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        // 2. Create uniform buffer (min, max, brightness, contrast, zoom, pan_x, pan_y, aspect_ratio, viewport_aspect, padding)
        let image_aspect = self.width as f32 / self.height as f32;
        let viewport_aspect = viewport_width as f32 / viewport_height as f32;

        println!(
            "📐 Image aspect: {} ({}x{})",
            image_aspect, self.width, self.height
        );
        println!(
            "📐 Viewport aspect: {} ({}x{})",
            viewport_aspect, viewport_width, viewport_height
        );

        let uniform_data = [
            0.0f32,          // min_value
            65535.0f32,      // max_value
            0.0f32,          // brightness
            1.0f32,          // contrast
            1.0f32,          // zoom (1.0 = fit to screen)
            0.0f32,          // pan_x
            0.0f32,          // pan_y
            image_aspect,    // aspect_ratio of image
            viewport_aspect, // viewport_aspect (actual window dimensions)
            0.0f32,          // padding1
            0.0f32,          // padding2
            0.0f32,          // padding3
        ];
        let uniform_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // 3. Create texture sampler (Nearest for R32Float since it's not filterable)
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("FITS Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // You must have a texture already loaded
        let texture = self.texture.as_ref().context("Texture not yet loaded")?;
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 4. Bind group layout for texture + sampler + uniform buffer
        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("FITS Bind Group Layout"),
                    entries: &[
                        // Texture binding
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // Sampler binding
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        // Uniform buffer binding
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // 5. Create the bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("FITS Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // 6. Create the pipeline layout
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("FITS Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // 7. Create a fullscreen quad vertex buffer layout (no vertex buffer used)
        let vertex_state = wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[], // fullscreen triangle via shader
            compilation_options: Default::default(),
        };

        // 8. Create the render pipeline
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("FITS Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: vertex_state,
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        // 9. Store them
        self.pipeline = Some(pipeline);
        self.bind_group = Some(bind_group);
        self.uniform_buffer = Some(uniform_buffer);

        Ok(())
    }

    /// Update pan and zoom controls
    pub fn update_view(&self, zoom: f32, pan_x: f32, pan_y: f32) {
        if let Some(buffer) = &self.uniform_buffer {
            // Update only the zoom and pan values (indices 4, 5, 6 in the uniform array)
            let data = [zoom, pan_x, pan_y];
            self.queue
                .write_buffer(buffer, 16, bytemuck::cast_slice(&data)); // offset 16 bytes (4 floats * 4 bytes)
        }
    }

    /// Update viewport aspect ratio when window is resized
    pub fn update_viewport_aspect(&self, viewport_width: u32, viewport_height: u32) {
        if let Some(buffer) = &self.uniform_buffer {
            let viewport_aspect = viewport_width as f32 / viewport_height as f32;
            // viewport_aspect is at index 8 in the uniform array
            self.queue
                .write_buffer(buffer, 32, bytemuck::cast_slice(&[viewport_aspect]));
            // offset 32 bytes (8 floats * 4 bytes)
        }
    }

    pub fn load_fits_data(&mut self, data: Vec<f32>, w: usize, h: usize) -> Result<()> {
        let size = wgpu::Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Fits DATA Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let texture = self.device.create_texture(&desc);

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&data), // Convert Vec<f32> to &[u8] bytes
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w as u32 * 4), // 4 bytes per f32
                rows_per_image: Some(h as u32),
            },
            size, // The same size you used for texture creation
        );

        // Store the texture
        self.texture = Some(Arc::new(texture));
        self.width = w as u32;
        self.height = h as u32;

        Ok(())
    }
}

pub fn init_renderer_for_window(
    window: &WebviewWindow,
) -> Result<(Arc<Mutex<FitsRenderer>>, wgpu::TextureFormat)> {
    println!(
        "🚀 Initializing transparent WGPU renderer for '{}'",
        window.label()
    );

    let window_handle = window.window_handle()?;
    let display_handle = window.display_handle()?;

    let instance = wgpu::Instance::default();

    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: display_handle.as_raw(),
            raw_window_handle: window_handle.as_raw(),
        })
    }?;

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .context("No compatible GPU adapter found")?;

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::wgt::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))?;

    let caps = surface.get_capabilities(&adapter);
    let format = caps.formats[0];
    let size = window.inner_size()?;

    // Wrap device and queue in Arc for sharing
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    // Create the renderer
    let renderer = Arc::new(Mutex::new(FitsRenderer::new(
        Arc::clone(&device),
        Arc::clone(&queue),
    )));

    // Clone for the render thread
    let renderer_clone = Arc::clone(&renderer);
    let device_clone = Arc::clone(&device);
    let queue_clone = Arc::clone(&queue);

    // Try to find a supported alpha mode, preferring PreMultiplied for transparency
    let alpha_mode = if caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
    {
        wgpu::CompositeAlphaMode::PreMultiplied
    } else if caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
    {
        wgpu::CompositeAlphaMode::PostMultiplied
    } else if caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::Inherit)
    {
        wgpu::CompositeAlphaMode::Inherit
    } else {
        println!("⚠️  No transparent alpha mode supported, using Opaque");
        caps.alpha_modes[0]
    };

    println!("Using alpha mode: {:?}", alpha_mode);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 1,
        alpha_mode,
        view_formats: vec![],
    };
    surface.configure(&device_clone, &config);

    // Spawn continuous rendering thread
    thread::spawn(move || {
        loop {
            match surface.get_current_texture() {
                StdOk(frame) => {
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("surface-view"),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        format: None,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                        usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
                    });

                    let mut encoder =
                        device_clone.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("render-encoder"),
                        });

                    {
                        let renderer = renderer_clone.lock().unwrap();

                        // Check if we have a pipeline to render with
                        if let (Some(pipeline), Some(bind_group)) =
                            (&renderer.pipeline, &renderer.bind_group)
                        {
                            // Render the FITS image
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("fits-render-pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                            store: wgpu::StoreOp::Store,
                                        },
                                        depth_slice: None,
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                });

                            rpass.set_pipeline(pipeline);
                            rpass.set_bind_group(0, bind_group, &[]);
                            rpass.draw(0..3, 0..1); // Draw fullscreen triangle
                        } else {
                            // No pipeline yet, just clear to blue
                            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("clear-pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.2,
                                            g: 0.5,
                                            b: 0.8,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                        }
                    }

                    queue_clone.submit(Some(encoder.finish()));
                    frame.present();
                }
                StdErr(_) => {}
            }
            thread::sleep(Duration::from_millis(16)); // ~60 fps
        }
    });

    println!("✅ WGPU transparent renderer started");
    Ok((renderer, format))
}
