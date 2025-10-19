use anyhow::*;
use std::result::Result::{Err as StdErr, Ok as StdOk};
use std::{thread, time::Duration};
use tauri::WebviewWindow;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

pub fn init_renderer_for_window(window: &WebviewWindow) -> Result<()> {
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
    surface.configure(&device, &config);

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
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("clear-encoder"),
                        });

                    {
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
                                        a: 1.0, // Bright blue so it's visible
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

                    queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                StdErr(_) => {}
            }
            thread::sleep(Duration::from_millis(16)); // ~60 fps
        }
    });

    println!("✅ WGPU transparent renderer started");
    Ok(())
}
