#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

@group(1) @binding(0)
var<uniform> time: f32;
@group(1) @binding(1)
var<uniform> steepness: f32;
@group(1) @binding(2)
var<uniform> wavelength: f32;
@group(1) @binding(3)
var<uniform> speed: f32;
@group(1) @binding(4)
var<uniform> wave_1_dir: vec2<f32>;
@group(1) @binding(5)
var<uniform> wave_2_dir: vec2<f32>;
@group(1) @binding(6)
var<uniform> wave_3_dir: vec2<f32>;

struct GerstnerWaveOutput {
    pt_increment: vec3<f32>,
    new_tangent: vec3<f32>,
    new_binormal: vec3<f32>,
}; 

fn gerstner_wave(wave_dir: vec2<f32>, pt: vec3<f32>, tangent: vec3<f32>, binormal: vec3<f32>) -> GerstnerWaveOutput {
    var out: GerstnerWaveOutput;

    var pi: f32 = 3.14159;

    var k: f32 = 2.0 * pi / wavelength;
    var c: f32 = sqrt(9.8 / k);
    var d: vec2<f32> = normalize(wave_dir.xy);
    var f: f32 = k * (dot(d, pt.xz) - c * time);
    var a: f32 = steepness / k;

    out.new_tangent = tangent + vec3<f32>(
				-d.x * d.x * (steepness * sin(f)),
				d.x * (steepness * cos(f)),
				-d.x * d.y * (steepness * sin(f))
			);
    
    out.new_binormal = binormal + vec3<f32>(
        -d.x * d.y * (steepness * sin(f)),
        d.y * (steepness * cos(f)),
        -d.y * d.y * (steepness * sin(f))
    );

    out.pt_increment = vec3<f32>(
        d.x * (a * cos(f)),
        a * sin(f),
        d.y * (a * cos(f))
    );

    return out;
}

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var model = mesh.model;

    var pt = vertex.position;
    var tangent = vec3<f32>(0.0, 0.0, 0.0);
    var binormal = vec3<f32>(0.0, 0.0, 0.0);
    var out_pt = pt;

    var wave_1 = gerstner_wave(wave_1_dir, pt, tangent, binormal);
    out_pt += wave_1.pt_increment;
    tangent = wave_1.new_tangent;
    binormal = wave_1.new_binormal;

    var wave_2 = gerstner_wave(wave_2_dir, pt, tangent, binormal);
    out_pt += wave_2.pt_increment;
    tangent = wave_2.new_tangent;
    binormal = wave_2.new_binormal;

    var wave_3 = gerstner_wave(wave_3_dir, pt, tangent, binormal);
    out_pt += wave_3.pt_increment;
    tangent = wave_3.new_tangent;
    binormal = wave_3.new_binormal;

    out.world_position = mesh_position_local_to_world(model, vec4<f32>(out_pt, 1.0));

    var normal = normalize(cross(tangent, binormal));
    out.world_normal = mesh_normal_local_to_world(normal);

    var light = (dot(normal, vertex.normal) + 2.0) / 2.0;

#ifdef VERTEX_COLORS
    out.color = vertex.color * (vec4<f32>(light, light, light, 1.0));
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, tangent);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

    out.clip_position = mesh_position_world_to_clip(out.world_position);

    return out;
}