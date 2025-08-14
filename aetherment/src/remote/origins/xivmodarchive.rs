use std::collections::HashMap;
use super::super::*;

const REMOTE_URL: &'static str = "https://www.xivmodarchive.com";

#[derive(Debug, Deserialize)]
struct SearchRow {
	search_name: String,
	results: SearchResults,
}

#[derive(Debug, Deserialize)]
struct SearchResults {
	// page: usize,
	total_pages: usize,
	items: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize)]
struct SearchEntry {
	id: String,
	author: Author,
	name: String,
	thumbnail: String,
	nsfw: bool,
	nsfl: bool,
}

#[derive(Debug, Deserialize)]
struct Author {
	display_name: String,
}

pub struct XivMa;

impl XivMa {
	pub fn new() -> Self {
		Self
	}
}

impl RemoteOrigin for XivMa {
	fn name(&self) -> &'static str {
		"XIV Mod Archive"
	}
	
	fn url(&self) -> &'static str {
		REMOTE_URL
	}
	
	fn search(&self, options: SearchOptions) -> Result<SearchResult, Error> {
		#[derive(Debug, Deserialize)]
		struct Result {
			#[serde(rename = "searchSettings")]
			search: SearchRow,
		}
		
		let mut query = options.extra;
		if options.content_rating == ContentRating::Sfw {
			query.push(("nsfw".to_string(), "false".to_string()));
		}
		query.push(("sortby".to_string(), options.sort_by));
		query.push(("sortorder".to_string(), if options.sort_order == SortOrder::Ascending {"asc"} else {"desc"}.to_string()));
		query.push(("basic_text".to_string(), options.query.clone()));
		query.push(("page".to_string(), (options.page + 1).to_string()));
		query.push(("dt_compat".to_string(), "1".to_string()));
		query.push(("types".to_string(), "1,3,7,9,12,15,2,4,8,10,14,16,6".to_string()));
		query.push(("json".to_string(), "true".to_string()));
		
		// println!("{query:#?}");
		
		let result = ureq::get(&format!("{REMOTE_URL}/search"))
			.query_pairs(query)
			.call()?
			.into_body()
			// .read_json::<serde_json::Value>();
			.read_json::<Result>()?
			.search
			.results;
		
		let entries = result
			.items
			.into_iter()
			.map(|v| ModEntry {
				name: v.name,
				author: v.author.display_name,
				id: v.id,
				thumbnail_url: v.thumbnail,
				content_rating: if v.nsfl {ContentRating::Nsfl}
					else if v.nsfw {ContentRating::Nsfw}
					else {ContentRating::Sfw},
			})
			.collect::<Vec<_>>();
		
		Ok(SearchResult {
			entries,
			query: options.query,
			// page: result.page,
			total_pages: result.total_pages,
		})
	}
	
	fn search_sort_types(&self) -> &'static [(&'static str, &'static str)] {
		&[
			("Relevance", "rank"),
			("Last Update", "time_edited"),
			("Release Date", "time_published"),
			("Name", "name_slug"),
			("Views", "views"),
			("Views Today", "views_today"),
			("Downloads", "downloads"),
			("Followers", "followers"),
		]
	}
	
	fn home(&self) -> Result<Vec<HomeResultEntry>, Error> {
		#[derive(Deserialize)]
		struct Result {
			#[serde(rename = "searchRows")]
			search_rows: Vec<SearchRow>,
		}
		
		let mut results = ureq::get(&format!("{REMOTE_URL}/?json=true"))
			.call()?
			.into_body()
			.read_json::<Result>()?
			.search_rows
			.into_iter()
			.map(|v| (
				v.search_name,
				v.results.items
					.into_iter()
					.map(|v| ModEntry {
						name: v.name,
						author: v.author.display_name,
						id: v.id,
						thumbnail_url: v.thumbnail,
						content_rating: if v.nsfl {ContentRating::Nsfl}
							else if v.nsfw {ContentRating::Nsfw}
							else {ContentRating::Sfw},
					})
					.collect::<Vec<_>>()
				))
			.collect::<HashMap<_, _>>();
		
		Ok(vec![
			HomeResultEntry {
				name: "Popular".to_string(),
				continued: Some(SearchOptions {
					query: String::new(),
					page: 0,
					content_rating: ContentRating::Sfw, // will get overwritten by user setting
					sort_by: "views_today".to_string(),
					sort_order: SortOrder::Descending,
					extra: Vec::new(),
				}),
				entries: results.remove_entry("Today's Most Viewed Mods")
					.ok_or(Error::Network2("Result did not contain key 'Today's Most Viewed Mods'".to_string()))?.1,
			},
			HomeResultEntry {
				name: "New".to_string(),
				continued: Some(SearchOptions {
					query: String::new(),
					page: 0,
					content_rating: ContentRating::Sfw,
					sort_by: "time_published".to_string(),
					sort_order: SortOrder::Descending,
					extra: Vec::new(),
				}),
				entries: results.remove_entry("Newest Mods from All Users")
					.ok_or(Error::Network2("Result did not contain key 'Newest Mods from All Users'".to_string()))?.1,
			},
			HomeResultEntry {
				name: "New and updated (Patreon)".to_string(),
				continued: Some(SearchOptions {
					query: String::new(),
					page: 0,
					content_rating: ContentRating::Sfw,
					sort_by: "time_edited".to_string(),
					sort_order: SortOrder::Descending,
					extra: vec![
						("sponsored".to_string(), "true".to_string()),
					],
				}),
				entries: results.remove_entry("New and Updated from our Patreon Subscribers")
					.ok_or(Error::Network2("Result did not contain key 'New and Updated from our Patreon Subscribers'".to_string()))?.1,
			},
		])
	}
	
	fn mod_page(&self, mod_id: &str) -> Result<ModPage, Error> {
		// since we do funky stuff with the save name, we need to undo that for checking for updates
		let pos = mod_id.rfind('_');
		let mod_id = if let Some(pos) = pos {&mod_id[pos + 1..]} else {mod_id};
		
		#[derive(Deserialize)]
		struct Result {
			r#mod: Mod,
		}
		
		#[derive(Deserialize)]
		struct Mod {
			id: i32,
			author: Author,
			name: String,
			nsfw: bool,
			nsfl: bool,
			version: String,
			images: HashMap<String, Image>,
			primary_download: Link,
			comments: String,
			contributor_comments: String,
			tags: Vec<String>,
			races: Vec<String>,
			genders: i32,
		}
		
		#[derive(Deserialize)]
		struct Link {
			link: String,
		}
		
		#[derive(Deserialize)]
		struct Image {
			url: String,
			order: i32,
		}
		
		let mod_entry = ureq::get(&format!("{REMOTE_URL}/modid/{mod_id}?json=true"))
			.call()?
			.into_body()
			.read_json::<Result>()?
			.r#mod;
		
		Ok(ModPage {
			name: mod_entry.name.clone(),
			description: {
				let contrib = mod_entry.contributor_comments.trim();
				let desc = if contrib.is_empty() {mod_entry.comments} else {format!("{}\n\n### Contributors\n{}", mod_entry.comments, mod_entry.contributor_comments)}
					.replace("\n", "  \n");
				
				// replace user mentions
				let desc = regex::Regex::new(r"<@:([^:]+):\d+>").unwrap()
					.replace_all(&desc, "$1");
				
				// replace links with href
				let desc = regex::Regex::new(r"(https?://\S+)").unwrap()
					.replace_all(&desc, "[$1]($1)");
				
				desc.to_string()
			},
			description_format: TextFormatting::Markdown,
			author: mod_entry.author.display_name,
			id: {
				// xivma id is just numerical, and since we only allow ascii, some names might end up empty, so we combine them.
				let mut name = mod_entry.name;
				name.retain(|v| v.is_ascii_alphanumeric() || v == ' ' || v == '_');
				format!("{}_{}", name, mod_entry.id.to_string())
			},
			version: mod_entry.version.clone(),
			download_options: vec![{
				let link = if mod_entry.primary_download.link.starts_with("/modid/") {format!("{REMOTE_URL}{}", mod_entry.primary_download.link)} else {mod_entry.primary_download.link};
				let file_type = FileType::from_path(&link);
				DownloadOption {
					is_direct: !matches!(file_type, FileType::Other(_)),
					file_type,
					link,
					name: mod_entry.version,
				}}],
			images: {
				let mut images = mod_entry.images.into_values().collect::<Vec<_>>();
				images.sort_by(|a, b| a.order.cmp(&b.order));
				images.into_iter()
					.map(|v| v.url)
					.collect()
			},
			content_rating: if mod_entry.nsfl {ContentRating::Nsfl}
				else if mod_entry.nsfw {ContentRating::Nsfw}
				else {ContentRating::Sfw},
			tags: {
				let mut tags = mod_entry.tags;
				tags.extend_from_slice(&mod_entry.races);
				if mod_entry.genders == 1 {
					tags.push("Male".to_string());
				} else if mod_entry.genders == 2 {
					tags.push("Female".to_string());
				} // 3 is unisex. we ignore it since mounts and minions are also tagged as such
				tags
			},
		})
	}
}