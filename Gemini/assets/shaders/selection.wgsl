#import bevy_pbr::mesh_view_bindings::globals

struct SelectionMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: SelectionMaterial;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    // Pulse effect
    let pulse = 0.5 + 0.5 * sin(globals.time * 5.0);
    
    // Glowing edges
    let edge_threshold = 0.03;
    let is_edge = uv.x < edge_threshold || uv.x > (1.0 - edge_threshold) || 
                  uv.y < edge_threshold || uv.y > (1.0 - edge_threshold);
    
    // Corner highlights
    let corner_threshold = 0.12;
    let is_corner = (uv.x < corner_threshold || uv.x > (1.0 - corner_threshold)) && 
                    (uv.y < corner_threshold || uv.y > (1.0 - corner_threshold));

    var final_color = material.color.rgb;
    var alpha = 0.08 + 0.04 * pulse;

    if (is_edge) {
        final_color *= 1.8;
        alpha = 0.7 + 0.3 * pulse;
    }
    
    if (is_corner) {
        final_color *= 2.5;
        alpha = 1.0;
    }

    // Boost emissive to trigger Bloom
    return vec4<f32>(final_color * 1.5, alpha);
}
