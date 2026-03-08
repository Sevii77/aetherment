use std::collections::HashMap;
use noumenon::format::{external::Bytes, game::Tex};
use crate::{EnumTools, modman::{Path, composite::tex::{self as comp, Tex as Comp}, meta}, ui_ext::UiExt};

struct TFile {
	tex: Tex,
	img: egui::TextureHandle,
	hash: String,
	paths: Vec<String>,
}

impl TFile {
	fn new(tex: Tex, ctx: &egui::Context, hash: String, paths: Vec<String>) -> Self {
		use image::EncodableLayout;
		
		let (w, h) = (tex.width, tex.height);
		let img: image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::from_vec(w as u32, h as u32, tex.slice(0, 0).pixels.to_vec()).unwrap();
		let scale = (512.0 / w as f32).min(512.0 / h as f32);
		let (w2, h2) = ((w as f32 * scale) as u32, (h as f32 * scale) as u32);
		let img = image::imageops::resize(&img, w2, h2, image::imageops::FilterType::Nearest);
		
		Self {
			img: ctx.load_texture("tool::tattoo", egui::epaint::image::ColorImage::from_rgba_unmultiplied([w2 as usize, h2 as usize], img.as_bytes()), Default::default()),
			tex,
			hash,
			paths,
		}
	}
}

struct Layer {
	// color: Option<(String, [f32; 4])>,
	color: Option<String>,
	blend_mode: comp::Blend,
	textures: Vec<(String, TFile)>,
}

pub struct Tattoo {
	progress: crate::modman::backend::TaskProgress,
	
	meta: meta::Meta,
	colors: HashMap<String, [f32; 4]>,
	layers: Vec<Layer>,
	
	add_race: Race,
	add_sex: bool,
	add_face: usize,
	add_custom_path: String,
	
	import: Option<(usize, String, Vec<String>, egui_file::FileDialog)>,
	export: Option<egui_file::FileDialog>,
}

impl Tattoo {
	pub fn new(progress: crate::modman::backend::TaskProgress) -> Self {
		Self {
			progress,
			
			meta: Default::default(),
			colors: HashMap::new(),
			layers: vec![
				Layer {
					color: None,
					blend_mode: comp::Blend::Normal,
					textures: Vec::new(),
				}
			],
			
			add_race: Race::Midlander,
			add_sex: true,
			add_face: 1,
			add_custom_path: String::new(),
			
			import: None,
			export: None,
		}
	}
	
