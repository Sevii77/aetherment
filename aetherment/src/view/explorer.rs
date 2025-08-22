use std::{collections::HashSet, io::Write};

mod workspace;
mod tree;
mod resource;

pub trait ExplorerView {
	fn as_any(&self) -> &dyn std::any::Any;
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
	fn title(&self) -> String;
	fn path(&self) -> Option<&resource::Path>;
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) -> Action;
}

#[derive(Debug, Clone)]
pub enum Action {
	None,
	OpenNew(TabType),
	OpenExisting(TabType),
	OpenComplex((TabType, (egui_dock::SurfaceIndex, egui_dock::NodeIndex))),
	Close(String),
	Save,
	#[doc(hidden)] SaveTab(usize),
	ModAssign,
	#[doc(hidden)] ModAssignTab(usize),
	Changed,
	Export((resource::Path, usize)),
	Import((resource::Path, usize)),
}

impl Action {
	pub fn or(&mut self, other: Self) {
		if matches!(*self, Self::None) {
			*self = other;
		}
	}
}

pub struct ModInfo {
	pub root: std::path::PathBuf,
	pub meta: std::rc::Rc<std::cell::RefCell<crate::modman::meta::Meta>>,
	pub option: Option<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum TabType {
	Tree,
	Resource(resource::Path),
	Meta(std::rc::Rc<std::cell::RefCell<crate::modman::meta::Meta>>, std::path::PathBuf)
}

impl Into<Box<dyn ExplorerView>> for TabType {
	fn into(self) -> Box<dyn ExplorerView> {
		match self {
			TabType::Tree => Box::new(tree::Tree::new()),
			TabType::Resource(path) => Box::new(resource::Resource::new(path)),
			TabType::Meta(meta, root) => Box::new(workspace::Workspace::new(meta, root)),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
enum Split {
	None,
	Horizontal(f32),
	Vertical(f32),
}

struct ExplorerTab {
	id: usize,
	tab: Box<dyn ExplorerView>,
}

struct Viewer<'r> {
	action: Action,
	renderer: &'r renderer::Renderer,
}

impl<'r> egui_dock::TabViewer for Viewer<'r> {
	type Tab = ExplorerTab;
	
	fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
		egui::Id::new(tab.id)
	}
	
	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		tab.tab.title().into()
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
		ui.push_id(tab.id, |ui| {
			match tab.tab.ui(ui, self.renderer) {
				Action::Save => self.action = Action::SaveTab(tab.id),
				Action::ModAssign => self.action = Action::ModAssignTab(tab.id),
				act => self.action.or(act),
			}
		});
	}
	
	fn add_popup(&mut self, ui: &mut egui::Ui, surface: egui_dock::SurfaceIndex, node: egui_dock::NodeIndex) {
		ui.set_min_width(150.0);
		
		if ui.selectable_label(false, "Add Tree").clicked() {
			self.action = Action::OpenComplex((TabType::Tree, (surface, node)));
		}
		
		if ui.selectable_label(false, "Add Resource View").clicked() {
			self.action = Action::OpenComplex((TabType::Resource(resource::Path::Game("ui/uld/logo_sqex_hr1.tex".to_string())), (surface, node)));
		}
	}
	
	fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
		if let Some(path) = tab.tab.path() {
			self.action = Action::Close(path.as_path().to_owned());
		}
		
		true
	}
	
	fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {
		let Some(path) = tab.tab.path() else {return};
		
		if ui.button("Export").clicked() {
			self.action = Action::Export((path.clone(), tab.id));
			ui.close_menu();
		}
		
		if ui.button("Import").clicked() {
			self.action = Action::Import((path.clone(), tab.id));
			ui.close_menu();
		}
	}
}

enum FileDialogStatus {
	Idle,
	Exporting(egui_file::FileDialog, usize),
	Importing(crate::ui_ext::ImporterDialog, usize),
	Result(egui::RichText),
}

pub struct Explorer {
	id_counter: usize,
	views: egui_dock::DockState<ExplorerTab>,
	last_focused_resource: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex, egui_dock::TabIndex, String)>,
	file_dialog: FileDialogStatus,
}

