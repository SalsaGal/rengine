@group(0)@binding(0)
var<uniform> projection: mat4x4<f32>;
@group(0)@binding(1)
var<uniform> camera: mat4x4<f32>;

@group(1)@binding(0)
var texture: texture_2d<f32>;
@group(1)@binding(1)
var texture_sampler: sampler;

struct Transform {
    @location(0) row0: vec4<f32>,
    @location(1) row1: vec4<f32>,
    @location(2) row2: vec4<f32>,
    @location(3) row3: vec4<f32>,
}

fn transform_to_mat(transform: Transform) -> mat4x4<f32> {
    return mat4x4<f32>(transform.row0, transform.row1, transform.row2, transform.row3);
}

struct ColorInput {
    @location(4) pos: vec3<f32>,
    @location(5) color: vec4<f32>,
}

struct ColorOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn color_vertex(input: ColorInput, transform: Transform) -> ColorOutput {
    let transform_mat = transform_to_mat(transform);
    var output: ColorOutput;
    output.pos = projection * camera * transform_mat * vec4<f32>(input.pos, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn color_fragment(output: ColorOutput) -> @location(0) vec4<f32> {
    return output.color;
}

struct TextureInput {
    @location(4) pos: vec3<f32>,
    @location(5) tex_coords: vec2<f32>,
}

struct TextureOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn texture_vertex(input: TextureInput, transform: Transform) -> TextureOutput {
    let transform_mat = transform_to_mat(transform);
    var output: TextureOutput;
    output.pos = projection * camera * transform_mat * vec4<f32>(input.pos, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@fragment
fn texture_fragment(output: TextureOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, output.tex_coords);
    if (color.a == 0.0) {
        discard;
    } else {
        return color;
    }
}
