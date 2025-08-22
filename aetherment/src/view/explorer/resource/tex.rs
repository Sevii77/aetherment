use std::collections::HashMap;
use noumenon::format::{external::Bytes, game::Tex};
use crate::{modman::{composite::tex::{self as comp, OptionSetting}, meta}, ui_ext::UiExt, view::explorer::{Action, ModInfo}, EnumTools};

pub struct TexView {
	tex: TexType,
	mod_info: Option<ModInfo>,
	preview: Option<Result<egui::TextureHandle, comp::CompositeError>>,
	path: super::Path,
	changed: bool,
	
	depth: u32,
	mip: u32,
	
	new_mod_file: Option<(crate::ui_ext::ImporterDialog, usize, Option<usize>)>,
	new_game_file: String,
	new_modifier: comp::Modifier,
}

impl TexView {
	pub fn new(path: &super::Path) -> Result<Self, crate::resource_loader::BacktraceError> {
		let data = super::read_file(path)?;
		
		let (tex, mod_info) = if path.is_composite() {
			let super::Path::Mod{mod_meta, mod_root, option, ..} = path else {unreachable!()};
			
			(
				TexType::Composite(Composite {
					settings: crate::modman::settings::CollectionSettings::from_meta(&mod_meta.borrow()),
					composite: serde_json::from_slice::<comp::Tex>(&data)?,
					textures: HashMap::new(),
					textures_previews: HashMap::new(),
				}),
				Some(ModInfo {
					root: mod_root.to_path_buf(),
					meta: mod_meta.clone(),
					option: option.clone(),
				})
			)
		} else {
			(
				TexType::Tex(Tex::read(&mut std::io::Cursor::new(&data))?),
				None
			)
		};
		
		Ok(Self {
			tex,
			mod_info,
			preview: None,
			path: path.clone(),
			changed: false,
			
			depth: 0,
			mip: 0,
			
			new_mod_file: None,
			new_game_file: String::new(),
			new_modifier: comp::Modifier::iter().next().unwrap(),
		})
	}
	
	fn create_comp_tex(&mut self) {
		let mod_info = self.mod_info.as_ref().unwrap();
		self.tex = TexType::Composite(Composite {
			settings: crate::modman::settings::CollectionSettings::from_meta(&mod_info.meta.borrow()),
			composite: comp::Tex {
				layers: match &self.path {
					super::Path::Game(game_path) =>
						vec![
							comp::Layer {
								name: "Game Texture".to_string(),
								path: crate::modman::Path::Game(game_path.clone()),
								modifiers: Vec::new(),
								blend: comp::Blend::Normal,
							}
						],
					
					super::Path::Real(_) => Vec::new(),
					
					super::Path::Mod{game_path, ..} =>
						vec![
							comp::Layer {
								name: "Mod Texture".to_string(),
								path: crate::modman::Path::Mod(game_path.clone()),
								modifiers: Vec::new(),
								blend: comp::Blend::Normal,
							}
						],
				}
			},
			textures: HashMap::new(),
			textures_previews: HashMap::new(),
		});
	}
	
	fn load_preview(&mut self, ctx: &egui::Context) {
		match &self.tex {
			TexType::Tex(tex) => {
				let slice = tex.slice(self.depth, self.mip);
				let data = egui::ColorImage {
					size: [slice.width as usize, slice.height as usize],
					pixels: slice.pixels.chunks_exact(4).map(|v| egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3])).collect(),
				};
				
				if let Some(Ok(img)) = self.preview.as_mut() {
					img.set(data, egui::TextureOptions::NEAREST);
				} else {
					self.preview = Some(Ok(ctx.load_texture("explorer::resource::tex.preview", data, egui::TextureOptions::NEAREST)));
				}
			}
			
