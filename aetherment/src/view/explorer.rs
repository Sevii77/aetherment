mod workspace;
mod tree;
mod resource;

pub trait ExplorerView {
	fn title(&self) -> String;
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &crate::Renderer);
}

enum TabType {
	Tree,
	Resource,
}

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
	add: Option<(TabType, (egui_dock::SurfaceIndex, egui_dock::NodeIndex))>,
	renderer: &'r crate::Renderer,
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
			tab.tab.ui(ui, self.renderer);
		});
	}
	
	fn add_popup(&mut self, ui: &mut egui::Ui, surface: egui_dock::SurfaceIndex, node: egui_dock::NodeIndex) {
		ui.set_min_width(150.0);
		
		if ui.selectable_label(false, "Add Tree").clicked() {
			self.add = Some((TabType::Tree, (surface, node)));
		}
		
		if ui.selectable_label(false, "Add Resource View").clicked() {
			self.add = Some((TabType::Resource, (surface, node)));
		}
	}
}

pub struct Explorer {
	id_counter: usize,
	views: egui_dock::DockState<ExplorerTab>,
}

impl Explorer {
	pub fn new() -> Self {
		let mut s = Self {
			id_counter: 0,
			views: egui_dock::DockState::new(Vec::new()),
		};
		
		s.add_tab(Box::new(tree::Tree::new()), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("common/graphics/texture/-mogu_anime_en.tex")), Split::Horizontal(0.2), None);
		s.add_tab(Box::new(resource::Resource::new("common/graphics/texture/-caustics.tex")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/human/c0201/obj/body/b0001/texture/c0201b0001_base.tex")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/monster/m0934/obj/body/b0001/model/m0934b0001.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/human/c1401/obj/face/f0001/model/c1401f0001_fac.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/model/c0201e6100_top.mdl")), Split::None, None);
		s.add_tab(Box::new(resource::Resource::new("chara/equipment/e6100/material/v0001/mt_c0201e6100_top_a.mtrl")), Split::Vertical(0.5), None);
		
		s
	}
	
	fn add_tab(&mut self, tab: Box<dyn ExplorerView>, split: Split, surface_node: Option<(egui_dock::SurfaceIndex, egui_dock::NodeIndex)>) {
		self.id_counter += 1;
		let tab = ExplorerTab {
			id: self.id_counter,
			tab
		};
		
		'split: {
			// let Some(fraction) = split else {break 'split};
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
	}
}

impl super::View for Explorer {
	fn title(&self) -> &'static str {
		"Explorer"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, renderer: &crate::Renderer) {
		let mut viewer = Viewer {
			add: None,
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
		
		match viewer.add {
			Some((TabType::Tree, v)) => self.add_tab(Box::new(tree::Tree::new()), Split::None, Some(v)),
			Some((TabType::Resource, v)) => self.add_tab(Box::new(resource::Resource::new("ui/uld/logo_sqex_hr1.tex")), Split::None, Some(v)),
			None => {}
		}
	}
}