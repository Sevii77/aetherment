struct Uniforms {
	cam_view: mat4x4f,
	cam_proj: mat4x4f,
	object: mat4x4f,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;


struct VInput {
	@location(0) uv: vec2f,
	@location(1) position: vec3f,
	@location(2) normal: vec3f,
	@location(3) tangent: vec4f,
	@location(4) color: vec4f,
}

struct VOutput {
	@builtin(position) position: vec4f,
	@location(0) uv: vec2f,
	@location(1) normal: vec3f,
	@location(2) tangent: vec3f,
	@location(3) bitangent: vec3f,
	@location(4) color: vec4f,
};

@vertex
fn vs_main(in: VInput) -> VOutput {
	var rotation = mat3x3f(
		normalize(uniforms.object[0].xyz),
		normalize(uniforms.object[1].xyz),
		normalize(uniforms.object[2].xyz),
	);
	
	var out: VOutput;
	out.uv = in.uv;
	out.position = uniforms.cam_proj * uniforms.cam_view * uniforms.object * vec4f(in.position, 1.0);
	out.normal = normalize(rotation * in.normal);
	out.tangent = normalize(rotation * in.tangent.xyz);
	out.bitangent = cross(out.normal, out.tangent * in.tangent.w);
	out.color = in.color;
	
	return out;
}

@group(1) @binding(0) var albedo_texture: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;

@group(1) @binding(2) var normal_texture: texture_2d<f32>;
@group(1) @binding(3) var normal_sampler: sampler;

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4f {
	let sundir = normalize(vec3f(0.3, -1.0, -0.3));
	// let sundir = normalize(vec3f(0.0, -1.0, 0.0));
	
	var albedo = textureSample(albedo_texture, albedo_sampler, in.uv);
	var normal = textureSample(normal_texture, normal_sampler, in.uv).xyz * 2.0 - 1.0;
	
	let t = normalize(in.tangent);
	let b = normalize(in.bitangent);
	let n = normalize(in.normal);
	let tbn = mat3x3f(t, b, n);
	
	normal = normalize(tbn * normal);
	
	var shade = max(0.05, dot(-sundir, normal) / 2.0 + 0.5);
	
	return albedo * vec4f(in.color.xyz * shade, in.color.w);
}