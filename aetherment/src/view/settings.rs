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
	
	fn ui(&mut self, ui: &mut egui::Ui, _viewer: &super::Viewer) {
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
		
		ui.collapsing("Proxy", |ui| {
			let mut proxy = config.proxy.is_some();
			ui.checkbox(&mut proxy, "Use Proxy");
			if proxy != config.proxy.is_some() {
				if proxy {
					config.proxy = Some("".to_owned());
				} else {
					config.proxy = None;
				}
			}
			
			if let Some(proxy) = &mut config.proxy {
				egui::TextEdit::singleline(proxy)
					.hint_text("<protocol>://<user>:<password>@<host>:port")
					.show(ui);
				
				ui.label("\
Your proxy config is stored in plain text, this includes username & password!

Protocols
- http: HTTP CONNECT proxy
- https: HTTPS CONNECT proxy
- socks4: SOCKS4
- socks4a: SOCKS4A
- socks5 and socks: SOCKS5

Examples proxy formats
- http://127.0.0.1:8080
- socks5://john:smith@socks.google.com
- john:smith@socks.google.com:8000
- localhost");
			}
		});
		
		_ = config_manager.save();
	}
}