	pub fn ui_creator(&mut self, ui: &mut egui::Ui, start_path: Option<std::path::PathBuf>) {
		let meta = &mut self.meta;
		ui.horizontal(|ui| {
			ui.label("What is this");
			ui.helptext("\
This tool allows you to easily create tattoo mods from transparent files.
Simply click the button for the body type (you add files for multiple
body types) and select the transparent file. Afterwards export the mod.
It can now be imported using the \"Import Mods\" button on the Mods tab.
Afterwards set the priority above your body mods and Aetherment will
automatically overlay the tattoo on your body. Changing your body texture
will be reflected after hitting apply, and you can stack multiple overlay
mods ontop of one another (multiple tattoos for example).");
		});
		
		ui.add_space(10.0);
		
		ui.label("Name");
		ui.text_edit_singleline(&mut meta.name);
		ui.add_space(10.0);
		
		ui.label("Description");
		ui.text_edit_multiline(&mut meta.description);
		ui.add_space(10.0);
		
		ui.label("Version");
		ui.text_edit_singleline(&mut meta.version);
		ui.add_space(10.0);
		
		ui.label("Author");
		ui.text_edit_singleline(&mut meta.author);
		ui.add_space(10.0);
		
		ui.label("Website");
		ui.text_edit_singleline(&mut meta.website);
		ui.add_space(10.0);
		
		ui.label(format!("Layers"));
		// let mut delete = None;
		ui.indent("layers", |ui| {
			let mut import = None;
			
			for (i, layer) in self.layers.iter_mut().enumerate() {
				ui.label(format!("Layer {i}"));
				ui.indent(i, |ui| {
					ui.horizontal(|ui| {
						ui.label("Color edit option");
						
						let mut checked = layer.color.is_some();
						ui.add(egui::Checkbox::without_text(&mut checked));
						
						if let Some(name) = &mut layer.color {
						// if let Some((name, color)) = &mut layer.color {
							let color = self.colors.entry(name.clone()).or_insert([0.0, 0.0, 0.0, 1.0]);
							ui.text_edit_singleline(name);
							ui.color_edit(color);
							ui.label("Default");
							
							if !checked {
								layer.color = None;
							}
						} else if checked {
							layer.color = Some("Color".to_string());
						}
						
						if !checked {
							ui.label("Disabled");
						}
					});
					
					ui.horizontal(|ui| {
						ui.label("Blend mode");
						ui.combo_enum_id(&mut layer.blend_mode, "blendmode");
					});
					
					ui.horizontal(|ui| {
						ui.label("Textures");
						
						for texture in layer.textures.iter_mut() {
							ui.button(&texture.0).on_hover_ui_at_pointer(|ui| {
								let w = texture.1.tex.width as f32;
								let h = texture.1.tex.height as f32;
								let w = 400.0 * (w / h).min(1.0);
								let h = 400.0 * (h / w).min(1.0);
								ui.add(egui::Image::new(&texture.1.img).max_size(egui::vec2(w, h)));
							});
						}
						
						ui.menu_button("➕", |ui| {
							// let config = crate::config();
							// let body_presets = config.config.tool_tattoo_presets.clone().unwrap_or_else(||
							// 	BODY_PRESETS
							// 		.into_iter()
							// 		.map(|v| (v.0, v.1
							// 			.into_iter()
							// 			.map(|v| (v.0.to_string(), v.1
							// 				.into_iter()
							// 				.map(|v| v.to_string())
							// 				.collect::<Vec<_>>())
							// 			).collect::<Vec<_>>())
							// 		).collect::<Vec<_>>());
							let body_presets = BODY_PRESETS
								.into_iter()
								.map(|v| (v.0, v.1
									.into_iter()
									.map(|v| (v.0.to_string(), v.1
										.into_iter()
										.map(|v| v.to_string())
										.collect::<Vec<_>>())
									).collect::<Vec<_>>())
								).collect::<Vec<_>>();
							
							for (typ, uvs) in &body_presets {
								ui.menu_button(format!("Body {}", typ.name()), |ui| {
									for (uv, paths) in uvs {
										if ui.button(uv).clicked() {
											import = Some((i, format!("Body {} - {uv}", typ.name()), paths.clone()));
											ui.close_menu();
										}
									}
								});
							}
							
							ui.add_space(10.0);
							for (typ, _) in &body_presets {
								ui.menu_button(format!("Face {}", typ.name()), |ui| {
									let race_str = self.add_race.sexname(self.add_sex);
									ui.menu_button(format!("Race ({race_str})"), |ui| {
										for race in Race::iter() {
											if ui.selectable_label(self.add_race == race && !self.add_sex, race.sexname(false)).clicked() {
												self.add_race = race;
												self.add_sex = false;
											}
											
											if ui.selectable_label(self.add_race == race && self.add_sex, race.sexname(true)).clicked() {
												self.add_race = race;
												self.add_sex = true;
											}
										}
									});
									
									ui.num_edit_range(&mut self.add_face, "Face", 1..=10);
									
									if ui.button("Select").clicked() {
										let path = racesex_to_facepath(self.add_race, self.add_sex, self.add_face, *typ);
										import = Some((i, format!("Face {} - {race_str} Face {}", typ.name(), self.add_face), vec![path]));
										ui.close_menu();
									}
								});
							}
							
							ui.add_space(10.0);
							ui.label("Custom Path");
							ui.horizontal(|ui| {
								if ui.button("➕").clicked() {
									import = Some((i, format!("Custom - {}", self.add_custom_path), vec![self.add_custom_path.clone()]));
									ui.close_menu();
								}
								
								ui.text_edit_singleline(&mut self.add_custom_path);
							});
						});
					});
				});
			}
			
			if let Some((layer, label, paths)) = import {
				let mut dialog = egui_file::FileDialog::open_file(Some(start_path.clone().unwrap_or(crate::config().config.file_dialog_path.clone())))
					.default_filename(format!("{}.aeth", self.meta.name))
					.title(&format!("Import overlay ({label})"));
				dialog.open();
				
				self.import = Some((layer, label, paths, dialog));
			}
		});
		
		if ui.button("➕ Add new layer").clicked() {
			self.layers.push(Layer {
				blend_mode: comp::Blend::Normal,
				color: None,
				textures: Vec::new(),
			});
		}
		
		
		if let Some((layer, label, paths, dialog)) = &mut self.import {
			let config = crate::config();
			match dialog.show(ui.ctx()).state() {
				egui_file::State::Selected => 'outer: {
					if start_path.is_none() {
						config.config.file_dialog_path = dialog.directory().to_path_buf();
						_ = config.save_forced();
					}
					
					let Some(path) = dialog.path() else {break 'outer};
					
					let f = match std::fs::File::open(&path) {
						Ok(v) => v,
						Err(err) => {
							log!(err, "Failed importing file ({err:?})");
							break 'outer;
						}
					};
					
					let Some(ext) = &path.extension().map(|v| v.to_string_lossy().to_string()) else {
						log!(err, "Failed importing file (Has no extension)");
						break 'outer;
					};
					
					let converter = match noumenon::Convert::from_ext(ext, &mut std::io::BufReader::new(f)) {
						Ok(v) => v,
						Err(err) => {
							log!(err, "Failed importing file ({err:?})");
							break 'outer;
						}
					};
					
					let mut buf = Vec::new();
					if let Err(err) = converter.convert("tex", &mut std::io::Cursor::new(&mut buf), None, None::<fn(&str) -> Option<Vec<u8>>>) {
						log!(err, "Failed importing file ({err:?})");
						break 'outer;
					}
					
					let tex = noumenon::format::game::Tex::read(&mut std::io::Cursor::new(&buf)).unwrap();
					let hash = crate::hash_str(blake3::hash(&buf));
					
					let Some(layer) = self.layers.get_mut(*layer) else {
						log!(err, "Failed adding file, layer no longer exists");
						break 'outer;
					};
					
					layer.textures.push((label.clone(), TFile::new(tex, ui.ctx(), hash, paths.clone())));
				}
				
				egui_file::State::Cancelled => {
					if start_path.is_none() {
						config.config.file_dialog_path = dialog.directory().to_path_buf();
						_ = config.save_forced();
					}
				}
				
				_ => {}
			}
		}
	}
	
	pub fn create_modpack(&self, modpack_file: &mut std::fs::File) -> Result<(), crate::resource_loader::BacktraceError> {
		let mut colors = Vec::<(String, [f32; 4])>::new();
		let mut compfiles = HashMap::<String, Comp>::new();
		
		let mut modpack = crate::modman::modpack::ModPack::new(std::io::BufWriter::new(modpack_file), crate::modman::modpack::ModCreationSettings {
			current_game_files_hash: true,
		});
		
		for layer in &self.layers {
			let mut color_modifier = Vec::new();
			if let Some(color) = &layer.color && !colors.iter().any(|v| v.0 == *color) {
				colors.push((color.clone(), *self.colors.get(color).unwrap()));
				color_modifier = vec![comp::Modifier::Color{value: comp::OptionOrStatic::Option(comp::ColorOption(color.clone()))}];
			}
			
			for (label, tfile) in &layer.textures {
				let mut data = std::io::Cursor::new(Vec::new());
				tfile.tex.write(&mut data)?;
				modpack.add_file(&tfile.hash, &data.into_inner())?;
				
				for path in &tfile.paths {
					let comp = compfiles.entry(path.clone())
						.or_insert_with(|| Comp {
							layers: vec![
								comp::Layer {
									name: "Game".to_string(),
									path: Path::Game(path.to_string()),
									modifiers: vec![],
									blend: comp::Blend::Normal,
								},
							],
						});
					
					comp.layers.push(comp::Layer {
						name: label.clone(),
						path: Path::Mod(tfile.hash.clone()),
						modifiers: color_modifier.clone(),
						blend: layer.blend_mode.clone(),
					});
				}
			}
		}
		
		let mut meta = self.meta.clone();
		for (name, color) in colors {
			meta.options.push(meta::OptionType::Option(meta::Option {
				name: name,
				description: String::new(),
				settings: meta::OptionSettings::Rgba(meta::ValueRgba {
					default: color,
					min: [0.0; 4],
					max: [1.0; 4],
				})
			}))
		}
		
		for (path, mut compfile) in compfiles {
			compfile.layers.reverse(); // every day my past decisions come to haunt me
			let data = serde_json::to_vec(&compfile)?;
			let hash_str = crate::hash_str(blake3::hash(&data));
			meta.files.insert(format!("{path}.comp"), hash_str.clone());
			modpack.add_file(&hash_str, &data)?;
		}
		
		modpack.add_meta(&meta)?;
		modpack.finalize()?;
		
		Ok(())
	}
}

impl super::super::View for Tattoo {
	fn title(&self) -> &'static str {
		"Tattoo Overlay Creator"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _viewer: &super::super::Viewer) {
		self.ui_creator(ui, None);
		
		ui.add_space(20.0);
		
		ui.horizontal(|ui| {
			if ui.button("Export").clicked() {
				let mut dialog = egui_file::FileDialog::save_file(Some(crate::config().config.file_dialog_path.clone()))
					.title("Save modpack");
				dialog.open();
				
				self.export = Some(dialog);
			}
			
			let mut save = None;
			if let Some(dialog) = &mut self.export {
				let config = crate::config();
				match dialog.show(ui.ctx()).state() {
					egui_file::State::Selected => {
						save = dialog.path().map(|v| v.to_path_buf());
						
						config.config.file_dialog_path = dialog.directory().to_path_buf();
						_ = config.save_forced();
						self.export = None;
					}
					
					egui_file::State::Cancelled => {
						config.config.file_dialog_path = dialog.directory().to_path_buf();
						_ = config.save_forced();
						self.export = None;
					}
					
					_ => {}
				}
			}
			
			if let Some(path) = save {'outer: {
				let path = path.with_extension("aeth");
				let mut modpack_file = match std::fs::File::create(&path) {
					Ok(v) => v,
					Err(err) => {
						log!(err, "Failed creating modpack file ({err:?})");
						break 'outer;
					}
				};
				
				if let Err(err) = self.create_modpack(&mut modpack_file) {
					log!(err, "Failed creating modpack file ({err:?})");
					break 'outer;
				}
			}}
			
			if ui.button("Create & Import").clicked() {'outer: {
				let mut modpack_file = match tempfile::tempfile() {
					Ok(v) => v,
					Err(err) => {
						log!(err, "Failed creating modpack file ({err:?})");
						break 'outer;
					}
				};
				
				if let Err(err) = self.create_modpack(&mut modpack_file) {
					log!(err, "Failed creating modpack file ({err:?})");
					break 'outer;
				}
				
				let name = self.meta.name.clone();
				let progress = self.progress.clone();
				std::thread::spawn(move || {
					progress.add_task_count(1);
					crate::backend().install_mods(progress.clone(), vec![(name, modpack_file)]);
					progress.progress_task();
				});
			}}
			
			if ui.button("Clear").clicked() {
				*self = Self::new(self.progress.clone());
			}
		});
	}
}

