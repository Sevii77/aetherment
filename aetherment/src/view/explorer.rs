use std::{collections::HashSet, io::Write};

mod workspace;
mod tree;
mod resource;

pub trait ExplorerView {
	fn as_any(&self) -> &dyn std::any::Any;
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
	fn title(&self) -> String;
	fn path(&self) -> Option<&str>;
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &renderer::Renderer) -> Action;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
	None,
	OpenNew(String),
	OpenExisting(String),
	OpenComplex((TabType, (egui_dock::SurfaceIndex, egui_dock::NodeIndex))),
	Close(String),
	Export((String, usize)),
}

impl Action {
	pub fn or(&mut self, other: Self) {
		if *self == Self::None {
			*self = other;
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
	Tree,
	Resource,
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
			self.action.or(tab.tab.ui(ui, self.renderer));
		});
	}
	
	fn add_popup(&mut self, ui: &mut egui::Ui, surface: egui_dock::SurfaceIndex, node: egui_dock::NodeIndex) {
		ui.set_min_width(150.0);
		
		if ui.selectable_label(false, "Add Tree").clicked() {
			self.action = Action::OpenComplex((TabType::Tree, (surface, node)));
		}
		
		if ui.selectable_label(false, "Add Resource View").clicked() {
			self.action = Action::OpenComplex((TabType::Resource, (surface, node)));
		}
	}
	
	fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
		if let Some(path) = tab.tab.path() {
			self.action = Action::Close(path.to_owned());
		}
		
		true
	}
	
	fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {
		let Some(path) = tab.tab.path() else {return};
		
		if ui.button("Export").clicked() {
			self.action = Action::Export((path.to_owned(), tab.id));
			ui.close_menu();
		}
	}
}

enum FileDialogStatus {
	Idle,
	Exporting(egui_file::FileDialog, usize),
	Importing(egui_file::FileDialog, usize),
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
		
		s.add_tab(Box::new(tree::Tree::new()), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/monster/m0934/obj/body/b0001/model/m0934b0001.mdl")), Split::Horizontal(0.2), None);
		s.add_tab(Box::new(resource::Resource::new("chara/human/c1401/obj/face/f0001/model/c1401f0001_fac.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("bgcommon/hou/indoor/general/0080/texture/fun_b0_m0080_1a_d.tex")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("bgcommon/hou/indoor/general/0080/bgparts/fun_b0_m0080.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/model/c0201e6100_top.mdl")), Split::None, None);
		
		s.add_tab(Box::new(resource::Resource::new("chara/human/c0201/skeleton/base/b0001/skl_c0201b0001.sklb")), Split::Vertical(0.5), None);
		
		s.add_tab(Box::new(resource::Resource::new("bgcommon/hou/indoor/general/0080/material/fun_b0_m0080_1a.mtrl")), Split::Horizontal(0.4), None);
		s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/material/v0001/mt_c0201e6100_top_a.mtrl")), Split::None, None);
		
		s
	}
	
	fn update_trees(&mut self) {
		let mut open_paths = HashSet::new();
		for surface in self.views.iter_surfaces() {
			for node in surface.iter_nodes() {
				for tab in node.iter_tabs() {
					if let Some(path) = tab.tab.path() {
						open_paths.insert(path.to_owned());
					}
				}
			}
		}
		
		for (_, tab) in self.views.iter_all_tabs_mut() {
			let Some(tree) = tab.tab.as_any_mut().downcast_mut::<tree::Tree>() else {continue};
			tree.open_paths = open_paths.clone();
		}
	}
	
	fn add_tab(&mut self, tab: Box<dyn ExplorerView>, split: Split, surface_node: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex)>) {
		self.last_focused_resource = None;
		self.id_counter += 1;
		let tab = ExplorerTab {
			id: self.id_counter,
			tab
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
					let path = path.to_string();
					let tab_id = tab.id;
					let node = self.views.iter_all_nodes().nth(focused.1.0).unwrap();
					if let Some(index) = node.1.iter_tabs().position(|v| v.id == tab_id) {
						self.last_focused_resource = Some((focused.0, focused.1, egui_dock::TabIndex(index), path));
					}
				}
			}
		}
		
		match viewer.action {
			Action::OpenNew(path) => self.add_tab(Box::new(resource::Resource::new(&path)), Split::None, self.last_focused_resource.as_ref().map(|v| (v.0, v.1))),
			Action::OpenExisting(path) => {
				if !'s: {
					let Some(focused) = &mut self.last_focused_resource else {break 's false};
					let node = &mut self.views[focused.0][focused.1];
					let Some(tabs) = node.tabs_mut() else {break 's false};
					tabs[focused.2.0].tab = Box::new(resource::Resource::new(&path));
					self.update_trees();
					
					true
				} {
					self.add_tab(Box::new(resource::Resource::new(&path)), Split::None, self.last_focused_resource.as_ref().map(|v| (v.0, v.1)));
				}
			}
			Action::OpenComplex((TabType::Tree, v)) => self.add_tab(Box::new(tree::Tree::new()), Split::None, Some(v)),
			Action::OpenComplex((TabType::Resource, v)) => self.add_tab(Box::new(resource::Resource::new("ui/uld/logo_sqex_hr1.tex")), Split::None, Some(v)),
			Action::Close(_path) =>{
				self.update_trees();
			}
			Action::Export((path, tab_id)) => {
				if let FileDialogStatus::Idle = self.file_dialog {
					let mut dialog = egui_file::FileDialog::save_file(Some(crate::config().config.file_dialog_path.clone()))
						.title(&format!("Exporting {}", path.split("/").last().unwrap()));
					dialog.open();
					
					self.file_dialog = FileDialogStatus::Exporting(dialog, tab_id);
				}
			}
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
												break 'd (resource.resource.export(), game_path);
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
										crate::noumenon().unwrap().file::<Vec<u8>>(path).ok()
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
			
			FileDialogStatus::Importing(dialog, _id) => {
				match dialog.show(ui.ctx()).state() {
					egui_file::State::Selected => {
						save_path(dialog.directory().to_path_buf());
						self.file_dialog = FileDialogStatus::Idle;
						
						// TODO
					}
					
					egui_file::State::Cancelled => {
						save_path(dialog.directory().to_path_buf());
						self.file_dialog = FileDialogStatus::Idle;
					}
					
					_ => {}
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