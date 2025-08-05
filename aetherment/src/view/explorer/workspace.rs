use std::path::{Path, PathBuf};
use egui::Widget;
use crate::modman::meta::Meta;

pub struct Workspace {
	pub root: PathBuf,
	pub meta: Meta,
}

impl Workspace {
	pub fn new(path: &Path) -> Option<Workspace> {
		let meta_path = path.join("meta.json");
		if !meta_path.exists() {return None};
		
		let meta = crate::resource_loader::read_json(&meta_path).ok()?;
		
		Some(Self {
			root: path.to_path_buf(),
			meta,
		})
	}
}

impl super::ExplorerView for Workspace {
	fn title(&self) -> String {
		format!("Workspace - {}", self.meta.name)
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &crate::Renderer) {
		let meta = &mut self.meta;
		let mut changed = false;
		
		ui.label("Name");
		changed |= ui.text_edit_singleline(&mut meta.name).changed();
		ui.add_space(10.0);
		
		ui.label("Description");
		if meta.description.starts_with("[md]") {
			changed |= egui::TextEdit::multiline(&mut meta.description)
				.code_editor()
				.lock_focus(false)
				.ui(ui)
				.changed();
		} else {
			changed |= ui.text_edit_multiline(&mut meta.description).changed();
		}
		ui.add_space(10.0);
		
		ui.label("Version");
		changed |= ui.text_edit_singleline(&mut meta.version).changed();
		ui.add_space(10.0);
		
		ui.label("Author");
		changed |= ui.text_edit_singleline(&mut meta.author).changed();
		ui.add_space(10.0);
		
		ui.label("Website");
		changed |= ui.text_edit_singleline(&mut meta.website).changed();
		ui.add_space(10.0);
		
		ui.label("Tags");
		{
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
		}
		ui.add_space(10.0);
		
		ui.label("Dependencies");
		{
			let mut delete = None;
			for (i, tag) in meta.dependencies.iter_mut().enumerate() {
				ui.horizontal(|ui| {
					changed |= ui.text_edit_singleline(tag).changed();
					if ui.button("🗑").clicked() {
						delete = Some(i);
					}
				});
			}
			
			if let Some(i) = delete {
				meta.dependencies.remove(i);
				changed = true;
			}
			
			if ui.button("➕ Add dependency").clicked() {
				meta.dependencies.push(String::new());
			}
		}
		ui.add_space(10.0);
		
		ui.label("Options");
		{
			
		}
		
		if changed {
			_ = self.meta.save(&self.root.join("meta.json"));
		}
	}
}