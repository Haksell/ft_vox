struct Screen {
    center: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen: Screen;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VSOut {
    // Fullscreen triangle (no vertex buffer)
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(positions[vi], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    // Pixel coords of current fragment
    let p = frag_pos.xy;
    let c = screen.center;

    let dx = abs(p.x - c.x);
    let dy = abs(p.y - c.y);

    // Crosshair sizing (in pixels)
    let core_half_thickness = 0.5;     // 1px core line (centered)
    let arm_len = 7.0;      // arm length from center
    let outline_px = 1.0;      // 1px black outline around the white core

    // Core (white) hit test
    let in_vert_core = (dx <= core_half_thickness) && (dy <= arm_len);
    let in_horz_core = (dy <= core_half_thickness) && (dx <= arm_len);
    let in_core = in_vert_core || in_horz_core;

    // Outline region (slightly larger box), but exclude core so outline surrounds it
    let in_vert_outline = (dx <= core_half_thickness + outline_px) && (dy <= arm_len + outline_px);
    let in_horz_outline = (dy <= core_half_thickness + outline_px) && (dx <= arm_len + outline_px);
    let in_outline_ring = (in_vert_outline || in_horz_outline) && !in_core;

    if in_core {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else if in_outline_ring {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
