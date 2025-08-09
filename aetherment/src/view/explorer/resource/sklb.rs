use noumenon::format::{external::Bytes, game::Sklb};
use renderer::vertex;
use crate::ui_ext::InteractableScene;

pub struct SklbView {
	sklb: Sklb,
	scene: Option<InteractableScene>,
}

impl SklbView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			sklb: Sklb::read(&mut std::io::Cursor::new(&data))?,
			scene: None,
		})
	}
}

impl super::ResourceView for SklbView {
	fn title(&self) -> String {
		"Skeleton".to_string()
	}
	
	fn has_changes(&self) -> bool {
		false
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) -> crate::view::explorer::Action {
		let scene = self.scene.get_or_insert_with(|| {
			let mut scene = InteractableScene::new(renderer);
			
			scene.add_object(Box::new(renderer::Skybox::simple(renderer)));
			
			let vertices_buf;
			let indices_buf;
			{
				let vertices = &mut [
					vertex(glam::vec3( 1.0, 0.0,  1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::ONE),
					vertex(glam::vec3(-1.0, 0.0,  1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::Y),
					vertex(glam::vec3(-1.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::ZERO),
					vertex(glam::vec3( 1.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::X),
					
					vertex(glam::vec3(-1.0, 0.0,  1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::ZERO),
					vertex(glam::vec3( 1.0, 0.0,  1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::X),
					vertex(glam::vec3( 0.0, 1.0,  0.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::Y),
					
					vertex(glam::vec3(-1.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::ZERO),
					vertex(glam::vec3(-1.0, 0.0,  1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::X),
					vertex(glam::vec3( 0.0, 1.0,  0.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::Y),
					
					vertex(glam::vec3( 1.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::ZERO),
					vertex(glam::vec3(-1.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::X),
					vertex(glam::vec3( 0.0, 1.0,  0.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::Y),
					
					vertex(glam::vec3( 1.0, 0.0,  1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::ZERO),
					vertex(glam::vec3( 1.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::X),
					vertex(glam::vec3( 0.0, 1.0,  0.0), glam::Vec3::ZERO, glam::Vec4::ONE, glam::Vec2::Y),
				];
				
				let indices = &[
					0, 1, 2, 2, 3, 0,
					4, 5, 6,
					7, 8, 9,
					10, 11, 12,
					13, 14, 15,
				];
				
				renderer::calculate_normals(vertices, indices);
				renderer::calculate_tangents(vertices, indices);
				
				vertices_buf = renderer::mesh::create_vertex_buffer(renderer, vertices);
				indices_buf = renderer::mesh::create_index_buffer(renderer, indices);
			}
			
			let mut global_matrixes = Vec::<glam::Mat4>::new();
			for bone in &self.sklb.bones {
				let mut matrix = glam::Mat4::from_rotation_translation(bone.rotation, bone.translation);
				if bone.parent >= 0 {
					matrix = global_matrixes[bone.parent as usize] * matrix;
				}
				global_matrixes.push(matrix);
				
				let id = scene.add_object(Box::new(renderer::Mesh::new_buffer(renderer, vertices_buf.clone(), indices_buf.clone())));
				let obj = scene.get_object_mut(id).unwrap();
				*obj.get_matrix_mut() = matrix;
				obj.set_scale(glam::vec3(0.01, 0.1, 0.01));
			}
			
			scene
		});
		
		let size = ui.available_size();
		scene.render(renderer, size.x as usize, size.y as usize, ui);
		
		crate::view::explorer::Action::None
	}
	
	fn export(&self) -> super::Export {
		super::Export::Invalid
	}
}