struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.position = vec4<f32>(in.pos, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

const COLOR: vec3<f32> = vec3<f32>(0.005,0.01,0.005);

fn is_white(v: vec3<f32>) -> bool {
    return v.x == 1.0 && v.y == 1.0 && v.z == 1.0;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords).xyz;
    if is_white(c) {
        return vec4<f32>(COLOR * 30.0, 0.0); 
    } else {
        return vec4<f32>(COLOR, 0.0);
    }
}