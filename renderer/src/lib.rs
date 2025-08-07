use glam::{Vec2, Vec3, Vec4};

pub mod renderer;
pub use renderer::Renderer;
pub mod scene;
pub use scene::*;

pub(crate) type ObjectBuffer = Vec<Option<Box<dyn Object>>>;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub uv: Vec2,
	pub position: Vec3,
	pub normal: Vec3,
	pub tangent: Vec4,
	pub color: Vec4,
}

pub const fn vertex(position: Vec3, normal: Vec3, color: Vec4, uv: Vec2) -> Vertex {
	Vertex {
		uv,
		position,
		normal,
		tangent: Vec4::ZERO,
		color,
	}
}

// https://terathon.com/blog/tangent-space.html
pub fn calculate_tangents(vertices: &mut [Vertex], indices: &[u16]) {
	let mut tan1 = vec![Vec3::ZERO; vertices.len()];
	let mut tan2 = vec![Vec3::ZERO; vertices.len()];
	
	for i in (0..indices.len()).step_by(3) {
		let i1 = indices[i    ] as usize;
		let i2 = indices[i + 1] as usize;
		let i3 = indices[i + 2] as usize;
		
		let v1 = &vertices[i1];
		let v2 = &vertices[i2];
		let v3 = &vertices[i3];
		
		let x1 = v2.position.x - v1.position.x;
		let x2 = v3.position.x - v1.position.x;
		let y1 = v2.position.y - v1.position.y;
		let y2 = v3.position.y - v1.position.y;
		let z1 = v2.position.z - v1.position.z;
		let z2 = v3.position.z - v1.position.z;
		
		let s1 = v2.uv.x - v1.uv.x;
		let s2 = v3.uv.x - v1.uv.x;
		let t1 = v2.uv.y - v1.uv.y;
		let t2 = v3.uv.y - v1.uv.y;
		
		let r = 1.0 / (s1 * t2 - s2 * t1);
		let sdir = Vec3::new((t2 * x1 - t1 * x2) * r, (t2 * y1 - t1 * y2) * r, (t2 * z1 - t1 * z2) * r);
		let tdir = Vec3::new((s1 * x2 - s2 * x1) * r, (s1 * y2 - s2 * y1) * r, (s1 * z2 - s2 * z1) * r);
		
		tan1[i1] += sdir;
		tan1[i2] += sdir;
		tan1[i3] += sdir;
		
		tan2[i1] += tdir;
		tan2[i2] += tdir;
		tan2[i3] += tdir;
	}
	
	for i in 0..vertices.len() {
		let v = &mut vertices[i];
		let n = v.normal;
		let t = tan1[i];
		
		let tangent = (t - n * n.dot(t)).normalize();
		let w = if n.cross(t).dot(tan2[i]) < 0.0 {-1.0} else {1.0};
		v.tangent = Vec4::new(tangent.x, tangent.y, tangent.z, w);
	}
}

pub fn calculate_normals(vertices: &mut [Vertex], indices: &[u16]) {
	for i in (0..indices.len()).step_by(3) {
		let i1 = indices[i    ] as usize;
		let i2 = indices[i + 1] as usize;
		let i3 = indices[i + 2] as usize;
		
		let v1 = &vertices[i1];
		let v2 = &vertices[i2];
		let v3 = &vertices[i3];
		
		let normal = (v2.position - v1.position).cross(v3.position - v1.position).normalize();
		vertices[i1].normal = normal;
		vertices[i2].normal = normal;
		vertices[i3].normal = normal;
	}
}