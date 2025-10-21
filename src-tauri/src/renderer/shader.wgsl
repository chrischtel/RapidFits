// Bind group 0: Texture, sampler, and uniforms (all in one group)
@group(0) @binding(0)
var fits_texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

// Uniforms for stretching
struct Uniforms {
    min_value: f32,
    max_value: f32,
    brightness: f32,
    contrast: f32,
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
    // Sample the FITS texture (single channel float)
    let raw_value = textureSample(fits_texture, texture_sampler, input.tex_coords).r;
    
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
