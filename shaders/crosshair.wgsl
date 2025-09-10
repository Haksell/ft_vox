struct CrosshairUniform {
    center: vec2<f32>,
    is_deleting: u32,
};

@group(0) @binding(0)
var<uniform> crosshair: CrosshairUniform;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    return VertexOutput(vec4<f32>(positions[vi], 0.0, 1.0));
}

const ARM_LEN: f32 = 5.0;
const OUTLINE_PX: f32 = 1.0;

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let d = abs(floor(frag_pos.xy) - crosshair.center);
    let in_core = (d.x == 0.0 && d.y <= ARM_LEN) || (d.y == 0.0 && d.x <= ARM_LEN);
    let in_crosshair = (d.x <= OUTLINE_PX && d.y <= ARM_LEN + OUTLINE_PX) || (d.y <= OUTLINE_PX && d.x <= ARM_LEN + OUTLINE_PX);
    let core_color = select(vec3(1.0), vec3(1.0, 0.1, 0.1), bool(crosshair.is_deleting));
    let color = select(vec3(0.0), core_color, in_core);
    return vec4<f32>(color, f32(in_crosshair));
}
