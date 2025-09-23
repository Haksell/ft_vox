struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_skybox_inverse: mat4x4<f32>,
    pos: vec3<f32>,
    _pad0: f32,
    near: f32,
    far: f32,
    _pad1: vec2<f32>,
};

@group(0) @binding(0) var sceneTexture: texture_2d<f32>;
@group(0) @binding(1) var sceneSampler: sampler;
@group(0) @binding(2) var depthTexture: texture_depth_2d;
@group(0) @binding(3) var depthSampler: sampler;
@group(0) @binding(4) var<uniform> cam: CameraUniform;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0)
    );

    var out: VertexOutput;
    out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

const OUTLINE_COLOR = vec4<f32>(0.0, 0.0, 0.0, 1.0);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let texSize = vec2<f32>(textureDimensions(sceneTexture, 0));
    let duv = 1.0 / texSize;

    let baseColor = textureSample(sceneTexture, sceneSampler, uv).rgb;

    let dc = textureSample(depthTexture, depthSampler, uv);
    if dc >= 0.999999 {
        return vec4<f32>(baseColor, 1.0);
    }

    let dl = textureSample(depthTexture, depthSampler, uv + vec2<f32>(-duv.x, 0.0));
    let dr = textureSample(depthTexture, depthSampler, uv + vec2<f32>(duv.x, 0.0));
    let dt = textureSample(depthTexture, depthSampler, uv + vec2<f32>(0.0, -duv.y));
    let db = textureSample(depthTexture, depthSampler, uv + vec2<f32>(0.0, duv.y));

    if dl >= 0.999999 || dr >= 0.999999 || dt >= 0.999999 || db >= 0.999999 {
        return OUTLINE_COLOR;
    }

    let depthThreshold = 0.00001 * dc;

    let dx = dr - dl;
    let dy = dt - db;
    var edgeDepth = sqrt(dx * dx + dy * dy);

    if edgeDepth > depthThreshold {
        return OUTLINE_COLOR;
    }

    return vec4<f32>(baseColor, 1.0);
}