			TexType::Composite(comp) => {
				match comp.composite.composite_raw_hashmap(&comp.settings, comp.textures.iter().collect()) {
					Ok((w, h, data)) =>
						self.preview = Some(Ok(ctx.load_texture("explorer::resource::tex.preview", egui::ColorImage {
							size: [w as usize, h as usize],
							pixels: data.chunks_exact(4).map(|v| egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3])).collect(),
						}, egui::TextureOptions::NEAREST))),
					
					Err(e) =>
						self.preview = Some(Err(e)),
				}
			}
		}
	}
	
	fn draw_layers(&mut self, ui: &mut egui::Ui) -> Action {
		if !matches!(self.tex, TexType::Composite(_)) {return Action::None}
		let Some(mod_info) = &self.mod_info else {return Action::None};
		
		let mut changed = false;
		let mut redraw = false;
		
		// New layer
		ui.menu_button("➕ Add layer", |ui| {
			if let Some(path) = import_file(ui, &mut self.new_mod_file, &mut self.new_game_file, mod_info, 0, None) {
				let TexType::Composite(comp) = &mut self.tex else {unreachable!()};
				comp.composite.layers.insert(0, comp::Layer {
					name: "New Layer".to_string(),
					path,
					modifiers: Vec::new(),
					blend: comp::Blend::Normal,
				});
				
				redraw = true;
				ui.close_menu();
			}
		});
		
		let TexType::Composite(comp) = &mut self.tex else {unreachable!()};
		comp.settings = crate::modman::settings::CollectionSettings::from_meta(&mod_info.meta.borrow());
		
		if let Some((dialog, layer_id, modifier_id)) = &mut self.new_mod_file {
			match dialog.show(ui) {
				Ok(crate::ui_ext::DialogResult::Success(data)) => 'o: {
					let Some(layer) = comp.composite.layers.get_mut(*layer_id) else {break 'o};
					let Some(mod_info) = &self.mod_info else {break 'o};
					// let game_path = self.path.as_path();
					// let option_dirs = mod_info.option.as_ref().map_or(String::new(), |(a, b)| format!("{a}/{b}/"));
					let hash = crate::hash_str(blake3::hash(&data));
					let ext = self.path.ext();
					let path_rel = format!("{hash}.{ext}");
					// let path_rel = format!("{game_path}/{option_dirs}{hash}.{ext}");
					
					let file_path = mod_info.root.join("files").join(&path_rel);
					_ = std::fs::create_dir_all(file_path.parent().unwrap());
					
					if let Err(err) = std::fs::write(file_path, data) {
						log!(err, "Failed writing file ({err:?})");
						self.new_mod_file = None;
						break 'o;
					}
					
					match modifier_id {
						Some(modifier) => {
							let Some(modifier) = layer.modifiers.get_mut(*modifier) else {break 'o};
							match modifier {
								comp::Modifier::AlphaMask{path, ..} |
								comp::Modifier::AlphaMaskAlphaStretch{path, ..} =>
									*path = crate::modman::Path::Mod(path_rel),
								
								_ => {}
							}
						}
						
						None =>
							layer.path = crate::modman::Path::Mod(path_rel),
					}
					
					redraw = true;
					self.new_mod_file = None;
				}
				
				Ok(crate::ui_ext::DialogResult::Cancelled) =>
					self.new_mod_file = None,
				
				Ok(crate::ui_ext::DialogResult::Busy) => {}
				
				Err(err) => {
					log!(err, "failed importing file {err:?}");
					self.new_mod_file = None;
				}
			}
		}
		
		// Layers
		let len = comp.composite.layers.len();
		let mut delete = None;
		egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
			redraw |= egui_dnd::dnd(ui, "layers").show_vec(&mut comp.composite.layers, |ui, layer, handle, state| {
				let layer_id = state.index;
				ui.push_id(layer_id, |ui| {
					ui.add_space(16.0);
					
					ui.horizontal(|ui| {
						handle.ui(ui, |ui| {
							if let Some(img) = comp.textures_previews.get(&layer.path) {
								preview(ui, img, egui::vec2(32.0, 32.0), true, 8).on_hover_ui(|ui| {
									ui.label(path_name(&layer.path));
									preview(ui, img, ui.available_size(), false, 32);
								})
							} else {
								ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::all()).1.on_hover_ui(|ui| {
									ui.label(path_name(&layer.path));
								})
							}.context_menu(|ui| {
								if ui.button("🗑 Delete Layer").clicked() {
									delete = Some(layer_id);
									ui.close_menu();
								}
							});
						});
						
						ui.vertical(|ui| {
							ui.add_space(8.0);
							
							ui.horizontal(|ui| {
								ui.label((len - 1 - state.index).to_string());
								changed |= ui.text_edit_singleline(&mut layer.name).changed();
							});
							
							if layer_id != len - 1 {
								redraw |= ui.combo_enum(&mut layer.blend, "Blend").changed();
							}
						});
					});
					
					ui.horizontal(|ui| {
						ui.add_space(32.0);
						ui.vertical(|ui| {
							let mut delete = None;
							for (i, modifier) in layer.modifiers.iter_mut().enumerate() {
								ui.push_id(i, |ui| {
									let (resp, c) = draw_modifier(ui, modifier, &comp.textures_previews, &comp.settings);
									redraw |= c;
									
									resp.context_menu(|ui| {
										if ui.button("🗑 Delete modifier").clicked() {
											delete = Some(i);
											ui.close_menu();
										}
									})
								});
							}
							
							if let Some(i) = delete {
								layer.modifiers.remove(i);
								redraw = true;
							}
							
							ui.horizontal(|ui| {
								let mut add = false;
								let mut new_modifier = self.new_modifier.clone();
								match &mut new_modifier {
									comp::Modifier::AlphaMask{path, ..} |
									comp::Modifier::AlphaMaskAlphaStretch{path, ..} =>
										_ = ui.menu_button("➕ Add modifier", |ui| {
											if let Some(selected_path) = import_file(ui, &mut self.new_mod_file, &mut self.new_game_file, mod_info, layer_id, Some(layer.modifiers.len())) {
												*path = selected_path;
												add = true;
												ui.close_menu();
											}
										}),
									
									_ => if ui.button("➕ Add modifier").clicked() {
										add = true;
									},
								}
								
								if add {
									redraw = true;
									layer.modifiers.push(new_modifier.clone());
								}
								
								ui.combo_enum_id(&mut self.new_modifier, "new modifier");
							});
						});
					});
				});
			}).is_drag_finished();
		});
		
		if let Some(i) = delete {
			comp.composite.layers.remove(i);
			redraw = true;
		}
		
		if redraw {
			_ = comp.fetch_textures(ui.ctx(), self.mod_info.as_ref());
			self.load_preview(ui.ctx());
		}
		
		if redraw || changed {Action::Changed} else {Action::None}
	}
}

