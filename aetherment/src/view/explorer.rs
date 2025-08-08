use std::collections::HashSet;

mod workspace;
mod tree;
mod resource;

pub trait ExplorerView {
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
}

pub struct Explorer {
	id_counter: usize,
	views: egui_dock::DockState<ExplorerTab>,
	last_focused_resource: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex, egui_dock::TabIndex, String)>,
}

impl Explorer {
	pub fn new() -> Self {
		let mut s = Self {
			id_counter: 0,
			views: egui_dock::DockState::new(Vec::new()),
			last_focused_resource: None,
		};
		
		s.add_tab(Box::new(tree::Tree::new()), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/human/c0201/obj/body/b0001/texture/c0201b0001_base.tex")), Split::Horizontal(0.2), None);
		s.add_tab(Box::new(resource::Resource::new("chara/monster/m0934/obj/body/b0001/model/m0934b0001.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/human/c1401/obj/face/f0001/model/c1401f0001_fac.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/model/c0201e6100_top.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/material/v0001/mt_c0201e6100_top_a.mtrl")), Split::Vertical(0.5), None);
		s.add_tab(Box::new(resource::Resource::new("chara/human/c0201/skeleton/base/b0001/skl_c0201b0001.sklb")), Split::None, None);
		
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
				if let Some(focused) = &mut self.last_focused_resource {
					self.views[focused.0][focused.1].tabs_mut().unwrap()[focused.2.0].tab = Box::new(resource::Resource::new(&path));
					self.update_trees();
				}
			}
			Action::OpenComplex((TabType::Tree, v)) => self.add_tab(Box::new(tree::Tree::new()), Split::None, Some(v)),
			Action::OpenComplex((TabType::Resource, v)) => self.add_tab(Box::new(resource::Resource::new("ui/uld/logo_sqex_hr1.tex")), Split::None, Some(v)),
			Action::Close(_path) =>{
				self.update_trees();
			}
			Action::None => {}
		}
	}
}