fn racesex_to_facepath(race: Race, female: bool, face: usize, typ: TextureType) -> String {
	let mut id = match race {
		Race::Midlander => 1,
		Race::Highlander => 3,
		Race::Wildwood => 5,
		Race::Duskwight => 5,
		Race::Plainsfolk => 11,
		Race::Dunesfolk => 11,
		Race::SeekerOfTheSun => 7,
		Race::KeeperOfRheMoon => 7,
		Race::SeaWolf => 9,
		Race::Hellsguard => 9,
		Race::Raen => 13,
		Race::Xaela => 13,
		Race::Helions => 15,
		Race::TheLost => 15,
		Race::Rava => 7,
		Race::Veena => 7,
	};
	
	if female {
		id += 1;
	}
	
	format!("chara/human/c{id:02}01/obj/face/f{face:04}/texture/c{id:02}01f{face:04}_fac_{}.tex", typ.path())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Race {
	Midlander,
	Highlander,
	Wildwood,
	Duskwight,
	Plainsfolk,
	Dunesfolk,
	SeekerOfTheSun,
	KeeperOfRheMoon,
	SeaWolf,
	Hellsguard,
	Raen,
	Xaela,
	Helions,
	TheLost,
	Rava,
	Veena,
}

impl Race {
	pub fn sexname(&self, is_female: bool) -> String {
		format!("{} {}", if is_female {"Female"} else {"Male"}, self.to_str())
	}
}

impl EnumTools for Race {
	type Iterator = std::array::IntoIter<Self, 16>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Midlander => "Midlander",
			Self::Highlander => "Highlander",
			Self::Wildwood => "Wildwood",
			Self::Duskwight => "Duskwight",
			Self::Plainsfolk => "Plainsfolk",
			Self::Dunesfolk => "Dunesfolk",
			Self::SeekerOfTheSun => "Seeker of the Sun",
			Self::KeeperOfRheMoon => "Keeper of the Moon",
			Self::SeaWolf => "Sea Wolf",
			Self::Hellsguard => "Hellsguard",
			Self::Raen => "Raen",
			Self::Xaela => "Xaela",
			Self::Helions => "Helions",
			Self::TheLost => "The Lost",
			Self::Rava => "Rava",
			Self::Veena => "Veena",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Midlander,
			Self::Highlander,
			Self::Wildwood,
			Self::Duskwight,
			Self::Plainsfolk,
			Self::Dunesfolk,
			Self::SeekerOfTheSun,
			Self::KeeperOfRheMoon,
			Self::SeaWolf,
			Self::Hellsguard,
			Self::Raen,
			Self::Xaela,
			Self::Helions,
			Self::TheLost,
			Self::Rava,
			Self::Veena,
		].into_iter()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TextureType {
	Diffuse,
	Normal,
	Mask,
}

impl TextureType {
	pub fn name(&self) -> &'static str {
		match self {
			Self::Diffuse => "Diffuse",
			Self::Normal => "Normal",
			Self::Mask => "Mask",
		}
	}
	
	pub fn path(&self) -> &'static str {
		match self {
			Self::Diffuse => "base",
			Self::Normal => "norm",
			Self::Mask => "mask",
		}
	}
}

