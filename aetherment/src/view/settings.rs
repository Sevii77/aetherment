use crate::ui_ext::UiExt;

pub struct Settings {
	
}

impl Settings {
	pub fn new() -> Self {
		Self {
			
		}
	}
}

impl super::View for Settings {
	fn title(&self) -> &'static str {
		"Settings"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) {
		let config_manager = crate::config();
		config_manager.mark_for_changes();
		let config = &mut config_manager.config;
		
		#[cfg(not(feature = "plugin"))]
		ui.horizontal(|ui| {
			let mut game_install = config.game_install.is_some();
			ui.checkbox(&mut game_install, "Custom Game install location");
			if game_install != config.game_install.is_some() {
				if game_install {
					config.game_install = Some("".to_owned());
				} else {
					config.game_install = None;
				}
			}
			
			if let Some(game_install) = &mut config.game_install {
				ui.text_edit_singleline(game_install);
			}
			
			ui.helptext("Path to the game, use this if you use a custom location where autodetection fails (requires a restart (for now))\nExample: Z:/SteamLibrary/steamapps/common/FINAL FANTASY XIV - A Realm Reborn")
		});
		
		#[cfg(feature = "plugin")]
		ui.checkbox(&mut config.plugin_open_on_launch, "Open window on launch");
		
		ui.collapsing("Browser", |ui| {
			ui.horizontal(|ui| {
				ui.text_edit_singleline(&mut config.browser_default_origin);
				ui.label("Default origin");
			});
			ui.combo_enum(&mut config.browser_content_rating, "Content Rating");
		});
		
		ui.collapsing("Auto Apply", |ui| {
			ui.horizontal(|ui| {
				let mut val = config.auto_apply_last_viewed.as_secs();
				if ui.num_edit(&mut val, "Seconds since last viewed").changed() {
					config.auto_apply_last_viewed = std::time::Duration::from_secs(val);
				}
				ui.helptext("Time in seconds since the mods page was last seen in order to auto apply.");
			});
			
			ui.horizontal(|ui| {
				let mut val = config.auto_apply_last_interacted.as_secs();
				if ui.num_edit(&mut val, "Seconds since last interaction").changed() {
					config.auto_apply_last_interacted = std::time::Duration::from_secs(val);
				}
				ui.helptext("Time in seconds since a change to a mod setting was made in order to auto apply.");
			});
		});
		
		_ = config_manager.save();
	}
}