use serde::{Deserialize, Serialize};
use crate::render_helper::EnumTools;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Issue {
	UiResolution(String),
	UiTheme(String),
	Collection(String),
}

#[derive(Debug, Clone)]
pub enum Status {
	Ok,
	Warning(String),
}

impl Issue {
	pub fn get_status(&self) -> Status {
		let funcs = unsafe{FUNCS.as_ref().unwrap()};
		
		match self {
			Issue::UiResolution(res) =>
				if match res.to_ascii_lowercase().as_str() {
					"standard" => Some(0),
					"high" => Some(1),
					_ => None
				}.unwrap_or(255) == (funcs.ui_resolution)() {
					Status::Ok
				} else {
					Status::Warning(format!("Ui Resolution is required to be set to {res}.\n\
					                         To fix this open the System Configuration window, Graphics Settings (3rd button), \
					                         and set 'UI Resolution' (2nd option) to {res}. Restart your game after."))
				}
			
			Issue::UiTheme(theme) =>
				if match theme.to_ascii_lowercase().as_str() {
					"dark" => Some(0),
					"light" => Some(1),
					"classic ff" => Some(2),
					"clear blue" => Some(3),
					_ => None
				}.unwrap_or(255) == (funcs.ui_theme)() {
					Status::Ok
				} else {
					Status::Warning(format!("Ui Theme is required to be set to {theme}.\n\
					                         To fix this open the System Configuration window, Theme Settings (6th button), \
					                         and select {theme} from the dropdown. Restart your game after."))
				}
			
			Issue::Collection(collection_type_name) =>
				if super::backend::CollectionType::iter()
					.find(|v| v.to_str().to_ascii_lowercase() == collection_type_name.to_ascii_lowercase())
					.map_or(false, |v| (funcs.collection)(v).is_valid()) {
					Status::Ok
				} else {
					Status::Warning(format!("A collection is required to be assigned to {collection_type_name}.\n\
					                         To fix this open the Penumbra window, click on the 'Collections' tab, and assign \
					                         a collection to {collection_type_name}, possibly create a new collection if needed."))
				}
		}
	}
}

// ----------

pub struct IssueInitializers {
	pub ui_resolution: Box<dyn Fn() -> u8>,
	pub ui_theme: Box<dyn Fn() -> u8>,
	pub collection: Box<dyn Fn(super::backend::CollectionType) -> super::backend::Collection>,
}

static mut FUNCS: Option<IssueInitializers> = None;
pub(crate) fn initialize(funcs: IssueInitializers) {
	unsafe{FUNCS = Some(funcs)}
}