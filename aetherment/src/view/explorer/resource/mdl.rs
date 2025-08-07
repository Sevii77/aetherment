use std::collections::HashMap;
use glam::Vec4Swizzles;
use noumenon::format::{external::Bytes, game::{Mdl, Mtrl, Tex}};
use crate::ui_ext::{InteractableScene, UiExt};

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
	
	fn has_changes(&self) -> bool {
		false
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) {
		let scene = self.scene.get_or_insert_with(|| {
			let mut scene = InteractableScene::new(renderer);
			
			scene.add_object(Box::new(renderer::Skybox::simple(renderer)));
			
			// load in textures and modify them for our viewing pleasure
			let mut textures = HashMap::new();
			let noumenon = crate::noumenon().unwrap();
			for lod in &self.mdl.lods {
				for mesh in &lod.meshes {
					let mtrl_path = Mdl::absolute_mtrl_path(&mesh.material, 1);
					if textures.contains_key(&mtrl_path) {continue}
					let Ok(mtrl) = noumenon.file::<Mtrl>(&mtrl_path) else {continue};
					
					// Some meshes have both Diffuse and ColorsetIndex
					// for the one i tested it doesnt seem like it is actually used ingame
					// chara/monster/m0934/obj/body/b0001/model/m0934b0001.mdl
					let mut diffuse_is_diffuse = false;
					let mut diffuse = None;
					let mut normal = None;
					
					
					for sampler in &mtrl.samplers {
						match sampler.typ {
							290653886 => { // Diffuse
								let Ok(tex) = noumenon.file::<Tex>(&sampler.texture) else {continue};
								diffuse = Some((tex.width, tex.height, tex.pixels));
								diffuse_is_diffuse = true;
							}
							
							1449103320 => { // ColorsetIndex
								let Ok(tex) = noumenon.file::<Tex>(&sampler.texture) else {continue};
								let pixels = tex.pixels
									.chunks_exact(4)
									.flat_map(|v| {
										let [r, g, _b, a] = v else {unreachable!()};
										let id = (*r as f32 / 17.0).round() as usize;
										let point = *g as f32 / 255.0;
										let row1 = &mtrl.colorsets[0].regular[id * 2];
										let row2 = &mtrl.colorsets[0].regular[id * 2 + 1];
										let clr = row1.diffuse * point + row2.diffuse * (1.0 - point);
										
										[
											(clr.x * 255.0) as u8,
											(clr.y * 255.0) as u8,
											(clr.z * 255.0) as u8,
											*a,
										]
									}).collect::<Vec<u8>>();
								
								if !diffuse_is_diffuse {
									diffuse = Some((tex.width, tex.height, pixels));
								}
							}
							
							207536625 => { // Normal
								let Ok(tex) = noumenon.file::<Tex>(&sampler.texture) else {continue};
								normal = Some((tex.width, tex.height, tex.pixels));
							}
							
							_ => {}
						}
					}
					
					textures.insert(mesh.material.clone(), (mtrl.shader.to_ascii_lowercase(), diffuse, normal));
				}
			}
			
			// do what shaders do to get accurate visuals
			// https://docs.google.com/spreadsheets/d/1kIKvVsW3fOnVeTi9iZlBDqJo6GWVn6K6BCUIRldEjhw/edit?gid=1406279597#gid=1406279597
			let textures = textures
				.into_iter()
				.map(|(k, (shader, mut diffuse, mut normal))| {
					// This doesnt actually seem to work, might be because of the render pipeline (probably is), cba for now
					// TODO: make this work
					match (shader.as_str(), &mut diffuse, &mut normal) {
						("character.shpk", Some((_, _, d_pixels)), Some((_, _, n_pixels))) |
						("characterlegacy.shpk", Some((_, _, d_pixels)), Some((_, _, n_pixels))) => {
							d_pixels.chunks_exact_mut(4)
								.zip(n_pixels.chunks_exact(4))
								.for_each(|(dv, nv)| dv[3] = nv[2])
						}
						
						_ => {}
					}
					
					let format = renderer::renderer::TextureFormat::Rgba8Unorm;
					let usage = renderer::renderer::TextureUsage::TEXTURE_BINDING;
					
					(
						k,
						(
							diffuse.map(|(w, h, p)| renderer.create_texture_initialized(w, h, format, usage, &p)),
							normal.map(|(w, h, p)| renderer.create_texture_initialized(w, h, format, usage, &p)),
						)
					)
				}).collect::<HashMap<_, _>>();
			
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
			
			scene
		});
		
		ui.splitter("splitter", crate::ui_ext::SplitterAxis::Horizontal, 0.8, |ui_left, ui_right| {
			let ui = ui_left;
			let size = ui.available_size();
			scene.render(renderer, size.x as usize, size.y as usize, ui);
			
			let ui = ui_right;
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
				ui.collapsing(format!("Mesh {mesh_index}"), |ui| {
					ui.label("Material");
					ui.text_edit_singleline(&mut mesh.material);
					
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
							}
							
							ui.horizontal(|ui| {
								if ui.button("➕").clicked() {
									submesh.attributes.push(self.add_attr.clone());
									self.add_attr.clear();
								}
								
								ui.text_edit_singleline(&mut self.add_attr);
							});
						});
					}
				});
			}
		});
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