use noumenon::format::{external::Bytes, game::{mtrl::{self, AddressMode}, Mtrl}};
use crate::{ui_ext::UiExt, EnumTools};

impl EnumTools for AddressMode {
	type Iterator = std::array::IntoIter<Self, 4>;

	fn to_str(&self) -> &'static str {
		match self {
			AddressMode::Wrap   => "Wrap",
			AddressMode::Mirror => "Mirror",
			AddressMode::Clamp  => "Clamp",
			AddressMode::Border => "Border",
		}
	}

	fn iter() -> Self::Iterator {
		[
			AddressMode::Wrap,
			AddressMode::Mirror,
			AddressMode::Clamp,
			AddressMode::Border,
		].into_iter()
	}
}

pub struct MtrlView {
	changed: bool,
	mtrl: Mtrl,
	view_row: usize,
}

impl MtrlView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		Ok(Self {
			changed: false,
			mtrl: Mtrl::read(&mut std::io::Cursor::new(&data))?,
			view_row: 0,
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
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) -> crate::view::explorer::Action {
		egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
			ui.label("Shader");
			ui.combo(self.mtrl.shader.to_string(), "", |ui| {
				for v in mtrl::USED_SHADERS {
					self.changed |= ui.selectable_value(&mut self.mtrl.shader, v.to_string(), v).changed();
				}
				self.changed |= ui.text_edit_singleline(&mut self.mtrl.shader).changed();
			});
			
			ui.spacer();
			ui.label("Uv Sets");
			for name in &mut self.mtrl.uvsets {
				self.changed |= ui.text_edit_singleline(name).changed();
			}
			
			for colorset in &mut self.mtrl.colorsets {
				ui.spacer();
				ui.collapsing(format!("Colorset {}", colorset.name), |ui| {
					{
						let size = ui.spacing().interact_size.y;
						let (rect, resp) = ui.allocate_exact_size(egui::vec2(size * 16.0 + 17.0, size * 2.0 + 3.0), egui::Sense::click());
						let painter = ui.painter_at(rect);
						let style = ui.style().interact(&resp);
						
						painter.rect_filled(rect, style.corner_radius, style.bg_fill); // border
						
						for y in 0..2 {
							for x in 0..16 {
								let row_id = x * 2 + y;
								let offset = if self.view_row == row_id {size * 0.2} else {0.0};
								let clr = colorset.regular[row_id].diffuse;
								
								painter.rect_filled(
									egui::Rect::from_min_size(
										rect.min + egui::vec2(x as f32 * size + (x + 1) as f32 + offset, y as f32 * size + (y + 1) as f32 + offset),
										egui::vec2(size - offset * 2.0, size - offset * 2.0),
									),
									if x == 0 && y == 0 {egui::CornerRadius{nw: style.corner_radius.nw, ..Default::default()}}
										else if x == 0 && y == 1 {egui::CornerRadius{sw: style.corner_radius.sw, ..Default::default()}}
										else if x == 15 && y == 0 {egui::CornerRadius{ne: style.corner_radius.ne, ..Default::default()}}
										else if x == 15 && y == 1 {egui::CornerRadius{se: style.corner_radius.se, ..Default::default()}}
										else {egui::CornerRadius::ZERO},
									egui::Color32::from_rgb((clr.x * 255.0) as u8, (clr.y * 255.0) as u8, (clr.z * 255.0) as u8),
								);
							}
						}
						
						if let Some(pos) = resp.interact_pointer_pos() {
							let pos = pos - rect.min;
							self.view_row = ((pos.x / (size + 1.0)) as usize).min(15) * 2 + ((pos.y / (size + 1.0)) as usize).min(1);
						}
					}
					
					ui.spacer();
					ui.label("Colorset");
					
					egui::Grid::new("row").show(ui, |ui| {
						let row = &mut colorset.regular[self.view_row];
						
						ui.label("Diffuse");
						self.changed |= ui.color_edit(&mut row.diffuse).changed();
						self.changed |= ui.num_edit(&mut row._diffuse_alpha, "").changed();
						ui.end_row();
						
						ui.label("Specular");
						self.changed |= ui.color_edit(&mut row.specular).changed();
						self.changed |= ui.num_edit(&mut row._specular_alpha, "").changed();
						ui.end_row();
						
						ui.label("Emmisive");
						self.changed |= ui.color_edit(&mut row.emmisive).changed();
						self.changed |= ui.num_edit(&mut row._emmisive_alpha, "").changed();
						ui.end_row();
						
						ui.label("Sheen");
						self.changed |= ui.num_edit(&mut row.sheen_rate, "")
							.on_hover_text("Rate").changed();
						self.changed |= ui.num_edit(&mut row.sheen_tint_rate, "")
							.on_hover_text("Tint Rate").changed();
						self.changed |= ui.num_edit(&mut row.sheen_aperature, "")
							.on_hover_text("Aperature").changed();
						ui.end_row();
						
						ui.label("Roughness");
						self.changed |= ui.num_edit(&mut row.roughness, "").changed();
						ui.end_row();
						
						ui.label("Metalic");
						self.changed |= ui.num_edit(&mut row.metalic, "").changed();
						ui.end_row();
						
						ui.label("Anisotropy");
						self.changed |= ui.num_edit(&mut row.anisotropy, "").changed();
						ui.end_row();
						
						ui.label("Shader ID");
						self.changed |= ui.num_edit(&mut row.shader_id, "").changed();
						ui.end_row();
						
						ui.label("Sphere Map");
						self.changed |= ui.num_edit(&mut row.sphere_map_index, "")
							.on_hover_text("Index").changed();
						self.changed |= ui.num_edit(&mut row.sphere_map_mask, "")
							.on_hover_text("Mask").changed();
						ui.end_row();
						
						ui.label("Unknowns");
						self.changed |= ui.num_edit(&mut row._unknown15, "")
							.on_hover_text("_unknown15").changed();
						self.changed |= ui.num_edit(&mut row._unknown17, "")
							.on_hover_text("_unknown17").changed();
						self.changed |= ui.num_edit(&mut row._unknown20, "")
							.on_hover_text("_unknown20").changed();
						self.changed |= ui.num_edit(&mut row._unknown22, "")
							.on_hover_text("_unknown22").changed();
						self.changed |= ui.num_edit(&mut row._unknown23, "")
							.on_hover_text("_unknown23").changed();
						ui.end_row();
						
						// TODO: make this nicer, tile preview and shit
						ui.label("Tilemap");
						self.changed |= ui.num_edit(&mut row.tile_index, "")
							.on_hover_text("Index").changed();
						self.changed |= ui.num_edit(&mut row.tile_alpha, "")
							.on_hover_text("Alpha").changed();
						self.changed |= ui.num_edit(&mut row.tile_transform.x_axis.x, "")
							.on_hover_text("Transformation XX").changed();
						self.changed |= ui.num_edit(&mut row.tile_transform.x_axis.y, "")
							.on_hover_text("Transformation XY").changed();
						self.changed |= ui.num_edit(&mut row.tile_transform.y_axis.x, "")
							.on_hover_text("Transformation YX").changed();
						self.changed |= ui.num_edit(&mut row.tile_transform.y_axis.y, "")
							.on_hover_text("Transformation YY").changed();
						ui.end_row();
					});
						
					if let Some(dyes) = &mut colorset.dyes {
						ui.spacer();
						ui.label("Dye");
						
						egui::Grid::new("dyerow").show(ui, |ui| {
							let row = &mut dyes[self.view_row];
							
							ui.label("Template");
							self.changed |= ui.num_edit(&mut row.template, "").changed();
							ui.end_row();
							
							ui.label("Channel");
							self.changed |= ui.num_edit(&mut row.channel, "").changed();
							ui.end_row();
							
							ui.label("Diffuse");
							self.changed |= ui.checkbox(&mut row.diffuse, "").changed();
							ui.end_row();
							
							ui.label("Specular");
							self.changed |= ui.checkbox(&mut row.specular, "").changed();
							ui.end_row();
							
							ui.label("Emmisive");
							self.changed |= ui.checkbox(&mut row.emmisive, "").changed();
							ui.end_row();
							
							ui.label("Scalar3");
							self.changed |= ui.checkbox(&mut row.scalar3, "").changed();
							ui.end_row();
							
							ui.label("Roughness");
							self.changed |= ui.checkbox(&mut row.roughness, "").changed();
							ui.end_row();
							
							ui.label("Metalic");
							self.changed |= ui.checkbox(&mut row.metalic, "").changed();
							ui.end_row();
							
							ui.label("Anisotropy");
							self.changed |= ui.checkbox(&mut row.anisotropy, "").changed();
							ui.end_row();
							
							ui.label("Sheen");
							ui.horizontal(|ui| {
								self.changed |= ui.checkbox(&mut row.sheen_rate, "")
									.on_hover_text("Rate").changed();
								self.changed |= ui.checkbox(&mut row.sheen_tint_rate, "")
									.on_hover_text("Tint Rate").changed();
								self.changed |= ui.checkbox(&mut row.sheen_aperature, "")
									.on_hover_text("Aperature").changed();
							});
							ui.end_row();
							
							ui.label("Sphere Map");
							ui.horizontal(|ui| {
								self.changed |= ui.checkbox(&mut row.sphere_map_index, "")
									.on_hover_text("Index").changed();
								self.changed |= ui.checkbox(&mut row.sphere_map_mask, "")
									.on_hover_text("Mask").changed();
							});
							ui.end_row();
						});
					}
				});
			}
			
			ui.spacer();
			ui.collapsing("Textures", |ui| {
				let mut delete = None;
				for (i, sampler) in self.mtrl.samplers.iter_mut().enumerate() {
					ui.combo_id(shader_param_name(sampler.id), i, |ui| {
						for v in mtrl::USED_SAMPLERS {
							self.changed |= ui.selectable_value(&mut sampler.id, v, shader_param_name(v)).changed();
						}
					});
					
					let valid = 'v: {
						let Some((_, samplers)) = mtrl::USED_SHADER_SAMPLERS.iter().find(|v| v.0 == self.mtrl.shader) else {break 'v false};
						samplers.contains(&sampler.id)
					};
					
					if !valid {
						ui.label(egui::RichText::new("The game does not use this sampler for this shader.").color(egui::Color32::RED));
					}
					
					ui.indent(i, |ui| {
						self.changed |= ui.text_edit_singleline(&mut sampler.texture).changed();
						self.changed |= ui.combo_enum(&mut sampler.u_address_mode, "U Address Mode").changed();
						self.changed |= ui.combo_enum(&mut sampler.v_address_mode, "V Address Mode").changed();
						self.changed |= ui.num_edit_range(&mut sampler.lod_bias, "Lod Bias", -8.0..=7.984375).changed();
						self.changed |= ui.num_edit_range(&mut sampler.min_lod, "Minimum Lod", 0..=15).changed();
						
						if ui.button("🗑 Delete texture").clicked() {
							delete = Some(i);
						}
					});
					
					ui.spacer();
				}
				
				if let Some(i) = delete {
					self.mtrl.samplers.remove(i);
				}
				
				if ui.button("➕ Add new texture").clicked() {
					self.mtrl.samplers.push(mtrl::Sampler {
						id: mtrl::USED_SAMPLERS[0],
						texture: String::new(),
						u_address_mode: AddressMode::Wrap,
						v_address_mode: AddressMode::Wrap,
						lod_bias: 0.5,
						min_lod: 0,
					});
				}
			});
			
			ui.spacer();
			ui.collapsing("Constants", |ui| {
				for constant in &mut self.mtrl.constants {
					ui.horizontal(|ui| {
						if constant.value.len() % 4 == 0 {
							self.changed |= ui.num_multi_edit(constant.value_as::<f32>(), "").changed();
						} else {
							self.changed |= ui.num_multi_edit(&mut constant.value, "").changed();
						}
						ui.label(shader_param_name(constant.id));
					});
				}
			});
			
			ui.spacer();
			ui.collapsing("Shader Keys", |ui| {
				for (k, v) in &mut self.mtrl.shader_keys {
					ui.horizontal(|ui| {
						self.changed |= ui.num_edit(k, "").changed();
						self.changed |= ui.num_edit(v, "").changed();
					});
				}
			});
			
			ui.spacer();
			ui.label("Shader Flags");
			self.changed |= ui.num_edit(&mut self.mtrl.shader_flags, "").changed();
		});
		
		crate::view::explorer::Action::None
	}
}

fn shader_param_name(id: u32) -> String {
	mtrl::shader_param_name(id).unwrap_or_else(|| format!("unknown.{}", id))
}