use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc, sync::Arc};
use egui::Widget;
use noumenon::format::external::Bytes;
use crate::{modman::meta, ui_ext::{ImporterDialog, UiExt}, EnumTools};

#[derive(Clone)]
enum PackStatus {
	None,
	Busy(String),
	Success(String),
	Failure(String),
}

pub struct Workspace {
	root: PathBuf,
	meta: Rc<RefCell<meta::Meta>>,
	changed: bool,
	textures: HashMap<meta::ValuePathPath, Option<egui::TextureHandle>>,
	importer: Option<(ImporterDialog, usize, usize, usize)>,
	
	status: Arc<std::sync::RwLock<PackStatus>>,
}

impl Workspace {
	pub fn new(meta: Rc<RefCell<meta::Meta>>, root: PathBuf) -> Workspace {
		Self {
			root,
			meta,
			changed: false,
			textures: HashMap::new(),
			importer: None,
			
			status: Arc::new(std::sync::RwLock::new(PackStatus::None)),
		}
	}
	
	fn create_pack(&self) {
		let root = self.root.clone();
		let status = self.status.clone();
		
		*status.write().unwrap() = PackStatus::Busy("Creating Modpack".to_string());
		
		std::thread::spawn(move|| {
			match crate::modman::create_mod(&root, crate::modman::ModCreationSettings {
				current_game_files_hash: true,
			}) {
				Ok(path) => *status.write().unwrap() = PackStatus::Success(format!("Created modpack at {path:?}")),
				Err(err) => *status.write().unwrap() = PackStatus::Failure(format!("Failed creating modpack\n\n{err:?}")),
			};
		});
	}
}

impl super::ExplorerView for Workspace {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
	
	fn title(&self) -> String {
		format!("Workspace - {}{}", self.meta.borrow().name, if self.changed {" *"} else {""})
	}
	