impl super::ResourceView for TexView {
	fn title(&self) -> String {
		"Texture".to_string()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) -> Action {
		let mut act = Action::None;
		
		if self.preview.is_none() {
			if let TexType::Composite(comp) = &mut self.tex {
				_ = comp.fetch_textures(ui.ctx(), self.mod_info.as_ref());
			}
			
			self.load_preview(ui.ctx());
		}
		
		ui.splitter("splitter", crate::ui_ext::SplitterAxis::Horizontal, 0.7, |ui_left, ui_right| {
			let ui = ui_left;
			if let Some(img) = &self.preview {
				match img {
					Ok(img) => _ = preview(ui, img, ui.available_size(), false, 32),
					
					Err(err) => ui.centered("error", crate::ui_ext::Axis::Both, |ui| {
						ui.set_max_width(ui.available_width());
						ui.label(egui::RichText::new(err.to_string()).color(egui::Color32::RED).heading());
					}),
				}
			}
			
			let ui = ui_right;
			act.or(self.draw_layers(ui));
			
			if matches!(self.tex, TexType::Tex(_)) && ui.button("Convert to composite texture").clicked() {
				if self.mod_info.is_some() {
					self.create_comp_tex();
					act = Action::Changed;
				} else {
					act = Action::ModAssign;
					self.changed = true;
				}
			}
		});
		
		if self.changed && matches!(act, Action::None) {
			act.or(Action::Changed);
			self.changed = false;
		}
		
		act
	}
	
