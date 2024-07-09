// TODO: plugin system w/e (using lua) so that people can make their own for different sites

use std::{collections::HashMap, io};
use serde::Deserialize;

pub mod settings;

const REMOTE_URL: &'static str = "https://aetherment.sevii.dev";
// const REMOTE_URL: &'static str = "http://127.0.0.1:3000";

#[derive(Deserialize)]
pub struct ModEntry {
	pub name: String,
	pub author: String,
	pub description: String,
	pub id: String,
	pub versions: Vec<String>,
}

pub fn get_mods() -> Result<Vec<ModEntry>, crate::resource_loader::BacktraceError> {
	Ok(ureq::get(&format!("{REMOTE_URL}/mods"))
		.call()?
		.into_json::<Vec<ModEntry>>()?)
}

pub fn download(mod_id: &str, version: &str) -> Result<std::fs::File, crate::resource_loader::BacktraceError> {
	let mut s = crate::remote::settings::Settings::open(&mod_id);
	s.origin = REMOTE_URL.to_string(); // not actually used atm, used for future proofing
	s.save(&mod_id);
	
	let mut f = tempfile::tempfile()?;
	let mut data = ureq::get(&format!("{REMOTE_URL}/mod/{mod_id}/{version}"))
		.call()?
		.into_reader();
	io::copy(&mut data, &mut f)?;
	Ok(f)
}

fn check_update(entry: &ModEntry, cur_version: &str) -> Result<Option<std::fs::File>, crate::resource_loader::BacktraceError> {
	let latest = entry.versions.get(0).ok_or("No versions exist")?;
	if latest != cur_version {
		Ok(Some(download(&entry.id, &latest)?))
	} else {
		Ok(None)
	}
}

pub fn check_updates(progress: crate::modman::backend::InstallProgress) {
	let Ok(mods) = get_mods() else {return};
	
	let backend = crate::backend();
	let allowed = backend.get_mods().into_iter().filter_map(|mod_id| {
		match (settings::Settings::open(&mod_id).auto_update, backend.get_mod_meta(&mod_id)) {
			(true, Some(meta)) => Some((mod_id, meta.version.clone())),
			_ => None
		}
	}).collect::<HashMap<_, _>>();
	
	let mut files = Vec::new();
	for m in mods {
		if let Some(cur_version) = allowed.get(m.id.as_str()) {
			match check_update(&m, cur_version) {
				Ok(f) => if let Some(f) = f {files.push((m.id, f))},
				Err(err) => log!("Failed updating {err:?}"),
			}
		}
	}
	
	crate::backend().install_mods(progress, files);
}