impl Explorer {
	pub fn new() -> Self {
		let mut s = Self {
			id_counter: 0,
			views: egui_dock::DockState::new(Vec::new()),
			last_focused_resource: None,
			file_dialog: FileDialogStatus::Idle,
		};
		
		s.add_tab(TabType::Tree, Split::None, None);
		s.add_tab(TabType::Resource(resource::Path::Game("bgcommon/hou/indoor/general/0080/texture/fun_b0_m0080_1a_d.tex".to_string())), Split::Horizontal(0.2), None);
		
		// s.add_tab(Box::new(resource::Resource::new("chara/monster/m0934/obj/body/b0001/model/m0934b0001.mdl")), Split::Horizontal(0.2), None);
		// s.add_tab(Box::new(resource::Resource::new("chara/human/c1401/obj/face/f0001/model/c1401f0001_fac.mdl")), Split::None, None);
		// s.add_tab(Box::new(resource::Resource::new("bgcommon/hou/indoor/general/0080/texture/fun_b0_m0080_1a_d.tex")), Split::None, None);
		// s.add_tab(Box::new(resource::Resource::new("bgcommon/hou/indoor/general/0080/bgparts/fun_b0_m0080.mdl")), Split::None, None);
		// s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/model/c0201e6100_top.mdl")), Split::None, None);
		// 
		// s.add_tab(Box::new(resource::Resource::new("chara/human/c0201/skeleton/base/b0001/skl_c0201b0001.sklb")), Split::Vertical(0.5), None);
		// 
		// s.add_tab(Box::new(resource::Resource::new("bgcommon/hou/indoor/general/0080/material/fun_b0_m0080_1a.mtrl")), Split::Horizontal(0.4), None);
		// s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/material/v0001/mt_c0201e6100_top_a.mtrl")), Split::None, None);
		
		s
	}
	
	fn update_trees(&mut self) {
		let mut open_paths = HashSet::new();
		for surface in self.views.iter_surfaces() {
			for node in surface.iter_nodes() {
				for tab in node.iter_tabs() {
					if let Some(path) = tab.tab.path() {
						open_paths.insert(path.as_path().to_owned());
					}
				}
			}
		}
		
		for (_, tab) in self.views.iter_all_tabs_mut() {
			let Some(tree) = tab.tab.as_any_mut().downcast_mut::<tree::Tree>() else {continue};
			tree.open_paths = open_paths.clone();
		}
	}
	
	fn get_mod_info(&self) -> Option<ModInfo> {
		for (_, tab) in self.views.iter_all_tabs() {
			let Some(tree) = tab.tab.as_any().downcast_ref::<tree::Tree>() else {continue};
			let Some(mod_info) = tree.get_mod_info() else {continue};
			return Some(mod_info);
		}
		
		None
	}
	