	fn assign_mod(&mut self, mod_info: ModInfo) {
		self.mod_info = Some(mod_info);
		self.create_comp_tex();
		self.preview = None;
		self.changed = true;
	}
	
	fn is_composite(&self) -> bool {
		match self.tex {
			TexType::Tex(_) => false,
			TexType::Composite(_) => true,
		}
	}
	
	fn export(&self) -> super::Export {
		match &self.tex {
			TexType::Tex(tex) => super::Export::Converter(noumenon::Convert::Tex(tex.clone())),
			TexType::Composite(comp) => super::Export::Bytes(crate::json_pretty(&comp.composite).unwrap().into_bytes()),
		}
	}
}

enum TexType {
	Tex(Tex),
	Composite(Composite),
}

struct Composite {
	settings: crate::modman::settings::CollectionSettings,
	composite: comp::Tex,
	textures: HashMap<crate::modman::Path, Tex>,
	textures_previews: HashMap<crate::modman::Path, egui::TextureHandle>,
}

impl Composite {
	fn fetch_textures(&mut self, ctx: &egui::Context, mod_info: Option<&ModInfo>) -> Result<(), crate::resource_loader::BacktraceError> {
		let mod_info = mod_info.ok_or("Invalid mod info")?;
		
		let mut textures = HashMap::new();
		let mut previews = HashMap::new();
		let mut add_path = |path: &crate::modman::Path| -> Result<(), crate::resource_loader::BacktraceError> {
			// log!("layer path {file_path:?}");
			let file_path = match path {
				crate::modman::Path::Mod(v) => super::Path::Real(mod_info.root.join("files").join(v.clone())),
				crate::modman::Path::Game(v) => super::Path::Game(v.clone()),
				crate::modman::Path::Option(..) => {
					let path = path.resolve_option(&mod_info.meta.borrow(), &self.settings);
					// log!("comp {path:?} {:?}", path);
					let path = path.ok_or("aaah!")?;
					super::Path::Real(mod_info.root.join("files").join(path))
				},
			};
			
			let data = super::read_file(&file_path)?;
			let tex = Tex::read(&mut std::io::Cursor::new(&data))?;
			
			let slice = tex.slice(0, 0);
			let preview = ctx.load_texture("explorer::resource::tex.layer", egui::ColorImage {
				size: [slice.width as usize, slice.height as usize],
				pixels: slice.pixels.chunks_exact(4).map(|v| egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3])).collect(),
			}, egui::TextureOptions::NEAREST);
			
			textures.insert(path.clone(), tex);
			previews.insert(path.clone(), preview);
			
			Ok(())
		};
		
		for layer in &self.composite.layers {
			add_path(&layer.path)?;
			
			for modifier in &layer.modifiers {
				match modifier {
					comp::Modifier::AlphaMask{path, ..} |
					comp::Modifier::AlphaMaskAlphaStretch{path, ..} =>
						add_path(path)?,
					
					_ => {}
				}
			}
		}
		
		self.textures = textures;
		self.textures_previews = previews;
		
		Ok(())
	}
}

