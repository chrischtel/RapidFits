// Bind group 0: Texture, sampler, and uniforms (all in one group)
@group(0) @binding(0)
var fits_texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

// Uniforms for stretching and navigation
struct Uniforms {
    min_value: f32,
    max_value: f32,
    brightness: f32,
    contrast: f32,
    zoom: f32,        // Zoom level (1.0 = fit to screen, 2.0 = 2x zoom)
    pan_x: f32,       // Pan offset X (-1.0 to 1.0)
    pan_y: f32,       // Pan offset Y (-1.0 to 1.0)
    aspect_ratio: f32, // Image aspect ratio (width/height)
    viewport_aspect: f32, // Viewport aspect ratio
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

// Vertex shader output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Vertex shader - draws fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Generate fullscreen triangle positions
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    
    output.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    output.tex_coords = vec2<f32>(x, y);
    
    return output;
}

// Fragment shader - samples texture and applies color mapping
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Apply zoom and pan to texture coordinates
    var tex_coords = input.tex_coords - 0.5; // Center at origin
    
    // Correct aspect ratio to prevent stretching
    // If viewport is wider than image, scale X; if taller, scale Y
    if (uniforms.viewport_aspect > uniforms.aspect_ratio) {
        // Viewport is wider, letterbox sides
        tex_coords.x *= uniforms.viewport_aspect / uniforms.aspect_ratio;
    } else {
        // Viewport is taller, letterbox top/bottom
        tex_coords.y *= uniforms.aspect_ratio / uniforms.viewport_aspect;
    }
    
    // Apply zoom
    tex_coords = tex_coords / uniforms.zoom;
    
    // Apply pan
    tex_coords = tex_coords - vec2<f32>(uniforms.pan_x, uniforms.pan_y);
    
    // Back to texture space
    tex_coords = tex_coords + 0.5;
    
    // Check if we're outside the texture bounds
    if (tex_coords.x < 0.0 || tex_coords.x > 1.0 || tex_coords.y < 0.0 || tex_coords.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black outside bounds
    }
    
    // Sample the FITS texture (single channel float)
    let raw_value = textureSample(fits_texture, texture_sampler, tex_coords).r;
    
    // Normalize: map [min, max] to [0, 1]
    let normalized = (raw_value - uniforms.min_value) / (uniforms.max_value - uniforms.min_value);
    
    // Apply brightness and contrast
    let adjusted = (normalized - 0.5) * uniforms.contrast + 0.5 + uniforms.brightness;
    
    // Clamp to [0, 1]
    let final_value = clamp(adjusted, 0.0, 1.0);
    
    // Convert grayscale to RGB (currently just grayscale, can add false color later)
    let color = vec3<f32>(final_value, final_value, final_value);
    
    return vec4<f32>(color, 1.0);
}