const BODY_PRESETS: &[(TextureType, &[(&str, &[&str])])] = &[
	(TextureType::Diffuse, &[
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
			"chara/human/c0201/obj/body/b0001/texture/tfgen3midf_base.tex",
			"chara/human/c0401/obj/body/b0001/texture/tfgen3highf_base.tex",
			"chara/human/c1401/obj/body/b0001/texture/tfgen3raenf_base.tex",
			"chara/human/c1401/obj/body/b0101/texture/tfgen3xaelaf_base.tex",
			"chara/human/c1601/obj/body/b0001/texture/tfgen3hrothf_base.tex",
			"chara/human/c1801/obj/body/b0001/texture/tfgen3viera_base.tex",
		]),
		
		("Vanilla Tall Male", &[
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_base.tex", // midlander (elezen, miqote)
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_base.tex", // highlander
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_base.tex", // aura raen
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_base.tex", // aura xaela
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_base.tex", // viera
			
			// with tbse enabled it turns vanilla textures into this path, fuck you too
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_d.tex",
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_d.tex",
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_d.tex",
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_d.tex",
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_d.tex",
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
		
		("HR3", &[
			"chara/human/c0901/obj/body/b0001/texture/c0901b0001_b_d.tex", // roe
			"chara/human/c1501/obj/body/b0001/texture/c1501b0001_b_d.tex", // hrothgar
		]),
		
		("Vanilla Lalafell", &[
			"chara/human/c1101/obj/body/b0001/texture/c1101b0001_base.tex", // male
			// "chara/human/c1201/obj/body/b0001/texture/c1201b0001_base.tex", // female, uses male texture
		]),
		
		("Otopop", &[
			"chara/human/c1101/obj/body/b0001/texture/v01_c1101b0001_g_d.tex",
		]),
	]),
	
	(TextureType::Normal, &[
		("Vanilla Tall Female", &[
			"chara/human/c0201/obj/body/b0001/texture/c0201b0001_norm.tex", // midlander (elezen, miqote, roe)
			"chara/human/c0401/obj/body/b0001/texture/c0401b0001_norm.tex", // highlander
			"chara/human/c1401/obj/body/b0001/texture/c1401b0001_norm.tex", // aura
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
			"chara/human/c0201/obj/body/b0001/texture/tfgen3midf_norm.tex",
			"chara/human/c0401/obj/body/b0001/texture/tfgen3highf_norm.tex",
			"chara/human/c1401/obj/body/b0001/texture/tfgen3raenf_norm.tex",
			"chara/human/c1401/obj/body/b0101/texture/tfgen3xaelaf_norm.tex",
			"chara/human/c1601/obj/body/b0001/texture/tfgen3hrothf_norm.tex",
			"chara/human/c1801/obj/body/b0001/texture/tfgen3viera_norm.tex",
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
		
		("HR3", &[
			"chara/human/c0901/obj/body/b0001/texture/c0901b0001_b_n.tex", // roe
			"chara/human/c1501/obj/body/b0001/texture/c1501b0001_b_n.tex", // hrothgar
		]),
		
		("Vanilla Lalafell", &[
			"chara/human/c1101/obj/body/b0001/texture/c1101b0001_norm.tex", // male
			// "chara/human/c1201/obj/body/b0001/texture/c1201b0001_base.tex", // female, uses male texture
		]),
		
		("Otopop", &[
			"chara/human/c1101/obj/body/b0001/texture/v01_c1101b0001_g_n.tex",
		]),
	]),
	
	(TextureType::Mask, &[
		("Vanilla Tall Female", &[
			"chara/common/texture/skin_mask.tex", // midlander (elezen, miqote, roe, midlander, viera)
			"chara/human/c1401/obj/body/b0001/texture/c1401b0001_mask.tex", // aura
			"chara/human/c1601/obj/body/b0001/texture/v01_c1601b0001_mask.tex", // hrothgar
			"chara/human/c1601/obj/body/b0001/texture/v02_c1601b0001_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v03_c1601b0001_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v04_c1601b0001_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v05_c1601b0001_mask.tex",
		]),
		
		("Bibo", &[
			"chara/bibo_mid_mask.tex",
			"chara/bibo_high_mask.tex",
			"chara/bibo_raen_mask.tex",
			"chara/bibo_xaela_mask.tex",
			"chara/bibo_hroth_mask_v01.tex",
			"chara/bibo_hroth_mask_v02.tex",
			"chara/bibo_hroth_mask_v03.tex",
			"chara/bibo_hroth_mask_v04.tex",
			"chara/bibo_hroth_mask_v05.tex",
			"chara/bibo_viera_mask.tex",
		]),
		
		("TF Gen3", &[ // bibo compatability files atleast
			"chara/human/c0201/obj/body/b0001/texture/tfgen3midf_mask.tex",
			"chara/human/c0401/obj/body/b0001/texture/tfgen3highf_mask.tex",
			"chara/human/c1401/obj/body/b0001/texture/tfgen3raenf_mask.tex",
			"chara/human/c1401/obj/body/b0101/texture/tfgen3xaelaf_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v01_tfgen3hrothf_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v02_tfgen3hrothf_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v03_tfgen3hrothf_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v04_tfgen3hrothf_mask.tex",
			"chara/human/c1601/obj/body/b0001/texture/v05_tfgen3hrothf_mask.tex",
			"chara/human/c1801/obj/body/b0001/texture/tfgen3viera_mask.tex",
		]),
		
		("Vanilla Tall Male", &[
			"chara/common/texture/skin_mask.tex", // midlander (elezen, miqote, midlander, viera)
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_mask.tex", // aura
		]),
		
		("TBSE", &[
			"chara/human/c0101/obj/body/b0001/texture/c0101b0001_b_s.tex", // midlander (elezen, miqote)
			"chara/human/c0301/obj/body/b0001/texture/c0301b0001_b_s.tex", // highlander
			"chara/human/c1301/obj/body/b0001/texture/c1301b0001_b_s.tex", // aura raen
			"chara/human/c1301/obj/body/b0101/texture/c1301b0101_b_s.tex", // aura xaela
			"chara/human/c1701/obj/body/b0001/texture/c1701b0001_b_s.tex", // viera
		]),
		
		("Vanilla Big Male", &[
			"chara/human/c0901/obj/body/b0001/texture/c0901b0001_norm.tex", // roe
			"chara/human/c1501/obj/body/b0001/texture/v01_c1501b0001_norm.tex", // hrothgar
			"chara/human/c1501/obj/body/b0001/texture/v02_c1501b0001_mask.tex",
			"chara/human/c1501/obj/body/b0001/texture/v03_c1501b0001_mask.tex",
			"chara/human/c1501/obj/body/b0001/texture/v04_c1501b0001_mask.tex",
			"chara/human/c1501/obj/body/b0001/texture/v05_c1501b0001_mask.tex",
		]),
		
		("HR3", &[
			"chara/human/c0901/obj/body/b0001/texture/c0901b0001_b_s.tex", // roe
			"chara/human/c1501/obj/body/b0001/texture/v01_c1501b0001_b_s.tex", // hrothgar
			"chara/human/c1501/obj/body/b0001/texture/v02_c1501b0001_b_s.tex", 
			"chara/human/c1501/obj/body/b0001/texture/v03_c1501b0001_b_s.tex", 
			"chara/human/c1501/obj/body/b0001/texture/v04_c1501b0001_b_s.tex", 
			"chara/human/c1501/obj/body/b0001/texture/v05_c1501b0001_b_s.tex", 
		]),
		
		("Vanilla Lalafell", &[
			"chara/common/texture/skin_mask.tex",
		]),
		
		("Otopop", &[
			"chara/human/c1101/obj/body/b0001/texture/v01_c1101b0001_g_s.tex",
		]),
	]),
];