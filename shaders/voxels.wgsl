struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_skybox_inverse: mat4x4<f32>,
    pos: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) opp_position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) atlas_offset: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) atlas_offset: vec2<u32>,
    @location(2) square_size: vec2<f32>,
    @location(3) dist: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    let clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    let opp_clip_position = camera.view_proj * vec4<f32>(model.opp_position, 1.0);

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.atlas_offset = model.atlas_offset;
    out.clip_position = clip_position;
    out.square_size = abs(clip_position.xy / clip_position.z - opp_clip_position.xy / clip_position.z);
    out.dist = distance(model.position.xyz, camera.pos);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

const ATLAS_SHAPE: vec2<f32> = vec2(128.0, 64.0);

fn a16(x: f32) -> f32 {
    return x;
}

fn a8(x: f32) -> f32 {
    return x / 2.0 + 0.5;
}

fn a4(x: f32) -> f32 {
    return x / 4.0 + 0.75;
}

fn a2(x: f32) -> f32 {
    return x / 8.0 + 0.875;
}

fn a1(x: f32) -> f32 {
    return x / 16.0 + 0.9375;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4(in.square_size.x, 0.0, in.square_size.y, 1.0);
    var uv = (fract(in.tex_coords) + vec2<f32>(in.atlas_offset)) / ATLAS_SHAPE;

    var lod_x = max(3.0 - log2(in.square_size.x) / 2.0, 0.0);
    var lod_y = max(3.0 - log2(in.square_size.y) / 2.0, 0.0);

    var x1: f32;
    var x2: f32;
    switch u32(lod_x) {
        case 0u: { x1 = a16(uv.x); x2 = a16(uv.x); }
        case 1u: { x1 = a16(uv.x); x2 = a8(uv.x);  }
        case 2u: { x1 = a8(uv.x);  x2 = a4(uv.x);  }
        case 3u: { x1 = a4(uv.x);  x2 = a2(uv.x);  }
        case 4u: { x1 = a2(uv.x);  x2 = a1(uv.x);  }
        default: { x1 = a1(uv.x);  x2 = a1(uv.x);  }
    }

    var y1: f32;
    var y2: f32;
    switch u32(lod_y) {
        case 0u: { y1 = a16(uv.y); y2 = a16(uv.y); }
        case 1u: { y1 = a16(uv.y); y2 = a8(uv.y);  }
        case 2u: { y1 = a8(uv.y);  y2 = a4(uv.y);  }
        case 3u: { y1 = a4(uv.y);  y2 = a2(uv.y);  }
        case 4u: { y1 = a2(uv.y);  y2 = a1(uv.y);  }
        default: { y1 = a1(uv.y);  y2 = a1(uv.y);  }
    }

    return mix(
        textureSample(t_diffuse, s_diffuse, vec2(x1, y1)),
        textureSample(t_diffuse, s_diffuse, vec2(x2, y2)),
        vec4(fract(lod_x), fract(lod_y), 0.5, 0.5),
    );
}