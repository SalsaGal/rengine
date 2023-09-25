@group(0)@binding(0)
var<uniform> projection: mat4x4<f32>;

@group(1)@binding(0)
var texture: texture_2d<f32>;
@group(1)@binding(1)
var texture_sampler: sampler;

struct ColorInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct ColorOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn color_vertex(input: ColorInput) -> ColorOutput {
    var output: ColorOutput;
    output.pos = projection * vec4<f32>(input.pos, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn color_fragment(output: ColorOutput) -> @location(0) vec4<f32> {
    return output.color;
}

struct TextureInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct TextureOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn texture_vertex(input: TextureInput) -> TextureOutput {
    var output: TextureOutput;
    output.pos = projection * vec4<f32>(input.pos, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@fragment
fn texture_fragment(output: TextureOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, output.tex_coords);
}