	// fn add_tab(&mut self, tab: Box<dyn ExplorerView>, split: Split, surface_node: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex)>) {
	fn add_tab(&mut self, tab: impl Into<Box<dyn ExplorerView>>, split: Split, surface_node: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex)>) {
		self.last_focused_resource = None;
		self.id_counter += 1;
		let tab = ExplorerTab {
			id: self.id_counter,
			tab: tab.into(),
		};
		
		'split: {
			let Some(surface_node) = surface_node.or_else(||
				self.views.focused_leaf().or_else(||
					self.views.iter_all_tabs().last().map(|v| v.0))) else {break 'split};
			
			match split {
				Split::None => break 'split,
				Split::Horizontal(fraction) => self.views.split(surface_node, egui_dock::Split::Right, fraction, egui_dock::Node::leaf(tab)),
				Split::Vertical(fraction) => self.views.split(surface_node, egui_dock::Split::Below, fraction, egui_dock::Node::leaf(tab)),
			};
			
			return;
		}
		
		if let Some(surface_node) = surface_node {
			self.views.set_focused_node_and_surface(surface_node);
		}
		
		self.views.push_to_focused_leaf(tab);
		self.update_trees();
	}
	
	fn get_tab(&self, tab_id: usize) -> Option<&ExplorerTab> {
		for surface in self.views.iter_surfaces() {
			for node in surface.iter_nodes() {
				let Some(tabs) = node.tabs() else {continue};
				for tab in tabs {
					if tab.id == tab_id {
						return Some(tab);
					}
				}
			}
		}
		
		None
	}
	
	fn get_tab_mut(&mut self, tab_id: usize) -> Option<&mut ExplorerTab> {
		for surface in self.views.iter_surfaces_mut() {
			for node in surface.iter_nodes_mut() {
				let Some(tabs) = node.tabs_mut() else {continue};
				for tab in tabs {
					if tab.id == tab_id {
						return Some(tab);
					}
				}
			}
		}
		
		None
	}
	
	fn save_data(&self, path: &resource::Path, is_composite: bool, data: Vec<u8>) -> Result<(), String> {
		let (root, meta, option) = match path {
			resource::Path::Mod{mod_root, mod_meta, option, ..} =>
				(mod_root.clone(), mod_meta.clone(), option.clone()),
			
			_ => match self.get_mod_info() {
				Some(v) => (v.root, v.meta, v.option),
				None => return Err("There is no active mod, open a tree if needed and open a mod".to_string())
			}
		};
		
		let hash = crate::hash_str(blake3::hash(&data));
		let ext = if is_composite {format!("{}.comp", path.ext())} else {path.ext()};
		// let option_dirs = option.as_ref().map_or(String::new(), |(a, b)| format!("{a}/{b}/"));
		// let path_rel = format!("{}/{option_dirs}{hash}.{ext}", path.as_path());
		let path_rel = format!("{hash}.{ext}");
		let game_path = if is_composite {format!("{}.comp", path.as_path().trim_end_matches(".comp"))} else {path.as_path().to_string()};
		
		match &option {
			Some((option_name, suboption_name)) => 's: {
				use crate::modman::meta;
				for opt in meta.borrow_mut().options.iter_mut() {
					let meta::OptionType::Option(opt) = opt else {continue};
					if opt.name != *option_name {continue};
					let (meta::OptionSettings::MultiFiles(sub) | meta::OptionSettings::SingleFiles(sub)) = &mut opt.settings else {continue};
					for sub in &mut sub.options {
						if sub.name != *suboption_name {continue};
						sub.files.insert(game_path, path_rel.clone());
						break 's;
					}
				}
				
				return Err("Option to import for no longer exists".to_string())
			}
			
			None => _ = meta.borrow_mut().files.insert(game_path, path_rel.clone()),
		}
		
		let file_path = root.join("files").join(path_rel);
		_ = std::fs::create_dir_all(file_path.parent().unwrap());
		
		if let Err(err) = std::fs::write(file_path, data) {
			return Err(format!("Failed writing imported file to disk {err:?}"));
		}
		
		_ = meta.borrow().save(&root.join("meta.json"));
		
		Ok(())
	}
}

