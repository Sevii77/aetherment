use std::collections::HashMap;
use crate::modman::meta;
use crate::modman::composite::tex as comp;
use crate::render_helper::UiExt;

struct Layer {
	color_option: Option<(meta::ValueRgba, String)>,
	blend_mode: comp::Blend,
	files: Vec<File>,
}

impl Default for Layer {
	fn default() -> Self {
		Self {
			color_option: Some((Default::default(), "Color Option".to_string())),
			blend_mode: comp::Blend::Normal,
			files: Vec::new(),
		}
	}
}

struct File {
	name: &'static str,
	paths: Vec<String>,
	file: FileData,
}

enum FileData {
	Data(Vec<u8>),
	Importing(renderer::FilePicker),
}

pub struct Overlay {
	meta: meta::Meta,
	layers: Vec<Layer>,
	export_dialog: Option<renderer::FilePicker>,
}

impl Overlay {
	pub fn new() -> Self {
		Self {
			meta: Default::default(),
			layers: vec![Default::default()],
			export_dialog: None,
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		let mut dialog_open = false;
		
		// export
		if let Some(picker) = &mut self.export_dialog {
			let conf = crate::config();
			
			match picker.show(ui) {
				renderer::FilePickerStatus::Success(dir, paths) => {
					conf.config.file_dialog_path = dir;
					_ = conf.save_forced();
					self.export_dialog = None;
					
					if let Err(err) = create_mod(&paths[0].with_extension("aeth"), &self.meta, &self.layers) {
						log!(err, "Failed creating mod ({err:?})");
					}
				}
				
				renderer::FilePickerStatus::Canceled(dir) => {
					conf.config.file_dialog_path = dir;
					_ = conf.save_forced();
					self.export_dialog = None;
				}
				
				_ => {}
			}
		}
		
		// layer file import
		'outer: for layer in self.layers.iter_mut() {
			for (i, file) in layer.files.iter_mut().enumerate() {
				if let FileData::Importing(picker) = &mut file.file {
					dialog_open = true;
					
					let conf = crate::config();
					match picker.show(ui) {
						renderer::FilePickerStatus::Success(dir, paths) => {
							conf.config.file_dialog_path = dir;
							_ = conf.save_forced();
							
							let f = match std::fs::File::open(&paths[0]) {
								Ok(v) => v,
								Err(err) => {
									log!(err, "Failed importing file ({err:?})");
									layer.files.remove(i);
									break 'outer;
								}
							};
							
							let Some(ext) = &paths[0].extension().map(|v| v.to_string_lossy().to_string()) else {
								log!(err, "Failed importing file (Has no extension)");
								layer.files.remove(i);
								break 'outer;
							};
							
							let converter = match noumenon::Convert::from_ext(ext, &mut std::io::BufReader::new(f)) {
								Ok(v) => v,
								Err(err) => {
									log!(err, "Failed importing file ({err:?})");
									layer.files.remove(i);
									break 'outer;
								}
							};
							
							let mut buf = Vec::new();
							if let Err(err) = converter.convert("tex", &mut std::io::Cursor::new(&mut buf)) {
								log!(err, "Failed importing file ({err:?})");
								layer.files.remove(i);
								break 'outer;
							}
							
							file.file = FileData::Data(buf);
						}
						
						renderer::FilePickerStatus::Canceled(dir) => {
							conf.config.file_dialog_path = dir;
							_ = conf.save_forced();
							layer.files.remove(i);
							break 'outer;
						}
						
						_ => {}
					}
					
					break 'outer;
				}
			}
		}
		
		//
		let meta = &mut self.meta;
		
		ui.input_text("Name", &mut meta.name);
		ui.input_text("Version", &mut meta.version);
		ui.input_text("Author", &mut meta.author);
		ui.input_text("Website", &mut meta.website);
		ui.input_text_multiline("Description", &mut meta.description);
		
		ui.add_space(16.0);
		
		let mut delete = None;
		for (i, layer) in self.layers.iter_mut().enumerate() {
			ui.collapsing_header(format!("Layer {i}"), |ui| {
				ui.push_id("coloroption", |ui| {
					ui.horizontal(|ui| {
						let mut c = layer.color_option.is_some();
						ui.push_id("checkbox", |ui| {ui.checkbox("", &mut c);});
						if let Some(color) = &mut layer.color_option {
							ui.color_edit_rgba("", &mut color.0.default);
							ui.input_text("", &mut color.1);
						} else {
							ui.label("Color Option");
						}
						
						if c && layer.color_option.is_none() {
							layer.color_option = Some((Default::default(), "Color Option".to_string()));
						}
						if !c && layer.color_option.is_some() {
							layer.color_option = None;
						}
					});
				});
				
				ui.combo_enum("Blend mode", &mut layer.blend_mode);
				ui.add_space(16.0);
				
				{
					let mut delete = None;
					for (i, file) in layer.files.iter_mut().enumerate() {
						ui.collapsing_header(format!("File {i}"), |ui| {
							if !file.name.is_empty() {
								ui.label(file.name)
							}
							
							{
								let mut delete = None;
								for (i, path) in file.paths.iter_mut().enumerate() {
									ui.horizontal(|ui| {
										ui.push_id(format!("path{i}"), |ui| {
											if ui.button("Delete").clicked {
												delete = Some(i);
											}
											
											ui.input_text("", path);
										});
									});
								}
								
								if let Some(delete) = delete {
									file.paths.remove(delete);
								}
								
								if ui.button("Add new path").clicked {
									file.paths.push(String::new());
								}
							}
							
							ui.add_space(16.0);
							if ui.button("Delete File").clicked {
								delete = Some(i);
							}
						});
					}
					
					if let Some(delete) = delete {
						layer.files.remove(delete);
					}
					
					ui.enabled(!dialog_open, |ui| {
						ui.combo("Add", "Add new", |ui| {
							if ui.button("New Empty").clicked {
								layer.files.push(File {
									name: "",
									paths: Vec::new(),
									file: FileData::Importing(renderer::FilePicker::new("Importing file for new layer", &crate::config().config.file_dialog_path, &[".png", ".tif", ".tiff", ".tga", ".dds", ".tex"], renderer::FilePickerMode::OpenFile)),
								})
							}
							
							for (category, presets) in PRESETS {
								ui.collapsing_header(category, |ui| {
									for (name, paths) in *presets {
										if ui.button(name).clicked {
											layer.files.push(File {
												name,
												paths: paths.iter().map(|v| v.to_string()).collect(),
												file: FileData::Importing(renderer::FilePicker::new(&format!("Importing file for new layer {name}"), &crate::config().config.file_dialog_path, &[".png", ".tif", ".tiff", ".tga", ".dds", ".tex"], renderer::FilePickerMode::OpenFile)),
											})
										}
									}
								})
							}
						})
					});
				}
				
				ui.add_space(16.0);
				if ui.button("Delete layer").clicked {
					delete = Some(i);
				}
			});
		}
		
		if let Some(delete) = delete {
			self.layers.remove(delete);
		}
		
		if ui.button("Add new layer").clicked {
			self.layers.push(Default::default());
		}
		
		ui.add_space(16.0);
		
		ui.enabled(!dialog_open, |ui| {
			if ui.button("Export").clicked {
				self.export_dialog = Some(renderer::FilePicker::new(&format!("Export mod {}", self.meta.name), &crate::config().config.file_dialog_path, &[".aeth"], renderer::FilePickerMode::Save));
			}
			
			if ui.button("Reset").clicked {
				*self = Self::new();
			}
		});
	}
}