	fn path(&self) -> Option<&super::resource::Path> {
		None
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) -> super::Action {
		let status = self.status.read().unwrap();
		match &*status {
			PackStatus::Busy(msg) => {
				ui.label(msg);
				return super::Action::None;
			}
			
			PackStatus::Success(msg) |
			PackStatus::Failure(msg) => {
				ui.label(msg);
				drop(status);
				
				if ui.button("Ok").clicked() {
					*self.status.write().unwrap() = PackStatus::None;
				}
				
				return super::Action::None;
			}
			
			PackStatus::None => {}
		}
		drop(status);
		
		let mut meta = self.meta.borrow_mut();
		let mut changed = false;
		
		ui.label("Name");
		changed |= ui.text_edit_singleline(&mut meta.name).changed();
		ui.spacer();
		
		ui.label("Description");
		changed |= ui.text_edit_multiline(&mut meta.description).changed();
		ui.spacer();
		
		ui.label("Version");
		changed |= ui.text_edit_singleline(&mut meta.version).changed();
		ui.spacer();
		
		ui.label("Author");
		changed |= ui.text_edit_singleline(&mut meta.author).changed();
		ui.spacer();
		
		ui.label("Website");
		changed |= ui.text_edit_singleline(&mut meta.website).changed();
		ui.spacer();
		
		ui.label("Tags");
		ui.indent("tags", |ui| {
			let mut delete = None;
			for (i, tag) in meta.tags.iter_mut().enumerate() {
				ui.horizontal(|ui| {
					changed |= ui.text_edit_singleline(tag).changed();
					if ui.button("🗑").clicked() {
						delete = Some(i);
					}
				});
			}
			
			if let Some(i) = delete {
				meta.tags.remove(i);
				changed = true;
			}
			
			if ui.button("➕ Add tag").clicked() {
				meta.tags.push(String::new());
			}
		});
		ui.spacer();
		
		// ui.label("Dependencies");
		// {
		// 	let mut delete = None;
		// 	for (i, tag) in meta.dependencies.iter_mut().enumerate() {
		// 		ui.horizontal(|ui| {
		// 			changed |= ui.text_edit_singleline(tag).changed();
		// 			if ui.button("🗑").clicked() {
		// 				delete = Some(i);
		// 			}
		// 		});
		// 	}
		// 	
		// 	if let Some(i) = delete {
		// 		meta.dependencies.remove(i);
		// 		changed = true;
		// 	}
		// 	
		// 	if ui.button("➕ Add dependency").clicked() {
		// 		meta.dependencies.push(String::new());
		// 	}
		// }
		// ui.spacer();
		
		ui.label("Options");
		ui.indent("options", |ui| {
			let mut delete = None;
			egui_dnd::dnd(ui, "options").show_vec(&mut meta.options, |ui, option, handle, state| {
				match option {
					meta::OptionType::Category(category) => {
						handle.ui(ui, |ui| {
							ui.label("Category Tab").context_menu(|ui| {
								if ui.button("🗑 Delete Option").clicked() {
									delete = Some(state.index);
									ui.close_menu();
								}
							});
						});
						
						ui.indent(state.index, |ui| {
							changed |= ui.text_edit_singleline(category).changed();
						});
					}
					
					meta::OptionType::Option(opt) => {
						handle.ui(ui, |ui| {
							ui.label(opt.settings.to_str()).context_menu(|ui| {
								if ui.button("🗑 Delete Option").clicked() {
									delete = Some(state.index);
									ui.close_menu();
								}
							});
						});
						
						ui.indent(state.index, |ui| {
							ui.horizontal(|ui| {
								changed |= ui.text_edit_singleline(&mut opt.name).changed();
								ui.label("Name");
							});
							
							ui.horizontal(|ui| {
								changed |= egui::TextEdit::multiline(&mut opt.description)
									.desired_rows(1)
									.ui(ui)
									.changed();
								ui.label("Description");
							});
							
							match &mut opt.settings {
								meta::OptionSettings::Grouped(val) => {
									ui.label("todo Grouped");
								}
								
								meta::OptionSettings::SingleFiles(val) => {
									changed |= file_option(ui, val, false);
								}
								
								meta::OptionSettings::MultiFiles(val) => {
									changed |= file_option(ui, val, true);
								}
								
								meta::OptionSettings::Rgb(val) => {
									changed |= ui.num_multi_edit_range(&mut val.min, "Min", &[const{0.0..=1.0}; 3]).changed();
									changed |= ui.num_multi_edit_range(&mut val.max, "Max", &[const{0.0..=1.0}; 3]).changed();
									ui.horizontal(|ui| {
										changed |= ui.color_edit(&mut val.default).changed();
										ui.label("Default");
									});
								}
								
								meta::OptionSettings::Rgba(val) => {
									changed |= ui.num_multi_edit_range(&mut val.min, "Min", &[const{0.0..=1.0}; 4]).changed();
									changed |= ui.num_multi_edit_range(&mut val.max, "Max", &[const{0.0..=1.0}; 4]).changed();
									ui.horizontal(|ui| {
										changed |= ui.color_edit(&mut val.default).changed();
										ui.label("Default");
									});
								}
								
								meta::OptionSettings::Grayscale(val) |
								meta::OptionSettings::Opacity(val) |
								meta::OptionSettings::Mask(val) => {
									changed |= ui.num_edit_range(&mut val.min, "Min", 0.0..=1.0).changed();
									changed |= ui.num_edit_range(&mut val.max, "Max", 0.0..=1.0).changed();
									changed |= ui.slider(&mut val.default, val.min..=val.max, "Default").changed();
								}
								
								meta::OptionSettings::Path(val) => {
									let mut delete = None;
									for i in 0..val.options.len() {
										if i > 0 {
											ui.spacer();
										}
										
										ui.horizontal(|ui| {
											if ui.button("🗑").clicked() {
												delete = Some(i);
											}
											
											if ui.checkbox(&mut (val.default as usize == i), "")
												.on_hover_text("Default")
												.clicked() {
												val.default = i as u32;
												changed = true;
											}
											
											changed |= ui.text_edit_singleline(&mut val.options[i].0).changed();
										});
										
										ui.indent(i, |ui| {
											let mut delete = None;
											for j in 0..val.options[i].1.len() {
												ui.horizontal(|ui| {
													// path options arent technically texture exclusive,
													// but its the only place they are used so should be fine for now
													if if let Some(img) = get_texture(ui.ctx(), val.options[i].1[j].1.clone(), &self.root, &mut self.textures) {
														super::resource::tex::preview(ui, img, egui::vec2(32.0, 32.0), true, 8).on_hover_ui(|ui| {
															super::resource::tex::preview(ui, img, ui.available_size(), false, 32);
														}).on_hover_text("Change texture")
													} else {
														let (rect, resp) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::all());
														let painter = ui.painter();
														painter.text(
															rect.center(),
															egui::Align2::CENTER_CENTER,
															"➕",
															ui.style().text_styles[&egui::TextStyle::Heading].clone(),
															ui.style().visuals.text_color(),
														);
														
														resp.on_hover_text("Set texture")
													}.clicked() {
														self.importer = Some((
															ImporterDialog::new(format!("Import texture for path option '{}/{}/{}'", opt.name, val.options[i].0, val.options[i].1[j].0), "tex"),
															state.index,
															i,
															j
														));
													}
													
													if ui.button("🗑").clicked() {
														delete = Some(j);
													}
													
													let id = &mut val.options[i].1[j].0;
													if ui.text_edit_singleline(id).changed() {
														let id = id.clone();
														for k in 0..val.options.len() {
															if i == k {continue}
															val.options[k].1[j].0 = id.clone();
														}
														
														changed = true;
													}
												});
											}
											
											if let Some(i) = delete {
												for (_, paths) in &mut val.options {
													paths.remove(i);
												}
												
												changed = true;
											}
											
											if ui.button("➕ Add path").clicked() {
												for (_, paths) in &mut val.options {
													paths.push(("id".to_string(), meta::ValuePathPath::Mod(String::new())));
												}
												
												changed = true;
											}
										});
									}
									
									if let Some(i) = delete {
										val.options.remove(i);
										changed = true;
									}
									
									if ui.button("➕ Add sub option").clicked() {
										let mut paths = Vec::new();
										if val.options.len() > 0 {
											for (id, _) in &val.options[0].1 {
												paths.push((id.clone(), meta::ValuePathPath::Mod(String::new())));
											}
										}
										
										val.options.push((
											"New sub option".to_string(),
											paths
										));
										
										changed = true;
									}
								}
							}
						});
					}
				}
				
				ui.spacer();
			});
			
			if let Some(i) = delete {
				meta.options.remove(i);
				changed = true;
			}
			
			ui.combo_id("Add New", "addnew", |ui| {
				if ui.button("Category Tab").clicked() {
					meta.options.push(meta::OptionType::Category("New Category".to_string()));
					changed = true;
				}
				
				ui.spacer();
				
				for opt in meta::OptionSettings::iter() {
					if ui.button(opt.to_str()).clicked() {
						meta.options.push(meta::OptionType::Option(meta::Option {
							name: "New Option".to_string(),
							description: String::new(),
							settings: opt,
						}));
						changed = true;
					}
				}
			});
		});
		ui.spacer();
		
