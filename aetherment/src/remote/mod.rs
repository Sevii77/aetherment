use std::{collections::BTreeMap, io::{BufWriter, Read, Write}};
use serde::{Deserialize, Serialize};

pub mod settings;
mod origins {
	pub mod aetherment;
	pub mod xivmodarchive;
}

pub static ORIGINS: std::sync::LazyLock<&'static Origins> = std::sync::LazyLock::new(|| create_origins());
pub struct Origins(BTreeMap<&'static str, Box<dyn RemoteOrigin + Send + Sync>>);

impl std::ops::Deref for Origins {
	type Target = BTreeMap<&'static str, Box<dyn RemoteOrigin + Send + Sync>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

fn create_origins() -> &'static Origins {
	let origins = BTreeMap::from([
		{
			let origin = origins::aetherment::Aetherment::new();
			let name = origin.name();
			let origin: Box<dyn RemoteOrigin + Send + Sync> = Box::new(origin);
			(name, origin)
		},
		{
			let origin = origins::xivmodarchive::XivMa::new();
			(origin.name(), Box::new(origin))
		},
	]);
	
	Box::leak(Box::new(Origins(origins)))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Network error {0:?}")]
	Network(#[from] ureq::Error),
	
	#[error("Network error: {0:?}")]
	Network2(String),
	
	#[error("The searched for mod is invalid; reason: {0:?}")]
	InvalidMod(String),
}

pub trait RemoteOrigin {
	fn name(&self) -> &'static str;
	fn url(&self) -> &'static str;
	fn disclaimer(&self) -> &'static str;
	fn default_auto_update(&self) -> bool;
	fn search(&self, options: SearchOptions) -> Result<SearchResult, Error>;
	fn search_sort_types(&self) -> &'static [(&'static str, &'static str)];
	fn home(&self) -> Result<Vec<HomeResultEntry>, Error>;
	fn mod_page(&self, mod_id: &str) -> Result<ModPage, Error>;
}

#[derive(Clone)]
pub struct HomeResultEntry {
	pub name: String,
	pub continued: Option<SearchOptions>,
	pub entries: Vec<ModEntry>,
}

#[derive(Clone)]
pub struct SearchResult {
	pub entries: Vec<ModEntry>,
	pub query: String,
	// pub page: usize,
	pub total_pages: usize,
}

#[derive(Clone)]
pub struct SearchOptions {
	pub query: String,
	pub page: usize,
	pub content_rating: ContentRating,
	pub sort_by: String,
	pub sort_order: SortOrder,
	pub extra: Vec<(String, String)>,
}

#[derive(Clone)]
pub struct ModEntry {
	pub name: String,
	pub author: String,
	pub id: String,
	pub thumbnail_url: String,
	pub content_rating: ContentRating,
}

#[derive(Clone)]
pub struct ModPage {
	pub name: String,
	pub description: String,
	pub description_format: TextFormatting,
	pub author: String,
	pub id: String,
	pub download_options: Vec<DownloadOption>,
	pub images: Vec<String>,
	pub content_rating: ContentRating,
	pub tags: Vec<String>,
	pub version: String,
}

#[derive(Clone)]
pub struct DownloadOption {
	pub name: String,
	pub link: String,
	pub is_direct: bool,
	pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
	Aetherment,
	Penumbra,
	Textools,
	Archive,
	Other(String),
}

impl FileType {
	pub fn from_path(path: &str) -> Self {
		let pos = path.rfind('.').map_or(0, |v| v + 1);
		let ext = &path[pos..];
		
		match ext.to_ascii_lowercase().as_str() {
			"aeth" => Self::Aetherment,
			"pmp" => Self::Penumbra,
			"ttmp" | "ttmp2" => Self::Textools, // are these the extensions? i honestly havent seen one in ages
			"7z" | "zip" | "rar" => Self::Archive,
			_ => Self::Other(ext.to_string()),
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub enum ContentRating {
	Sfw = 0,
	Nsfw = 1,
	Nsfl = 2,
}

impl crate::EnumTools for ContentRating {
	type Iterator = std::array::IntoIter<Self, 3>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Sfw => "Sfw",
			Self::Nsfw => "Nsfw",
			Self::Nsfl => "Nsfl",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Sfw,
			Self::Nsfw,
			Self::Nsfl,
		].into_iter()
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SortOrder {
	Descending,
	Ascending,
}

impl crate::EnumTools for SortOrder {
	type Iterator = std::array::IntoIter<Self, 2>;
	
	fn to_str(&self) -> &'static str {
		match self {
			Self::Descending => "Descending",
			Self::Ascending => "Ascending",
		}
	}
	
	fn iter() -> Self::Iterator {
		[
			Self::Descending,
			Self::Ascending,
		].into_iter()
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TextFormatting {
	Text,
	Markdown,
}

pub fn download(origin_url: &str, download_url: &str, mod_id: &str, progress: crate::modman::backend::Progress) -> Result<std::fs::File, crate::resource_loader::BacktraceError> {
	progress.set_msg("");
	
	let Some((_, origin)) = ORIGINS.iter().find(|(_, v)| v.url() == origin_url) else {return Err("Invalid origin".into())};
	let mut s = crate::remote::settings::Settings::open(mod_id);
	s.origin = origin_url.to_string();
	s.auto_update = origin.default_auto_update();
	s.save(mod_id);
	
	let resp = ureq::get(download_url)
		.call()?;
	
	let size = 's: {
		let Some(c) = resp.headers().get("Content-Length") else {break 's 0};
		let Ok(s) = c.to_str() else {break 's 0};
		s.parse::<u64>().unwrap_or(0)
	};
	
	let mut writer = BufWriter::new(tempfile::tempfile()?);
	let mut reader = resp
		.into_body()
		.into_reader();
	
	let mut buf = [0u8; 16384];
	let mut total_read = 0;
	loop {
		let readcount = reader.read(&mut buf)?;
		if readcount == 0 {break}
		
		writer.write_all(&buf[..readcount])?;
		
		total_read += readcount;
		progress.set(total_read as f32  / size as f32);
	}
	
	Ok(writer.into_inner()?)
}

pub fn download_size(download_url: &str) -> Option<u64> {
	ureq::get(download_url)
		.call().ok()?
		.headers()
		.get("Content-Length")?
		.to_str().ok()?
		.parse::<u64>().ok()
}

pub fn pretty_size(size: u64) -> String {
	const NAMES: [&'static str; 4] = ["B", "KB", "MB", "GB"];
	
	for i in (0..4).rev() {
		let mag = 1024u64.pow(i as u32);
		if size >= mag {
			return format!("{} {}", ((size as f32 / mag as f32 * 10.0).round() / 10.0), NAMES[i])
		}
	}
	
	size.to_string()
}

pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
	let a = a.split('.').collect::<Vec<_>>();
	let b = b.split('.').collect::<Vec<_>>();
	
	for (a, b) in a.iter().zip(b.iter()) {
		let Ok(a_num) = a.parse::<u32>() else {return std::cmp::Ordering::Less};
		let Ok(b_num) = b.parse::<u32>() else {return std::cmp::Ordering::Greater};
		let cmp = a_num.cmp(&b_num);
		if cmp != std::cmp::Ordering::Equal {
			return cmp;
		}
	}
	
	a.len().cmp(&b.len())
}

pub fn check_updates(progress: crate::modman::backend::TaskProgress) {
	progress.set_task_msg("Checking for updates");
	
	let mut files = Vec::new();
	let mods = crate::backend().get_mods();
	progress.add_task_count(mods.len());
	for mod_id in mods {
		progress.set_task_msg(format!("Checking for updates for '{mod_id}'"));
		progress.sub_task.set(0.0);
		
		crate::set_notification(progress.get_task_progress(), 0, "Checking for updates");
		
		'u: {
			if !settings::Settings::exists(&mod_id) {break 'u}
			let remote_settings = settings::Settings::open(&mod_id);
			if !remote_settings.auto_update {break 'u}
			let Some(meta) = crate::backend().get_mod_meta(&mod_id) else {break 'u};
			let Some(origin) = ORIGINS.iter().find(|(_, v)| v.url() == remote_settings.origin.as_str()) else {break 'u};
			let origin = origin.1;
			let origin_url = origin.url();
			std::thread::sleep(std::time::Duration::from_secs(1));
			let Ok(mod_page) = origin.mod_page(&mod_id) else {break 'u};
			// if mod_page.version == meta.version {break 'u}
			if compare_versions(&mod_page.version, &meta.version) != std::cmp::Ordering::Greater {break 'u}
			let Some(download_entry) = mod_page.download_options.first() else {break 'u};
			if !download_entry.is_direct {break 'u}
			log!("updating {mod_id}");
			let Ok(file) = download(origin_url, &download_entry.link, &mod_id, progress.sub_task.clone()) else {break 'u};
			files.push((mod_id.to_string(), file));
		}
		
		progress.progress_task();
	}
	
	let files_len = files.len();
	if files_len > 0 {
		crate::set_notification(1.0, 0, "Installing updates");
		crate::backend().install_mods(progress.clone(), files);
		if progress.get_messages().iter().any(|v| v.1) {
			crate::set_notification(1.0, 2, "There were issues updating mods");
		} else {
			crate::set_notification(1.0, 1, &format!("{} mods have been successfully updated", files_len));
		}
	} else {
		crate::set_notification(1.0, 1, "No updates found");
	}
}