fn create_mod(path: &std::path::Path, meta: &meta::Meta, layers: &[Layer]) -> Result<(), crate::resource_loader::BacktraceError> {
	let mut meta = meta.clone();
	let mut modpack = crate::modman::ModPack::new(std::io::BufWriter::new(std::fs::File::create(path)?), crate::modman::ModCreationSettings {
		current_game_files_hash: true,
	});
	
	let mut file_layers = HashMap::new();
	let mut color_option_counts = HashMap::new();
	for (i, layer) in layers.iter().enumerate() {
		let color = if let Some((color, name)) = &layer.color_option {
			let counts = color_option_counts.entry(name.to_owned()).or_insert(0);
			let name = if *counts == 0 {name.to_owned()} else {format!("{name} {counts}")};
			
			meta.options.0.push(meta::OptionType::Option(meta::Option {
				name: name.clone(),
				description: String::new(),
				settings: meta::OptionSettings::Rgba(color.to_owned()),
			}));
			
			*counts = *counts + 1;
			Some(name)
		} else {
			None
		};
		
		for file in &layer.files {
			let FileData::Data(data) = &file.file else {continue};
			let hash = crate::hash_str(blake3::hash(&data));
			modpack.add_file(&hash, data)?;
			
			for path in &file.paths {
				file_layers.entry(path).or_insert_with(|| vec![comp::Layer {
					name: "Base".to_string(),
					path: crate::modman::Path::Game(path.to_owned()),
					modifiers: Vec::new(),
					blend: comp::Blend::Normal,
				}]).push(comp::Layer {
					name: format!("Layer {i}"),
					path: crate::modman::Path::Mod(hash.clone()),
					modifiers: if let Some(color) = &color {vec![comp::Modifier::Color{value: comp::OptionOrStatic::Option(comp::ColorOption(color.to_owned()))}]} else {Vec::new()},
					blend: layer.blend_mode.clone(),
				});
			}
		}
	}
	
	for (path, mut layers) in file_layers {
		layers.reverse();
		let comp = comp::Tex{layers};
		
		let comp_data = serde_json::to_vec(&comp)?;
		let comp_hash = crate::hash_str(blake3::hash(&comp_data));
		
		modpack.add_file(&comp_hash, &comp_data)?;
		meta.files.insert(format!("{path}.comp"), comp_hash);
	}
	
	modpack.add_meta(&meta)?;
	modpack.finalize()?;
	
	Ok(())
}

