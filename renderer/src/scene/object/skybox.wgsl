struct Uniforms {
	cam_view: mat4x4f,
	cam_view_inv: mat4x4f,
	cam_proj: mat4x4f,
	cam_proj_inv: mat4x4f,
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
};

@vertex
fn vs_main(in: VInput) -> VOutput {
	var out: VOutput;
	out.uv = in.uv;
	out.position = vec4f(in.position.xy, 0.999999, 1.0);
	
	return out;
}

struct Colors {
	points: array<vec4f, 8>,
	colors: array<vec4f, 8>,
};

@group(0) @binding(1) var<uniform> colors: Colors;

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4f {
	let sundir = normalize(vec3f(0.3, -1.0, -0.3));
	
	let temp = uniforms.cam_proj_inv * vec4<f32>(1.0 - in.uv.x * 2.0, 1.0 - in.uv.y * 2.0, 1.0, -1.0);
	let dir = (uniforms.cam_view * vec4f(normalize(temp.xyz), 0.0)).xyz;
	let point = -dir.y / 2.0 + 0.5;
	
	var from_point = 0.0;
	var from_color = vec4f(0.0, 0.0, 0.0, 1.0);
	var to_point = 1.0;
	var to_color = vec4f(0.0, 0.0, 0.0, 1.0);
	for(var i = 0; i < 8; i += 1) {
		if colors.points[i].x <= point {
			from_point = colors.points[i].x;
			from_color = colors.colors[i];
			
			if i != 7 {
				to_point = colors.points[i + 1].x;
				to_color = colors.colors[i + 1];
			}
		} else {
			break;
		}
	}
	
	// TODO: maybe dithering? the banding on the black is horrendous
	let point_adjusted = (point - from_point) / (to_point - from_point);
	let skycolor = from_color * (1.0 - point_adjusted) + to_color * point_adjusted;
	
	let sundist = max(dot(sundir, dir) - 0.99, 0.0) * 100.0;
	return vec4f(1.0, 0.7, 0.2, 1.0) * sundist + skycolor * (1.0 - sundist);
}