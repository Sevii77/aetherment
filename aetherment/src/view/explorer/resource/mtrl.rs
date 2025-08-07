use noumenon::format::{external::Bytes, game::Mtrl};
use crate::ui_ext::UiExt;

pub struct MtrlView {
	changed: bool,
	mtrl: Mtrl,
}

impl MtrlView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			changed: false,
			mtrl: Mtrl::read(&mut std::io::Cursor::new(&data))?,
		})
	}
}

impl super::ResourceView for MtrlView {
	fn title(&self) -> String {
		"Material".to_string()
	}
	
	fn has_changes(&self) -> bool {
		self.changed
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) {
		egui::ScrollArea::both().show(ui, |ui| {
			ui.label("Shader");
			self.changed |= ui.text_edit_singleline(&mut self.mtrl.shader).changed();
			ui.spacer();
			
			ui.label("Uv Sets");
			for name in &mut self.mtrl.uvsets {
				self.changed |= ui.text_edit_singleline(name).changed();
			}
			ui.spacer();
			
			for colorset in &mut self.mtrl.colorsets {
				ui.collapsing(&colorset.name, |ui| {
					egui::Grid::new(&colorset.name).show(ui, |ui| {
						for (i, row) in colorset.regular.iter_mut().enumerate() {
							// i, too, love writing at the right side of my screen
							self.changed |= ui.color_edit(&mut row.diffuse)
								.on_hover_text("Diffuse")
								.changed();
							self.changed |= ui.num_edit(&mut row._diffuse_alpha, "")
								.on_hover_text("Diffuse (alpha, maybe)")
								.changed();
							
							self.changed |= ui.color_edit(&mut row.specular)
								.on_hover_text("Specular")
								.changed();
							self.changed |= ui.num_edit(&mut row._specular_alpha, "")
								.on_hover_text("Specular (alpha, maybe)")
								.changed();
							
							self.changed |= ui.color_edit(&mut row.emmisive)
								.on_hover_text("Emmisive")
								.changed();
							self.changed |= ui.num_edit(&mut row._emmisive_alpha, "")
								.on_hover_text("Emmisive (alpha, maybe)")
								.changed();
							
							self.changed |= ui.num_edit(&mut row.sheen_rate, "")
								.on_hover_text("Sheen rate")
								.changed();
							self.changed |= ui.num_edit(&mut row.sheen_tint_rate, "")
								.on_hover_text("Sheen tint rate")
								.changed();
							self.changed |= ui.num_edit(&mut row.sheen_aperature, "")
								.on_hover_text("Sheen aperature")
								.changed();
							self.changed |= ui.num_edit(&mut row._unknown15, "")
								.on_hover_text("_unknown15")
								.changed();
							self.changed |= ui.num_edit(&mut row.roughness, "")
								.on_hover_text("Roughness")
								.changed();
							self.changed |= ui.num_edit(&mut row._unknown17, "")
								.on_hover_text("_unknown17")
								.changed();
							self.changed |= ui.num_edit(&mut row.metalic, "")
								.on_hover_text("Metalic")
								.changed();
							self.changed |= ui.num_edit(&mut row.anisotropy, "")
								.on_hover_text("Anisotropy")
								.changed();
							self.changed |= ui.num_edit(&mut row._unknown20, "")
								.on_hover_text("_unknown20")
								.changed();
							self.changed |= ui.num_edit(&mut row.sphere_map_mask, "")
								.on_hover_text("Sphere map mask")
								.changed();
							self.changed |= ui.num_edit(&mut row._unknown22, "")
								.on_hover_text("_unknown22")
								.changed();
							self.changed |= ui.num_edit(&mut row._unknown23, "")
								.on_hover_text("_unknown23")
								.changed();
							self.changed |= ui.num_edit(&mut row.shader_id, "")
								.on_hover_text("Shader ID")
								.changed();
							self.changed |= ui.num_edit(&mut row.tile_index, "")
								.on_hover_text("Tile Index")
								.changed();
							self.changed |= ui.num_edit(&mut row.tile_alpha, "")
								.on_hover_text("Tile Alpha")
								.changed();
							self.changed |= ui.num_edit(&mut row.sphere_map_index, "")
								.on_hover_text("Sphere map index")
								.changed();
							
							self.changed |= ui.num_edit(&mut row.tile_transform.x_axis.x, "")
								.on_hover_text("Tile transformation XX")
								.changed();
							self.changed |= ui.num_edit(&mut row.tile_transform.x_axis.y, "")
								.on_hover_text("Tile transformation XY")
								.changed();
							self.changed |= ui.num_edit(&mut row.tile_transform.y_axis.x, "")
								.on_hover_text("Tile transformation YX")
								.changed();
							self.changed |= ui.num_edit(&mut row.tile_transform.y_axis.y, "")
								.on_hover_text("Tile transformation YY")
								.changed();
							
							if let Some(dye) = &mut colorset.dyes {
								self.changed |= ui.num_edit(&mut dye[i], "")
								.on_hover_text("Dye stuff idfk")
								.changed();
							}
							
							ui.end_row();
						}
					});
				});
			}
			
			if self.mtrl.colorsets.len() > 0 {
				ui.spacer();
			}
			
			ui.label("Constants");
			for constant in &mut self.mtrl.constants {
				ui.horizontal(|ui| {
					ui.label(format!("{:?}", constant.id));
					if constant.value.len() == 4 {
						self.changed |= ui.num_edit(constant.value_as::<f32>(), "").changed();
					} else {
						self.changed |= ui.num_multi_edit(&mut constant.value, "").changed();
					}
				});
			}
			ui.spacer();
			
			ui.label("Samplers");
			for sampler in &mut self.mtrl.samplers {
				ui.indent("sampler", |ui| {
					ui.label(format!("{:?}", sampler.typ));
					self.changed |= ui.text_edit_singleline(&mut sampler.texture).changed();
					self.changed |= ui.num_edit(&mut sampler.flags, "Flags").changed();
				});
				
				ui.spacer();
			}
			
			ui.label("Shader Keys");
			for (k, v) in &mut self.mtrl.shader_keys {
				ui.horizontal(|ui| {
					self.changed |= ui.num_edit(k, "").changed();
					self.changed |= ui.num_edit(v, "").changed();
				});
			}
			ui.spacer();
			
			ui.label("Shader Flags");
			self.changed |= ui.num_edit(&mut self.mtrl.shader_flags, "").changed();
		});
	}
}