		if ui.button("Create modpack").clicked() {
			self.create_pack();
		}
		
		if let Some((dialog, option_id, suboption_id, path_id)) = &mut self.importer {
			match dialog.show(ui) {
				Ok(crate::ui_ext::DialogResult::Success(data)) => 'o: {
					// let Some(layer) = comp.composite.layers.get_mut(*layer_id) else {break 'o};
					// let Some(mod_info) = &self.mod_info else {break 'o};
					let Some(meta::OptionType::Option(option)) = meta.options.get_mut(*option_id) else {break 'o};
					let meta::OptionSettings::Path(option) = &mut option.settings else {break 'o};
					let Some((_, suboption)) = option.options.get_mut(*suboption_id) else {break 'o};
					let Some((_, path)) = suboption.get_mut(*path_id) else {break 'o};
					let hash = crate::hash_str(blake3::hash(&data));
					let path_rel = format!("{hash}.tex");
					
					let file_path = self.root.join("files").join(&path_rel);
					_ = std::fs::create_dir_all(file_path.parent().unwrap());
					
					if let Err(err) = std::fs::write(file_path, data) {
						log!(err, "Failed writing file ({err:?})");
						self.importer = None;
						break 'o;
					}
					
					*path = meta::ValuePathPath::Mod(path_rel);
					
					changed = true;
					self.importer = None;
				}
				
				Ok(crate::ui_ext::DialogResult::Cancelled) =>
					self.importer = None,
				
				Ok(crate::ui_ext::DialogResult::Busy) => {}
				
				Err(err) => {
					log!(err, "failed importing file {err:?}");
					self.importer = None;
				}
			}
		}
		
		self.changed |= changed;
		
		if ui.interact(ui.max_rect(), egui::Id::new("keybinds"), egui::Sense::hover()).hovered() {
			let ctx = ui.ctx();
			if self.changed && ctx.input_mut(|v| v.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
				_ =  meta.save(&self.root.join("meta.json"));
				self.changed = false;
			}
		}
		
		super::Action::None
	}
}

fn file_option(ui: &mut egui::Ui, val: &mut meta::ValueFiles, multi: bool) -> bool {
	let mut changed = false;
	let mut delete = None;
	for i in 0..val.options.len() {
		if i > 0 {
			ui.spacer();
		}
		
		ui.horizontal(|ui| {
			if ui.button("🗑").clicked() {
				delete = Some(i);
			}
			
			if multi {
				if ui.checkbox(&mut (val.default >> i & 1 == 1), "")
					.on_hover_text("Default")
					.clicked() {
					val.default ^= 1 << i;
					changed = true;
				}
			} else {
				if ui.checkbox(&mut (val.default as usize == i), "")
					.on_hover_text("Default")
					.clicked() {
					val.default = i as u32;
					changed = true;
				}
			}
			
			changed |= ui.text_edit_singleline(&mut val.options[i].name).changed();
		});
		
		ui.indent(i, |ui| {
			ui.horizontal(|ui| {
				changed |= egui::TextEdit::multiline(&mut val.options[i].description)
					.desired_rows(1)
					.ui(ui)
					.changed();
				ui.label("Description");
			});
			
			if !multi {
				ui.combo(val.options[i].inherit.clone().unwrap_or("None".to_string()), "Inherit", |ui| {
					if ui.selectable_label(val.options[i].inherit.is_none(), "None").clicked() {
						val.options[i].inherit = None;
						changed = true;
					}
					
					for j in 0..val.options.len() {
						if j == i {continue}
						let newval = Some(val.options[j].name.clone());
						if ui.selectable_label(val.options[i].inherit == newval, &val.options[j].name).clicked() {
							val.options[i].inherit = newval;
							changed = true;
						}
					}
				});
			}
		});
	}
	
	if let Some(i) = delete {
		val.options.remove(i);
		changed = true;
	}
	
	if ui.button("➕ Add sub option").clicked() {
		val.options.push(meta::ValueFilesOption::default());
		changed = true;
	}
	
	changed
}

fn get_texture<'a>(ctx: &egui::Context, path: meta::ValuePathPath, root: &std::path::Path, textures: &'a mut HashMap<meta::ValuePathPath, Option<egui::TextureHandle>>) -> Option<&'a egui::TextureHandle> {
	textures.entry(path.clone()).or_insert_with(|| {
		match path {
			meta::ValuePathPath::Mod(path) => {
				let path = root.join("files").join(path);
				let Ok(file) = std::fs::File::open(path) else {return None};
				let Ok(tex) = noumenon::format::game::Tex::read(&mut std::io::BufReader::new(file)) else {return None};
				
				let slice = tex.slice(0, 0);
				let img = ctx.load_texture("explorer::workshop.optionpath", egui::ColorImage {
					size: [slice.width as usize, slice.height as usize],
					pixels: slice.pixels.chunks_exact(4).map(|v| egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3])).collect(),
				}, egui::TextureOptions::NEAREST);
				
				Some(img)
			}
		}
	}).as_ref()
}