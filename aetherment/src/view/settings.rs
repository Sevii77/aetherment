pub struct Settings {
	
}

impl Settings {
	pub fn new() -> Self {
		Self {
			
		}
	}
	
	pub fn draw(&mut self, ui: &mut renderer::Ui) {
		let config_manager = crate::config();
		config_manager.mark_for_changes();
		let config = &mut config_manager.config;
		
		#[cfg(not(feature = "plugin"))]
		ui.horizontal(|ui| {
			let mut game_install = config.game_install.is_some();
			ui.checkbox("Custom Game install location", &mut game_install);
			if game_install != config.game_install.is_some() {
				if game_install {
					config.game_install = Some("".to_owned());
				} else {
					config.game_install = None;
				}
			}
			
			if let Some(game_install) = &mut config.game_install {
				ui.input_text("", game_install);
			}
			
			ui.helptext("Path to the game, use this if you use a custom location where autodetection fails (requires a restart (for now))\nExample: Z:/SteamLibrary/steamapps/common/FINAL FANTASY XIV - A Realm Reborn")
		});
		
		_ = config_manager.save();
	}
}