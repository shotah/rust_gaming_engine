// Block rendering shader
// Renders voxel chunks with texture atlas and basic lighting

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// Texture atlas constants
const ATLAS_COLUMNS: f32 = 8.0;
const ATLAS_ROWS: f32 = 4.0;
const TILE_SIZE_U: f32 = 1.0 / ATLAS_COLUMNS;
const TILE_SIZE_V: f32 = 1.0 / ATLAS_ROWS;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) ao: f32,
    @location(4) local_uv: vec2<f32>,   // Local UV (0 to width, 0 to height) for tiling
    @location(5) atlas_uv: vec2<f32>,   // Atlas base position (top-left of tile)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) ao: f32,
    @location(4) local_uv: vec2<f32>,
    @location(5) atlas_uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_position = in.position;
    out.normal = in.normal;
    out.color = in.color;
    out.ao = in.ao;
    out.local_uv = in.local_uv;
    out.atlas_uv = in.atlas_uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Tile the texture within the atlas slot using fract()
    // local_uv goes from 0 to quad_width/height, fract gives us 0-1 for each tile
    let tiled_uv = fract(in.local_uv);
    
    // Map to atlas position: atlas_base + tiled_uv * tile_size
    let final_uv = in.atlas_uv + tiled_uv * vec2<f32>(TILE_SIZE_U, TILE_SIZE_V);
    
    // Sample texture at the computed atlas position
    let tex_color = textureSample(t_diffuse, s_diffuse, final_uv);
    
    // Alpha test - discard fully transparent pixels (cutout transparency)
    if tex_color.a < 0.1 {
        discard;
    }
    
    // Sun direction (from upper-right)
    let sun_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    
    // Basic directional lighting
    let ndotl = max(dot(in.normal, sun_dir), 0.0);
    
    // Ambient light (so shadows aren't completely black)
    let ambient = 0.4;
    
    // Face-based shading (different brightness per face for depth)
    var face_shade = 1.0;
    if abs(in.normal.y) > 0.5 {
        // Top/bottom faces
        if in.normal.y > 0.0 {
            face_shade = 1.0;  // Top is brightest
        } else {
            face_shade = 0.5;  // Bottom is darkest
        }
    } else if abs(in.normal.x) > 0.5 {
        // East/West faces
        face_shade = 0.8;
    } else {
        // North/South faces
        face_shade = 0.6;
    }
    
    // Combine lighting
    let light = ambient + ndotl * 0.6;
    
    // Apply AO and face shading
    let final_light = light * in.ao * face_shade;
    
    // Final color: texture * lighting (texture already has block color baked in)
    let final_color = tex_color.rgb * final_light;
    
    return vec4<f32>(final_color, tex_color.a);
}
