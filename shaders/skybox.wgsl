const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;

struct Camera {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
};

@group(1) @binding(0) var<uniform> camera: Camera;

@group(0) @binding(0) var sky_texture: texture_2d<f32>;
@group(0) @binding(1) var sky_sampler: sampler;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,  // clip-space xy in [-1, 1]
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // fullscreen triangle positions in clip space.
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );

    var out: VertexOutput;
    out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
    out.ndc = pos[vi];
    return out;
}

fn world_dir_from_ndc(ndc: vec2<f32>) -> vec3<f32> {
    let p0 = camera.inverse_view_proj * vec4<f32>(ndc, 0.0, 1.0);
    let p1 = camera.inverse_view_proj * vec4<f32>(ndc, 1.0, 1.0);
    let w0 = p0.xyz / p0.w;
    let w1 = p1.xyz / p1.w;
    return normalize(w1 - w0);
}

// convert direction to equirectangular UVs.
fn pano_uv(dir: vec3<f32>) -> vec2<f32> {
    let phi = atan2(dir.y, dir.x);
    let theta = acos(clamp(dir.z, -1.0, 1.0));
    let u = phi / TAU + 0.5;
    let v = 1.0 - theta / PI;
    return vec2<f32>(u, v);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = world_dir_from_ndc(in.ndc);
    let uv = pano_uv(dir);
    let c = textureSample(sky_texture, sky_sampler, uv);
    return vec4<f32>(c.rgb, 1.0);
}