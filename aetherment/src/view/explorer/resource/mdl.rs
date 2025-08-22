use std::collections::HashMap;
use glam::Vec4Swizzles;
use noumenon::format::{external::Bytes, game::Mdl};
use crate::{ui_ext::{InteractableScene, UiExt}, view::explorer::Action};

pub struct MdlView {
	mdl: Mdl,
	scene: Option<InteractableScene>,
	lod: usize,
	objects: HashMap<(usize, usize, usize), usize>,
	shapes: Vec<(String, bool)>,
	add_attr: String,
}

impl MdlView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			mdl: Mdl::read(&mut std::io::Cursor::new(&data))?,
			scene: None,
			lod: 0,
			objects: HashMap::new(),
			shapes: Vec::new(),
			add_attr: String::new(),
		})
	}
}

impl super::ResourceView for MdlView {
	fn title(&self) -> String {
		"Model".to_string()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) -> Action {
		let mut act = Action::None;
		
		let scene = self.scene.get_or_insert_with(|| {
			let mut scene = InteractableScene::new(renderer);
			
			scene.add_object(Box::new(renderer::Skybox::simple(renderer)));
			
			// load in textures and modify them for our viewing pleasure
			let textures = self.mdl.bake_materials(|path| {
				crate::noumenon_instance().unwrap().file::<Vec<u8>>(path).ok()
			}).into_iter()
				.map(|(k, material)| (k, (
					material.diffuse.map(|v| renderer.create_texture_initialized(v.width, v.height, renderer::renderer::TextureFormat::Rgba8Unorm, renderer::renderer::TextureUsage::TEXTURE_BINDING, &v.data)),
					material.normal.map(|v| renderer.create_texture_initialized(v.width, v.height, renderer::renderer::TextureFormat::Rgba8Unorm, renderer::renderer::TextureUsage::TEXTURE_BINDING, &v.data)),
				))).collect::<HashMap<_, _>>();
			
			// mesh
			for (lod_index, lod) in self.mdl.lods.iter().enumerate() {
				for (mesh_index, mesh) in lod.meshes.iter().enumerate() {
					for (submesh_index, submesh) in mesh.submeshes.iter().enumerate() {
						let mut vertices = submesh.vertices.iter().map(|v| renderer::vertex(v.position, v.normal, glam::Vec4::ONE, v.uv.xy())).collect::<Vec<_>>();
						
						// merge all shape indices so that we can correctly calculate tangents once
						let mut all_indices = submesh.indices.clone();
						for shape in &submesh.shapes {
							let mut indices = submesh.indices.clone();
							for v in &shape.values {
								indices[v.index as usize] = v.new_vertex;
							}
							all_indices.extend_from_slice(&indices);
						}
						renderer::calculate_tangents(&mut vertices, &all_indices);
						
						let id = scene.add_object(Box::new(renderer::Mesh::new(renderer, &vertices, &create_indices(submesh, &self.shapes))));
						let obj = scene.get_object_mut(id).unwrap();
						
						if let Some((diffuse, normal)) = textures.get(&mesh.material) {
							let resources = obj.get_shader_resources_mut();
							
							if let Some(diffuse) = diffuse {
								resources[0] = renderer::renderer::ShaderResource::Texture(diffuse.clone());
							}
							
							if let Some(normal) = normal {
								resources[2] = renderer::renderer::ShaderResource::Texture(normal.clone());
							}
						}
						
						if lod_index != self.lod {
							*obj.get_visible_mut() = false;
						}
						
						self.objects.insert((lod_index, mesh_index, submesh_index), id);
						
						for shape in &submesh.shapes {
							if self.shapes.iter().any(|v| v.0 == shape.name) {continue}
							self.shapes.push((shape.name.clone(), false));
						}
					}
				}
			}
			
			// camera defaults
			let mut min = glam::Vec3::MAX;
			let mut max = glam::Vec3::MIN;
			for lod in &self.mdl.lods {
				for mesh in &lod.meshes {
					for submesh in &mesh.submeshes {
						for vertex in &submesh.vertices {
							for i in 0..3 {
								min[i] = min[i].min(vertex.position[i]);
								max[i] = max[i].max(vertex.position[i]);
							}
						}
					}
				}
			}
			
			let size = max - min;
			scene.set_camera_defaults(min * 0.5 + max * 0.5, (size.x.max(size.y).max(size.z) * 2.0).max(1.0));
			
			scene
		});
		
		ui.splitter("splitter", crate::ui_ext::SplitterAxis::Horizontal, 0.8, |ui_left, ui_right| {
			let ui = ui_left;
			let size = ui.available_size();
			scene.render(renderer, size.x as usize, size.y as usize, ui);
			
			egui::ScrollArea::vertical().auto_shrink(false).show(ui_right, |ui| {
				if self.mdl.lods.len() > 1 {
					ui.combo(format!("LOD {}", self.lod), "", |ui| {
						for i in 0..self.mdl.lods.len() {
							if ui.selectable_label(self.lod == i, format!("LOD {i}")).clicked() {
								self.lod = i;
								
								for ((mesh_lod, _, _), id) in &self.objects {
									*scene.get_object_mut(*id).unwrap().get_visible_mut() = *mesh_lod == i;
								}
							}
						}
					});
				}
				
				ui.spacer();
				
				let mut update_objects = false;
				if self.shapes.len() > 0 {
					ui.label("Shapes");
					for (name, state) in &mut self.shapes {
						update_objects |= ui.checkbox(state, &*name).changed();
					}
					
					ui.spacer();
				}
				
				if update_objects {
					for ((lod_index, mesh_index, submesh_index), obj_id) in &self.objects {
						let submesh = &self.mdl.lods[*lod_index].meshes[*mesh_index].submeshes[*submesh_index];
						let obj = scene.get_object_mut(*obj_id).unwrap().as_any_mut().downcast_mut::<renderer::Mesh>().unwrap();
						obj.set_indices(renderer, &create_indices(submesh, &self.shapes));
					}
				}
				
				for (mesh_index, mesh) in self.mdl.lods[self.lod].meshes.iter_mut().enumerate() {
					let resp = ui.collapsing(format!("Mesh {mesh_index}"), |ui| {
						ui.label("Material");
						let resp = ui.text_edit_singleline(&mut mesh.material);
						resp.context_menu(|ui| {
							if ui.button("Open in new tab").clicked() {
								act = Action::OpenNew(crate::view::explorer::TabType::Resource(super::Path::Game(Mdl::absolute_mtrl_path(&mesh.material, 1))));
								ui.close_menu();
							}
						});
						
						if resp.changed() {
							act.or(Action::Changed);
						}
						
						for (submesh_index, submesh) in mesh.submeshes.iter_mut().enumerate() {
							let obj = scene.get_object_mut(self.objects[&(self.lod, mesh_index, submesh_index)]).unwrap();
							
							ui.spacer();
							ui.checkbox(obj.get_visible_mut(), format!("Submesh {submesh_index}"));
							ui.indent("options", |ui| {
								ui.label("Attributes");
								let mut delete = None;
								for (i, attr) in submesh.attributes.iter().enumerate() {
									ui.horizontal(|ui| {
										if ui.button("🗑").clicked() {
											delete = Some(i);
										}
										
										ui.label(attr);
									});
								}
								
								if let Some(i) = delete {
									submesh.attributes.remove(i);
									act.or(Action::Changed);
								}
								
								ui.horizontal(|ui| {
									if ui.button("➕").clicked() {
										submesh.attributes.push(self.add_attr.clone());
										self.add_attr.clear();
										act.or(Action::Changed);
									}
									
									ui.text_edit_singleline(&mut self.add_attr);
								});
							});
						}
					});
					
					let sublen = mesh.submeshes.len();
					if sublen > 0 {
						resp.header_response.context_menu(|ui| {
							let all_visible = (0..sublen)
								.all(|i| scene.get_object(self.objects[&(self.lod, mesh_index, i)]).unwrap().get_visible());
							
							if ui.button(if all_visible {"Hide All"} else {"Show All"}).clicked() {
								for i in 0..sublen {
									*scene.get_object_mut(self.objects[&(self.lod, mesh_index, i)])
										.unwrap()
										.get_visible_mut() = !all_visible;
								}
							}
						});
					}
				}
			});
		});
		
		act
	}
	
	fn export(&self) -> super::Export {
		super::Export::Converter(noumenon::Convert::Mdl(self.mdl.clone()))
	}
}

fn create_indices(submesh: &noumenon::format::game::mdl::Submesh, shape_states: &[(String, bool)]) -> Vec<u16> {
	let mut indices = submesh.indices.to_vec();
	for shape in &submesh.shapes {
		if !shape_states.iter().find_map(|v| if v.0 == shape.name {Some(v.1)} else {None}).unwrap_or(false) {continue}
		for val in &shape.values {
			indices[val.index as usize] = val.new_vertex;
		}
	}
	
	indices
}