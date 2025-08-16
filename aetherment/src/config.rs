use std::{fs::File, path::{PathBuf, Path}, io::{Write, Read}};
use serde::{Deserialize, Serialize};

pub struct ConfigManager {
	pub config: Config,
	save_check: Option<Config>,
	path: PathBuf,
}

impl ConfigManager {
	pub fn load(path: &Path) -> Self {
		Self {
			config: 's: {
				if let Ok(mut f) = File::open(path) {
					let mut buf = Vec::new();
					if f.read_to_end(&mut buf).is_ok() {
						if let Ok(c) = serde_json::from_slice(&buf) {
							break 's c;
						}
					}
				}
				
				Config::default()
			},
			save_check: None,
			path: path.to_owned(),
		}
	}
	
	pub fn mark_for_changes(&mut self) {
		self.save_check = Some(self.config.clone());
	}
	
	pub fn save(&mut self) -> std::io::Result<()> {
		if let Some(save_check) = self.save_check.take() {
			if self.config != save_check {
				self.save_forced()?;
			}
		}
		
		Ok(())
	}
	
	pub fn save_forced(&self) -> std::io::Result<()> {
		if let Some(parent) = self.path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		
		File::create(&self.path)?.write_all(serde_json::to_string(&self.config)?.as_bytes())?;
		
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
	pub plugin_open_on_launch: bool,
	pub game_install: Option<String>,
	pub repos: Vec<String>,
	pub browser_default_origin: String,
	pub browser_content_rating: crate::remote::ContentRating,
	pub auto_apply_last_viewed: std::time::Duration,
	pub auto_apply_last_interacted: std::time::Duration,
	
	pub mod_paths: Vec<PathBuf>,
	pub file_dialog_path: PathBuf,
	pub active_collection: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			plugin_open_on_launch: false,
			game_install: None,
			repos: Vec::new(),
			browser_default_origin: "Aetherment".to_string(),
			browser_content_rating: crate::remote::ContentRating::Sfw,
			auto_apply_last_viewed: std::time::Duration::from_secs(1),
			auto_apply_last_interacted: std::time::Duration::from_secs(15),
			
			mod_paths: Vec::new(),
			file_dialog_path: dirs::document_dir().unwrap_or(PathBuf::new()),
			active_collection: "Default".to_string(),
		}
	}
}