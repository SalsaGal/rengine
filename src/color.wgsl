struct ColorInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct ColorOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@group(0)@binding(0)
var<uniform> projection: mat4x4<f32>;

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