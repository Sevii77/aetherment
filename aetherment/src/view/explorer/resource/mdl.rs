use glam::Vec4Swizzles;
use noumenon::format::{external::Bytes, game::Mdl};
use crate::ui_ext::InteractableScene;

pub struct MdlView {
	scene: Option<InteractableScene>,
	mdl: Mdl
}

impl MdlView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			scene: None,
			mdl: Mdl::read(&mut std::io::Cursor::new(&data))?,
		})
	}
}

impl super::ResourceView for MdlView {
	fn title(&self) -> String {
		"Model".to_string()
	}
	
	fn has_changes(&self) -> bool {
		false
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &crate::Renderer) {
		let scene = self.scene.get_or_insert_with(|| {
			let mut scene = InteractableScene::new(renderer);
			
			scene.add_object(Box::new(renderer::Skybox::simple(renderer)));
			
			for m in &self.mdl.meshes[0] {
				let mut vertices = m.vertices.iter().map(|v| renderer::vertex(v.position, v.normal, v.color, v.uv.xy())).collect::<Vec<_>>();
				renderer::calculate_tangents(&mut vertices, &m.indices);
				
				scene.add_object(Box::new(renderer::Mesh::new(renderer, &vertices, &m.indices)));
			}
			
			scene
		});
		
		let size = ui.available_size();
		scene.render(renderer, size.x as usize, size.y as usize, ui);
	}
}