fn import_file(ui: &mut egui::Ui, new_mod_file: &mut Option<(crate::ui_ext::ImporterDialog, usize, Option<usize>)>, new_game_file: &mut String, mod_info: &ModInfo, layer: usize, modifier: Option<usize>) -> Option<crate::modman::Path> {
	let mut path = None;
	
	ui.horizontal(|ui| {
		if ui.button("Mod File").clicked() {
			*new_mod_file = Some((crate::ui_ext::ImporterDialog::new("Import layer", "tex"), layer, modifier));
			path = Some(crate::modman::Path::Mod(String::new()));
			ui.close_menu();
		}
		ui.helptext("Import a new file for use within the mod");
	});
	
	ui.spacer();
	
	{
		ui.horizontal(|ui| {
			ui.label("Game File");
			ui.helptext("Select a game/other mod path to use, used for dynamically overlaying textures onto others outside the mod");
		});
		
		ui.indent("gamefile", |ui| {
			ui.text_edit_singleline(new_game_file);
			if ui.button("Add Layer").clicked() {
				path = Some(crate::modman::Path::Game(new_game_file.clone()));
			}
		});
	}
	
	ui.spacer();
	
	{
		ui.horizontal(|ui| {
			ui.label("Path Option");
			ui.helptext("Select a path option, similar to regular single file option but for a single layer");
		});
		
		ui.indent("pathoption", |ui| {
			for opt in mod_info.meta.borrow().options.iter() {
				let meta::OptionType::Option(opt) = opt else {continue};
				let meta::OptionSettings::Path(opt_path) = &opt.settings else {continue};
				
				ui.label(&opt.name);
				ui.indent(&opt.name, |ui| {
					let Some((_, subs)) = opt_path.options.get(0) else {return};
					for o in subs {
						if ui.button(&o.0).clicked() {
							path = Some(crate::modman::Path::Option(opt.name.clone(), o.0.clone()));
						}
					}
				});
			}
			
			ui.spacer();
			ui.label("Create more options in the meta tab");
		});
	}
	
	path
}

fn draw_modifier(ui: &mut egui::Ui, modifier: &mut comp::Modifier, textures_previews: &HashMap<crate::modman::Path, egui::TextureHandle>, settings: &crate::modman::settings::CollectionSettings) -> (egui::Response, bool) {
	ui.horizontal(|ui| {
		let resp = draw_modifier_preview(ui, modifier, textures_previews, settings);
		let changed = ui.vertical(|ui| {
			ui.add_space(8.0);
			ui.label(modifier.to_str());
			match modifier {
				comp::Modifier::AlphaMask{cull_point, ..} |
				comp::Modifier::AlphaMaskAlphaStretch{cull_point, ..} =>
					draw_modifier_setting(ui, cull_point, settings),
				
				comp::Modifier::Color{value} =>
					draw_modifier_setting(ui, value, settings),
			}
		}).inner;
		
		(resp, changed)
	}).inner
}