impl super::View for Explorer {
	fn title(&self) -> &'static str {
		"Explorer"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) {
		let mut viewer = Viewer {
			action: Action::None,
			renderer,
		};
		
		egui_dock::DockArea::new(&mut self.views)
			.id(egui::Id::new("explorer_tabs"))
			.style(egui_dock::Style::from_egui(ui.style().as_ref()))
			.show_add_buttons(true)
			.show_add_popup(true)
			.show_leaf_close_all_buttons(false)
			.show_leaf_collapse_buttons(false)
			.show_inside(ui, &mut viewer);
		
		if let Some(focused) = self.views.focused_leaf() {
			if let Some((_, tab)) = self.views.find_active_focused() {
				if let Some(path) = tab.tab.path() {
					let path = path.as_path().to_string();
					let tab_id = tab.id;
					let node = self.views.iter_all_nodes().nth(focused.1.0).unwrap();
					if let Some(index) = node.1.iter_tabs().position(|v| v.id == tab_id) {
						self.last_focused_resource = Some((focused.0, focused.1, egui_dock::TabIndex(index), path));
					}
				}
			}
		}
		
		match viewer.action {
			Action::OpenNew(tab) =>
				self.add_tab(tab, Split::None, self.last_focused_resource.as_ref().map(|v| (v.0, v.1))),
			
			Action::OpenExisting(tab) => 'o: {
				's: {
					let Some(focused) = &mut self.last_focused_resource else {break 's};
					let node = &mut self.views[focused.0][focused.1];
					let Some(tabs) = node.tabs_mut() else {break 's};
					if tabs.len() <= focused.2.0 {break 's};
					tabs[focused.2.0].tab = tab.into();
					self.update_trees();
					break 'o;
				}
				
				self.add_tab(tab, Split::None, self.last_focused_resource.as_ref().map(|v| (v.0, v.1)));
			}
			
			Action::OpenComplex((tab, v)) =>
				self.add_tab(tab, Split::None, Some(v)),
			
			Action::Close(_path) =>
				self.update_trees(),
			
			Action::SaveTab(tab_id) => 'o: {
				let Some(tab) = self.get_tab_mut(tab_id) else {break 'o};
				let Some(resource) = tab.tab.as_any_mut().downcast_mut::<resource::Resource>() else {break 'o};
				let Some(path) = resource.path() else {break 'o};
				let path = path.clone();
				let ext = path.ext();
				
				let mut writer = std::io::Cursor::new(Vec::new());
				match resource.resource.export() {
					resource::Export::Converter(converter) =>
						if let Err(_) = converter.convert(&ext, &mut writer, None, None::<fn(&str) -> Option<Vec<u8>>>) {break 'o},
					
					resource::Export::Bytes(bytes) =>
						if let Err(_) = writer.write_all(&bytes) {break 'o},
					
					resource::Export::Invalid =>
						break 'o,
				}
				
				resource.changed_content = false;
				
				let is_composite = resource.resource.is_composite();
				_ = self.save_data(&path, is_composite, writer.into_inner());
			}
			
			Action::ModAssignTab(tab_id) => 'o: {
				let Some(info) = self.get_mod_info() else {break 'o};
				let Some(tab) = self.get_tab_mut(tab_id) else {break 'o};
				let Some(resource) = tab.tab.as_any_mut().downcast_mut::<resource::Resource>() else {break 'o};
				resource.resource.assign_mod(info);
			}
			
			Action::Export((path, tab_id)) => {
				if let FileDialogStatus::Idle = self.file_dialog {
					let mut dialog = egui_file::FileDialog::save_file(Some(crate::config().config.file_dialog_path.clone()))
						.title(&format!("Exporting {}", path.as_path().split("/").last().unwrap()));
					dialog.open();
					
					self.file_dialog = FileDialogStatus::Exporting(dialog, tab_id);
				}
			}
			
			Action::Import((path, tab_id)) => {
				if let FileDialogStatus::Idle = self.file_dialog {
					let dialog = crate::ui_ext::ImporterDialog::new(format!("Importing {}", path.as_path().split("/").last().unwrap()), path.ext());
					self.file_dialog = FileDialogStatus::Importing(dialog, tab_id);
				}
			}
			
			// used for inner tabs to indicate they want to save their data, will be converted to SaveTab(tab_id)
			Action::Save |
			// same as save
			Action::ModAssign |
			// used for inner resource tabs to indicate their content changed
			Action::Changed |
			Action::None => {}
		}
		
		match &mut self.file_dialog {
			FileDialogStatus::Exporting(dialog, id) => {
				match dialog.show(ui.ctx()).state() {
					egui_file::State::Selected => {
						let msg = 'x: {
							let id = *id;
							let path = dialog.path().map(|v| v.to_owned());
							save_path(dialog.directory().to_path_buf());
							
							let Some(path) = path else {break 'x "Selected path is invalid".to_string()};
							let Some(ext) = path.extension().map(|v| v.to_string_lossy().to_string()) else {break 'x "Invalid extension".to_string()};
							
							let (data, game_path) = 'd: {
								for surface in self.views.iter_surfaces() {
									for node in surface.iter_nodes() {
										let Some(tabs) = node.tabs() else {continue};
										for tab in tabs {
											if tab.id == id {
												let Some(resource) = tab.tab.as_any().downcast_ref::<resource::Resource>() else {break 'x "Resource to export is somehow not a resource".to_string()};
												let Some(game_path) = resource.path() else {break 'x "Resource to export somehow does not have a path".to_string()};
												break 'd (resource.resource.export(), game_path.as_path());
											}
										}
									}
								}
								
								break 'x "Tab to export is no longer valid".to_string();
							};
							
							if let resource::Export::Invalid = data {break 'x "Resource to export does not support exporting (currently)".to_string()};
							
							let file = match std::fs::File::create(path) {
								Ok(v) => v,
								Err(e) => break 'x format!("Failed to create time\n{e:#?}"),
							};
							let mut writer = std::io::BufWriter::new(file);
							
							match data {
								resource::Export::Converter(converter) => {
									fn file_reader(path: &str) -> Option<Vec<u8>> {
										crate::noumenon_instance().unwrap().file::<Vec<u8>>(path).ok()
									}
									
									if let Err(e) = converter.convert(&ext, &mut writer, Some(game_path), Some(file_reader)) {
										break 'x format!("Failed converting file to requested output format\n{e:#?}");
									};
								}
								
								resource::Export::Bytes(bytes) => {
									if let Err(e) = writer.write_all(&bytes) {
										break 'x format!("Failed writing raw bytes to file\n{e:#?}");
									};
								}
								
								resource::Export::Invalid => unreachable!(),
							}
							
							String::new()
						};
						
						let msg = if msg.is_empty() {
							egui::RichText::new("Export successful").heading()
						} else {
							egui::RichText::new(msg).color(egui::Color32::RED).heading()
						};
						
						self.file_dialog = FileDialogStatus::Result(msg);
					}
					
					egui_file::State::Cancelled => {
						save_path(dialog.directory().to_path_buf());
						self.file_dialog = FileDialogStatus::Idle;
					}
					
					_ => {}
				}
			}
			
			FileDialogStatus::Importing(dialog, tab_id) => {
				match dialog.show(ui) {
					Ok(crate::ui_ext::DialogResult::Success(data)) => {
						let msg = 'x: {
							let tab_id = *tab_id;
							let Some(tab) = self.get_tab(tab_id) else {break 'x "Tab to import is no longer valid".to_string()};
							let Some(resource) = tab.tab.as_any().downcast_ref::<resource::Resource>() else {break 'x "Resource to import is somehow not a resource".to_string()};
							let Some(path) = resource.path() else {break 'x "Resource to import somehow does not have a path".to_string()};
							
							if let Err(err) = self.save_data(path, resource.resource.is_composite(), data) {
								break 'x err;
							}
							
							String::new()
						};
						
						let msg = if msg.is_empty() {
							egui::RichText::new("Import successful").heading()
						} else {
							egui::RichText::new(msg).color(egui::Color32::RED).heading()
						};
						
						self.file_dialog = FileDialogStatus::Result(msg);
					}
					
					Ok(crate::ui_ext::DialogResult::Cancelled) =>
						self.file_dialog = FileDialogStatus::Idle,
					
					Ok(crate::ui_ext::DialogResult::Busy) => {}
					
					Err(err) =>
						self.file_dialog = FileDialogStatus::Result(egui::RichText::new(err.to_string()).color(egui::Color32::RED).heading()),
				}
			}
			
			FileDialogStatus::Result(msg) => {
				let mut close = false;
				egui::Modal::new(egui::Id::new("dialogresult")).show(ui.ctx(), |ui| {
					ui.set_max_width(600.0);
					ui.vertical_centered(|ui| {
						ui.label(msg.to_owned());
						
						ui.add_space(32.0);
						if ui.button("Close").clicked() {
							close = true;
						}
					});
				});
				
				if close {
					self.file_dialog = FileDialogStatus::Idle;
				}
			}
			
			FileDialogStatus::Idle => {}
		}
	}
}

fn save_path(path: std::path::PathBuf) {
	let config = crate::config();
	config.config.file_dialog_path = path;
	_ = config.save_forced();
}