const PRESETS: &[(&str, &[(&str, &[&str])])] = &[
	("Body Diffuse", &[
		("Vanilla Tall Female", &[
			"chara/human/c0201/obj/body/b0001/texture/c0201b0001_base.tex", // midlander (elezen, miqote, roe)
			"chara/human/c0401/obj/body/b0001/texture/c0401b0001_base.tex", // highlander
			"chara/human/c1401/obj/body/b0001/texture/c1401b0001_base.tex", // aura raen
			"chara/human/c1401/obj/body/b0101/texture/c1401b0101_base.tex", // aura xaela
			"chara/human/c1601/obj/body/b0001/texture/c1601b0001_base.tex", // hrothgar
			"chara/human/c1801/obj/body/b0001/texture/c1801b0001_base.tex", // viera
		]),
		
		("Bibo", &[
			"chara/bibo_mid_base.tex",
			"chara/bibo_high_base.tex",
			"chara/bibo_raen_base.tex",
			"chara/bibo_xaela_base.tex",
			"chara/bibo_hroth_base.tex",
			"chara/bibo_viera_base.tex",
		]),
		
		("TF Gen3", &[ // bibo compatability files atleast
			"chara/tfgen3midf_d.tex",
			"chara/tfgen3highf_d.tex",
			"chara/tfgen3raenf_d.tex",
			"chara/tfgen3xaelaf_d.tex",
			"chara/tfgen3hrothf_d.tex",
			"chara/tfgen3viera_d.tex",
		]),
		
		("Vanilla Tall Male", &[
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_base.tex", // midlander (elezen, miqote)
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_base.tex", // highlander
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_base.tex", // aura raen
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_base.tex", // aura xaela
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_base.tex", // viera
		]),
		
		("TBSE", &[
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_b_d.tex", // midlander (elezen, miqote)
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_b_d.tex", // highlander
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_b_d.tex", // aura raen
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_b_d.tex", // aura xaela
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_b_d.tex", // viera
		]),
		
		("Vanilla Big Male", &[
			"chara/human/c0901/obj/body/b0001/texture/c0901b0001_base.tex", // roe
			"chara/human/c1501/obj/body/b0001/texture/c1501b0001_base.tex", // hrothgar
		]),
		
		// TODO: HR3 once it gets updated for dt
		
		("Vanilla Lalafell", &[
			"chara/human/c1101/obj/body/b0001/texture/c1101b0001_base.tex", // male
			// "chara/human/c1201/obj/body/b0001/texture/c1201b0001_base.tex", // female, uses male texture
		]),
		
		// TODO: otopop once it gets updated for dt
	]),
	
	("Body Normal", &[
		("Vanilla Tall Female", &[
			"chara/human/c0201/obj/body/b0001/texture/c0201b0001_norm.tex", // midlander (elezen, miqote, roe)
			"chara/human/c0401/obj/body/b0001/texture/c0401b0001_norm.tex", // highlander
			"chara/human/c1401/obj/body/b0001/texture/c1401b0001_norm.tex", // aura raen
			"chara/human/c1401/obj/body/b0101/texture/c1401b0101_norm.tex", // aura xaela
			"chara/human/c1601/obj/body/b0001/texture/c1601b0001_norm.tex", // hrothgar
			"chara/human/c1801/obj/body/b0001/texture/c1801b0001_norm.tex", // viera
		]),
		
		("Bibo", &[
			"chara/bibo_mid_norm.tex",
			"chara/bibo_high_norm.tex",
			"chara/bibo_raen_norm.tex",
			"chara/bibo_xaela_norm.tex",
			"chara/bibo_hroth_norm.tex",
			"chara/bibo_viera_norm.tex",
		]),
		
		("TF Gen3", &[ // bibo compatability files atleast
			"chara/tfgen3midf_n.tex",
			"chara/tfgen3highf_n.tex",
			"chara/tfgen3raenf_n.tex",
			"chara/tfgen3xaelaf_n.tex",
			"chara/tfgen3hrothf_n.tex",
			"chara/tfgen3viera_n.tex",
		]),
		
		("Vanilla Tall Male", &[
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_norm.tex", // midlander (elezen, miqote)
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_norm.tex", // highlander
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_norm.tex", // aura raen
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_norm.tex", // aura xaela
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_norm.tex", // viera
		]),
		
		("TBSE", &[
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_b_n.tex", // midlander (elezen, miqote)
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_b_n.tex", // highlander
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_b_n.tex", // aura raen
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_b_n.tex", // aura xaela
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_b_n.tex", // viera
		]),
		
		("Vanilla Big Male", &[
			"chara/human/c0901/obj/body/b0001/texture/c0901b0001_norm.tex", // roe
			"chara/human/c1501/obj/body/b0001/texture/c1501b0001_norm.tex", // hrothgar
		]),
		
		// TODO: HR3 once it gets updated for dt
		
		("Vanilla Lalafell", &[
			"chara/human/c1101/obj/body/b0001/texture/c1101b0001_norm.tex", // male
			// "chara/human/c1201/obj/body/b0001/texture/c1201b0001_base.tex", // female, uses male texture
		]),
		
		// TODO: otopop once it gets updated for dt
	]),
];