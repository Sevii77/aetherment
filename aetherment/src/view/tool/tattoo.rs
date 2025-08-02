use std::collections::HashMap;
use noumenon::format::{external::Bytes, game::Tex};
use crate::{modman::{composite::tex::{self as comp, Tex as Comp}, meta, Path}, ui_ext::UiExt};

// #[derive(Debug)]
struct TFile {
	tex: Tex,
	img: egui::TextureHandle,
	hash: String,
	composites: HashMap<String, Comp>,
}

impl TFile {
	fn new(tex: Tex, ctx: &egui::Context, hash: String, composites: HashMap<String, Comp>) -> Self {
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
			composites,
		}
	}
}

pub struct Tattoo {
	meta: meta::Meta,
	color: Option<[f32; 4]>,
	blend_mode: comp::Blend,
	textures: HashMap<usize, HashMap<usize, TFile>>,
	
	import: Option<(usize, usize, egui_file::FileDialog)>,
	export: Option<egui_file::FileDialog>,
}

impl Tattoo {
	pub fn new() -> Self {
		Self {
			meta: Default::default(),
			color: None,
			blend_mode: comp::Blend::Normal,
			textures: HashMap::new(),
			
			import: None,
			export: None,
		}
	}
}

impl super::super::View for Tattoo {
	fn title(&self) -> &'static str {
		"Tattoo Overlay Creator"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &crate::Renderer) {
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
		
		ui.label("Color edit option");
		ui.horizontal(|ui| {
			let mut checked = self.color.is_some();
			ui.add(egui::Checkbox::without_text(&mut checked));
			
			if let Some(color) = &mut self.color {
				ui.color_edit(color);
				ui.label("Default");
				
				if !checked {
					self.color = None;
				}
			} else if checked {
				self.color = Some([0.0, 0.0, 0.0, 1.0]);
			}
			
			if !checked {
				ui.label("Disabled");
			}
		});
		ui.add_space(10.0);
		
		ui.label("Blend mode");
		ui.combo_enum_id(&mut self.blend_mode, "blendmode");
		ui.add_space(10.0);
		
		let mut import = None;
		ui.label("Textures");
		for (i, (category, presets)) in PRESETS.iter().enumerate() {
			ui.horizontal(|ui| {
				ui.label(format!("{category}:"));
				
				for (j, (preset, _paths)) in presets.iter().enumerate() {
					if preview_button(ui, *preset, self.textures.get(&i).map(|v| v.get(&j)).flatten()).clicked() {
						import = Some((i, j));
					};
				}
			});
		}
		
		if let Some((category_i, preset_i)) = import {
			let mut dialog = egui_file::FileDialog::open_file(Some(crate::config().config.file_dialog_path.clone()))
				.title(&format!("Import overlay ({} -  {})", PRESETS[category_i].0, PRESETS[category_i].1[preset_i].0));
			dialog.open();
			
			self.import = Some((category_i, preset_i, dialog));
		}
		
		if let Some((category, preset, dialog)) = &mut self.import {
			let config = crate::config();
			match dialog.show(ui.ctx()).state() {
				egui_file::State::Selected => 'outer: {
					config.config.file_dialog_path = dialog.directory().to_path_buf();
					_ = config.save_forced();
					
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
					if let Err(err) = converter.convert("tex", &mut std::io::Cursor::new(&mut buf)) {
						log!(err, "Failed importing file ({err:?})");
						break 'outer;
					}
					
					let tex = noumenon::format::game::Tex::read(&mut std::io::Cursor::new(&buf)).unwrap();
					let hash = crate::hash_str(blake3::hash(&buf));
					
					let mut composites = HashMap::new();
					
					for path in PRESETS[*category].1[*preset].1 {
						log!("creating composite for {path}");
						composites.insert(path.to_string(), Comp {
							layers: vec![
								comp::Layer {
									name: "Overlay".to_string(),
									path: Path::Mod(hash.clone()),
									modifiers: vec![],
									blend: comp::Blend::Normal,
								},
								
								comp::Layer {
									name: "Diffuse".to_string(),
									path: Path::Game(path.to_string()),
									modifiers: vec![],
									blend: comp::Blend::Normal,
								},
							],
						});
					}
					
					self.textures.entry(*category).or_default().insert(*preset, TFile::new(tex, ui.ctx(), hash, composites));
				}
				
				egui_file::State::Cancelled => {
					config.config.file_dialog_path = dialog.directory().to_path_buf();
					_ = config.save_forced();
				}
				
				_ => {}
			}
		}
		
		ui.add_space(20.0);
		if ui.button("Export").clicked() {
			let mut dialog = egui_file::FileDialog::save_file(Some(crate::config().config.file_dialog_path.clone()))
				.title("Save modpack");
			dialog.open();
			
			self.export = Some(dialog);
		}
		
		if let Some(dialog) = &mut self.export {
			let config = crate::config();
			match dialog.show(ui.ctx()).state() {
				egui_file::State::Selected => 'outer: {
					if let Some(path) = dialog.path() {
						let mut meta = self.meta.clone();
						
						if let Some(color) = self.color {
							meta.options.push(meta::OptionType::Option(meta::Option {
								name: "Color".to_string(),
								description: String::new(),
								settings: meta::OptionSettings::Rgba(meta::ValueRgba {
									default: color,
									min: [0.0; 4],
									max: [1.0; 4],
								})
							}))
						}
						
						let modpack_file = match std::fs::File::create(path.with_extension("aeth")) {
							Ok(v) => v,
							Err(err) => {
								log!(err, "Failed creating modpack file ({err:?})");
								break 'outer;
							}
						};
						
						let mut modpack = crate::modman::ModPack::new(std::io::BufWriter::new(modpack_file), crate::modman::ModCreationSettings {
							current_game_files_hash: true,
						});
						
						for (category, presets) in self.textures.iter() {
							for (_preset, tfile) in presets.iter() {
								let mut data = std::io::Cursor::new(Vec::new());
								tfile.tex.write(&mut data).unwrap();
								modpack.add_file(&tfile.hash, &data.into_inner()).unwrap();
								
								for (path, comp) in &tfile.composites {
									let mut comp = comp.clone();
									if *category == 0 { // diffuse
										if self.color.is_some() {
											comp.layers[0].modifiers.clear();
											comp.layers[0].modifiers.push(comp::Modifier::Color{value: comp::OptionOrStatic::Option(comp::ColorOption("Color".to_string()))});
										}
										
										comp.layers[0].blend = self.blend_mode.clone();
									}
									
									let data = serde_json::to_vec(&comp).unwrap();
									let hash_str = crate::hash_str(blake3::hash(&data));
									
									meta.files.insert(format!("{path}.comp"), hash_str.clone());
									modpack.add_file(&hash_str, &data).unwrap();
								}
							}
						}
						
						modpack.add_meta(&meta).unwrap();
						modpack.finalize().unwrap();
					}
					
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
		
		if ui.button("Clear").clicked() {
			*self = Self::new();
		}
	}
}

fn preview_button(ui: &mut egui::Ui, label: &str, preview: Option<&TFile>) -> egui::Response {
	if let Some(preview) = preview {
		ui.button(format!("{label} ✔")).on_hover_ui_at_pointer(|ui| {
			let w = preview.tex.width as f32;
			let h = preview.tex.height as f32;
			let w = 400.0 * (w / h).min(1.0);
			let h = 400.0 * (h / w).min(1.0);
			ui.add(egui::Image::new(&preview.img).max_size(egui::vec2(w, h)));
		})
	} else {
		ui.button(label)
	}
}

const PRESETS: &[(&str, &[(&str, &[&str])])] = &[
	("Diffuse", &[
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
	
	("Normal", &[
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
];