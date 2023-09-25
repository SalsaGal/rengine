struct ColorInput {
    @location(0) pos: vec3<f32>,
}

struct ColorOutput {
    @builtin(position) pos: vec4<f32>,
}

@vertex
fn color_vertex(input: ColorInput) -> ColorOutput {
    var output: ColorOutput;
    output.pos = vec4<f32>(input.pos, 1.0);
    return output;
}

@fragment
fn color_fragment(output: ColorOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}