use super::super::*;

const REMOTE_URL: &'static str = "https://aetherment.sevii.dev";
// const REMOTE_URL: &'static str = "http://127.0.0.1:3000";

#[derive(Deserialize)]
struct RemoteModEntry {
	name: String,
	author: String,
	description: String,
	id: String,
	versions: Vec<String>,
}

pub struct Aetherment;

impl Aetherment {
	pub fn new() -> Self {
		Self
	}
}

impl RemoteOrigin for Aetherment {
	fn name(&self) -> &'static str {
		"Aetherment"
	}
	
	fn url(&self) -> &'static str {
		REMOTE_URL
	}
	
	fn disclaimer(&self) -> &'static str {
		""
	}
	
	fn default_auto_update(&self) -> bool {
		true
	}
	
	fn search(&self, options: SearchOptions) -> Result<SearchResult, Error> {
		let query = options.query.to_ascii_lowercase();
		Ok(SearchResult {
			entries: ureq::get(&format!("{REMOTE_URL}/mods"))
				.call()?
				.into_body()
				.read_json::<Vec<RemoteModEntry>>()?
				.into_iter()
				.filter(|v| v.name.to_ascii_lowercase().contains(&query))
				.map(|v| ModEntry {
					name: v.name,
					author: v.author,
					id: v.id,
					thumbnail_url: String::new(),
					content_rating: ContentRating::Sfw,
				})
				.collect(),
			query: options.query,
			// page: 0,
			total_pages: 1,
		})
	}
	
	fn search_sort_types(&self) -> &'static [(&'static str, &'static str)] {
		&[]
	}
	
	fn home(&self) -> Result<Vec<HomeResultEntry>, Error> {
		Ok(vec![
			HomeResultEntry {
				name: "Mods".to_string(),
				continued: None,
				entries: ureq::get(&format!("{REMOTE_URL}/mods"))
					.call()?
					.into_body()
					.read_json::<Vec<RemoteModEntry>>()?
					.into_iter()
					.map(|v| ModEntry {
						name: v.name,
						author: v.author,
						id: v.id,
						thumbnail_url: String::new(),
						content_rating: ContentRating::Sfw,
					})
					.collect()
			}
		])
	}
	
	fn mod_page(&self, mod_id: &str) -> Result<ModPage, Error> {
		let mod_entry = ureq::get(&format!("{REMOTE_URL}/mods"))
			.call()?
			.into_body()
			.read_json::<Vec<RemoteModEntry>>()?
			.into_iter()
			.find(|v| v.id == mod_id)
			.ok_or(Error::InvalidMod("Mod does not exist".to_string()))?;
		
		Ok(ModPage {
			name: mod_entry.name,
			description: mod_entry.description,
			description_format: TextFormatting::Text,
			author: mod_entry.author,
			id: mod_entry.id,
			version: mod_entry.versions.get(0)
				.ok_or(Error::InvalidMod("Mod has no versions".to_string()))?
				.to_string(),
			download_options: mod_entry.versions
				.into_iter()
				.map(|version| DownloadOption {
					is_direct: true,
					link: format!("{REMOTE_URL}/mod/{mod_id}/{version}"),
					name: version,
					file_type: FileType::Aetherment,
				}).collect(),
			images: Vec::new(),
			content_rating: ContentRating::Sfw,
			tags: Vec::new(),
		})
	}
}