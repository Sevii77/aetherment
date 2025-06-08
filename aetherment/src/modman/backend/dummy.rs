use super::{ApplyProgress, Backend, Collection, InstallProgress, SettingsType, Status};

pub struct Dummy;
impl Backend for Dummy {
	fn name(&self) -> &'static str {
		"No backend"
	}
	
	fn description(&self) -> &'static str {
		#[cfg(feature = "plugin")]
		return "No valid backend found for plugin";
		#[cfg(feature = "client")]
		return "No valid backend found for standalone";
	}
	
	// fn is_functional(&self) -> bool {false}
	fn get_status(&self) -> Status {Status::Error("Dummy backend does not support mod installation".to_string())}
	
	fn get_mods(&self) -> Vec<String> {Vec::new()}
	fn get_active_collection(&self) -> String {String::new()}
	fn get_collections(&self) -> Vec<Collection> {Vec::new()}
	fn install_mods(&mut self, _progress: InstallProgress, _files: Vec<(String, std::fs::File)>) {}
	
	fn apply_mod_settings(&mut self, _mod_id: &str, _collection_id: &str, _settings: SettingsType) {}
	fn finalize_apply(&mut self, _progress: ApplyProgress) {}
	fn apply_queue_size(&self) -> usize {0}
	
	fn apply_services(&self) {}
	
	fn load_mods(&mut self) {}
	fn get_mod_meta(&self, _mod_id: &str) -> Option<&crate::modman::meta::Meta> {None}
	fn get_mod_asset(&self, _mod_id: &str, _path: &str) -> std::io::Result<Vec<u8>> {Err(std::io::ErrorKind::Unsupported.into())}
	// fn get_mod_settings(&self, _mod_id: &str, _collection_id: &str) -> Option<crate::modman::settings::Settings> {None}
	
	fn get_mod_enabled(&self, _mod_id: &str, _collection_id: &str) -> bool {false}
	fn set_mod_enabled(&mut self, _mod_id: &str, _collection_id: &str, _enabled: bool) {}
	
	fn get_mod_priority(&self, _mod_id: &str, _collection_id: &str) -> i32 {0}
	fn set_mod_priority(&mut self, _mod_id: &str, _collection_id: &str, _priority: i32) {}
}