fn draw_modifier_preview(ui: &mut egui::Ui, modifier: &comp::Modifier, textures_previews: &HashMap<crate::modman::Path, egui::TextureHandle>, settings: &crate::modman::settings::CollectionSettings) -> egui::Response {
	match modifier {
		comp::Modifier::AlphaMask{path, ..} =>
			preview_modifer(ui, path, &textures_previews),
		
		comp::Modifier::AlphaMaskAlphaStretch{path, ..} =>
			preview_modifer(ui, path, &textures_previews),
		
		comp::Modifier::Color{value} => {
			let color = value.get_value(&settings).unwrap_or([0.0; 4]);
			preview_complex(
				ui,
				egui::TextureId::Managed(0),
				egui::vec2(32.0, 32.0),
				egui::vec2(32.0, 32.0),
				[egui::epaint::WHITE_UV, egui::epaint::WHITE_UV].into(),
				true,
				8,
				egui::Color32::from_rgba_unmultiplied((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8, (color[3] * 255.0) as u8)
			)
		}
	}
}

fn draw_modifier_setting<T>(ui: &mut egui::Ui, setting: &mut comp::OptionOrStatic<T>, settings: &crate::modman::settings::CollectionSettings) -> bool where
T: OptionSetting + Sized + Default + PartialEq,
<T as comp::OptionSetting>::Value: OptionOrStaticUi {
	let mut changed = false;
	
	ui.horizontal(|ui| {
		changed |= ui.combo_enum_id(setting, "setting").changed();
		changed |= match setting {
			comp::OptionOrStatic::Option(option) => {
				let mut changed = false;
				ui.combo_id(option.option_id().to_owned(), "option", |ui| {
					for (setting_name, setting_value) in settings.iter() {
						if option.get_value(setting_value).is_some() {
							changed |= ui.selectable_value(option.option_id_mut(), setting_name.to_owned(), setting_name).clicked();
						}
					}
					
					ui.spacer();
					ui.label("Create more options in the meta tab");
				});
				
				changed
			}
			
			comp::OptionOrStatic::Static(value) =>
				value.draw(ui)
		}
	});
	
	changed
}

trait OptionOrStaticUi {
	fn draw(&mut self, ui: &mut egui::Ui) -> bool;
}

impl OptionOrStaticUi for <comp::ColorOption as comp::OptionSetting>::Value {
	fn draw(&mut self, ui: &mut egui::Ui) -> bool {
		ui.color_edit(self).changed()
	}
}

impl OptionOrStaticUi for <comp::MaskOption as comp::OptionSetting>::Value {
	fn draw(&mut self, ui: &mut egui::Ui) -> bool {
		ui.slider(self, 0.0..=1.0, "").changed()
	}
}

fn preview_modifer(ui: &mut egui::Ui, path: &crate::modman::Path, textures_previews: &HashMap<crate::modman::Path, egui::TextureHandle>) -> egui::Response {
	if let Some(img) = textures_previews.get(path) {
		preview(ui, img, egui::vec2(32.0, 32.0), true, 8).on_hover_ui(|ui| {
			ui.label(path_name(&*path));
			preview(ui, img, ui.available_size(), false, 32);
		})
	} else {
		ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::all()).1.on_hover_ui(|ui| {
			ui.label(path_name(&*path));
		})
	}
}

pub (crate) fn preview(ui: &mut egui::Ui, img: &egui::TextureHandle, max_size: egui::Vec2, force_max_size: bool, grid_size: usize) -> egui::Response {
	preview_complex(ui, img.id(), img.size_vec2(), max_size, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), force_max_size, grid_size, egui::Color32::WHITE)
}

fn preview_complex(ui: &mut egui::Ui, img: egui::TextureId, img_size: egui::Vec2, max_size: egui::Vec2, uv: egui::Rect, force_max_size: bool, grid_size: usize, color: egui::Color32) -> egui::Response {
	let grid_sizef = grid_size as f32;
	let size = img_size;
	let scale = (max_size.x / size.x).min(max_size.y / size.y);
	let size = egui::vec2(size.x * scale, size.y * scale);
	let next = ui.next_widget_position();
	let offset = egui::pos2(next.x + (max_size.x - size.x) / 2.0, next.y + (max_size.y - size.y) / 2.0); 
	let rect = egui::Rect{min: offset, max: offset + size};
	
	let draw = ui.painter().with_clip_rect(rect);
	draw.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::GRAY);
	for y in 0..size.y as usize / grid_size + 1 {
		for x in (0..size.x as usize / grid_size + 1).step_by(2) {
			let offset = offset + egui::vec2(if y % 2 == 0 {x * grid_size} else {x * grid_size + grid_size} as f32, y as f32 * grid_sizef);
			draw.rect_filled(egui::Rect{min: offset, max: offset + egui::vec2(grid_sizef, grid_sizef)}, egui::CornerRadius::ZERO, egui::Color32::DARK_GRAY);
		}
	}
	
	let draw = ui.painter();
	draw.image(img, rect, uv, color);
	ui.allocate_rect(if force_max_size {egui::Rect::from_min_size(next, max_size)} else {rect}, egui::Sense::all())
}

fn path_name(path: &crate::modman::Path) -> String {
	match path {
		crate::modman::Path::Mod(v) =>
			format!("Mod File: {v}"),
		
		crate::modman::Path::Game(v) =>
			format!("Game File: {v}"),
		
		crate::modman::Path::Option(option, id) =>
			format!("Path Option: {option}/{id}"),
	}
}