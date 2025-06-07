struct VInput {
	@location(0) pos: vec2f,
	@location(1) uv: vec2f,
	@location(2) col: vec4f,
	// @location(2) col: u32,
}

struct VOutput {
	@builtin(position) pos: vec4f,
	@location(0) col: vec4f,
	@location(1) uv: vec2f,
}

@vertex
fn vs_main(in: VInput) -> VOutput {
	var out: VOutput;
	out.pos = vec4(in.pos, 0.0, 1.0);
	out.col = in.col;
	// out.col = unpack4x8unorm(in.col);
	out.uv = in.uv;
	return out;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var sam: sampler;

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4f {
	// return in.col;
	return textureSample(tex, sam, in.uv) * in.col;
}