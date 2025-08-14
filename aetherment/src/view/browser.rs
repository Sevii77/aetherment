use std::{collections::VecDeque, ops::{Deref, DerefMut}, sync::{atomic::AtomicBool, Arc, Mutex}};

use crate::{remote::ORIGINS, ui_ext::UiExt};

enum Page {
	Error(crate::remote::Error),
	Home(Vec<crate::remote::HomeResultEntry>),
	Search(crate::remote::SearchResult),
}

enum UserInput {
	None,
	RequiredPick(tempfile::TempDir),
	InstallPath(tempfile::TempDir, std::path::PathBuf),
	DownloadError(String),
}

#[derive(Clone)]
enum DownloadSizeStatus {
	Requesting(String),
	Finished(Option<String>),
	None,
}

pub struct Browser {
	progress: crate::modman::backend::TaskProgress,
	
	loading: Arc<AtomicBool>,
	loading_infscroll: Arc<AtomicBool>,
	selected_origin: String,
	page: Arc<Mutex<Page>>,
	viewed_mod: Arc<Mutex<Option<Result<crate::remote::ModPage, crate::remote::Error>>>>,
	viewed_mod_size: Arc<Mutex<DownloadSizeStatus>>,
	search_options: crate::remote::SearchOptions,
	mod_image_index: usize,
	mod_download_index: usize,
	mod_markdown_cache: egui_commonmark::CommonMarkCache,
	download_queue: Arc<Mutex<VecDeque<(&'static str, String, String, crate::remote::FileType)>>>, // origin, download_url, mod_id
	download_user_input: Arc<Mutex<UserInput>>,
	is_downloading: Arc<AtomicBool>,
}

impl Browser {
	pub fn new(progress: crate::modman::backend::TaskProgress) -> Self {
		let mut selected_origin = crate::config().config.browser_default_origin.clone();
		if !ORIGINS.contains_key(selected_origin.as_str()) {
			selected_origin = "Aetherment".to_string();
		}
		
		let sort_by = ORIGINS[selected_origin.as_str()].search_sort_types().get(0).map_or("", |v| v.1).to_string();
		
		let s = Self {
			loading: Arc::new(AtomicBool::new(false)),
			loading_infscroll: Arc::new(AtomicBool::new(false)),
			selected_origin,
			page: Arc::new(Mutex::new(Page::Home(Vec::new()))),
			viewed_mod: Arc::new(Mutex::new(None)),
			viewed_mod_size: Arc::new(Mutex::new(DownloadSizeStatus::None)),
			search_options: crate::remote::SearchOptions {
				query: String::new(),
				page: 0,
				content_rating: crate::config().config.browser_content_rating,
				sort_by,
				sort_order: crate::remote::SortOrder::Descending,
				extra: Vec::new(),
			},
			mod_image_index: 0,
			mod_download_index: 0,
			mod_markdown_cache: Default::default(),
			download_queue: Arc::new(Mutex::new(VecDeque::new())),
			download_user_input: Arc::new(Mutex::new(UserInput::None)),
			is_downloading: Arc::new(AtomicBool::new(false)),
			
			progress,
		};
		
		s.load_home();
		
		s
	}
	
	fn download_mod(&self, download_url: &str, mod_id: &str, file_type: crate::remote::FileType) {
		let origin_url = ORIGINS[self.selected_origin.as_str()].url();
		let download_url = download_url.to_string();
		let mod_id = mod_id.to_string();
		let queue = self.download_queue.clone();
		queue.lock().unwrap().push_back((origin_url, download_url, mod_id, file_type));
		
		let user_input = self.download_user_input.clone();
		let progress = self.progress.clone();
		let is_downloading = self.is_downloading.clone();
		if is_downloading.load(std::sync::atomic::Ordering::SeqCst) {return}
		is_downloading.store(true, std::sync::atomic::Ordering::SeqCst);
		
		std::thread::spawn(move || {
			while let Some((origin_url, download_url, mod_id, file_type)) = queue.lock().unwrap().pop_front() {
				progress.add_task_count(1);
				progress.set_task_msg(format!("Downloading {mod_id} ({download_url})"));
				
				if let crate::remote::FileType::Other(ext) = &file_type {
					*user_input.lock().unwrap() = UserInput::DownloadError(format!("Unsupported extension '{ext}'"));
				} else {
					match crate::remote::download(origin_url, &download_url, &mod_id, progress.sub_task.clone()) {
						Ok(file) => {
							match file_type {
								crate::remote::FileType::Aetherment |
								crate::remote::FileType::Penumbra =>
									crate::backend().install_mods(progress.clone(), vec![(mod_id, file)]),
								
								crate::remote::FileType::Textools =>
									*user_input.lock().unwrap() = UserInput::DownloadError("TexTools mods are currently unsupported".to_string()),
								
								crate::remote::FileType::Archive =>
									*user_input.lock().unwrap() = UserInput::DownloadError("Archive mods are currently unsupported".to_string()),
								
								crate::remote::FileType::Other(_) => unreachable!()
							}
						}
						
						Err(e) => *user_input.lock().unwrap() = UserInput::DownloadError(e.to_string()),
					}
				}
				
				loop {
					let user_input_lock = user_input.lock().unwrap();
					match user_input_lock.deref() {
						UserInput::None => break,
						
						UserInput::InstallPath(_temp_dir, path) => {
							todo!()
						}
						
						_ => {}
					}
					drop(user_input_lock);
					
					std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
				}
				
				progress.progress_task();
			}
			
			is_downloading.store(false, std::sync::atomic::Ordering::SeqCst)
		});
	}
	
	fn load_home(&self) {
		log!("loading home");
		let loading = self.loading.clone();
		let origin = ORIGINS.get(self.selected_origin.as_str()).unwrap();
		let page = self.page.clone();
		
		loading.store(true, std::sync::atomic::Ordering::Relaxed);
		std::thread::spawn(move || {
			let resp = origin.home();
			match resp {
				Ok(v) => *page.lock().unwrap() = Page::Home(v),
				Err(e) => *page.lock().unwrap() = Page::Error(e),
			}
			
			loading.store(false, std::sync::atomic::Ordering::Relaxed);
		});
	}
	
	fn search(&self, is_infscroll: bool) {
		log!("searching {is_infscroll}");
		let loading = if is_infscroll {self.loading_infscroll.clone()} else {self.loading.clone()};
		let origin = ORIGINS.get(self.selected_origin.as_str()).unwrap();
		let page = self.page.clone();
		let search_options = self.search_options.clone();
		
		loading.store(true, std::sync::atomic::Ordering::Relaxed);
		std::thread::spawn(move || {
			let resp = origin.search(search_options);
			match resp {
				Ok(mut v) => {
					let mut page_lock = page.lock().unwrap();
					if !is_infscroll {
						*page_lock = Page::Search(v)
					} else if let Page::Search(page) = page_lock.deref_mut() {
						page.entries.append(&mut v.entries);
						drop(page_lock);
						
						// simple throttling
						std::thread::sleep(std::time::Duration::from_secs_f32(1.0));
					}
				}
				
				Err(e) => *page.lock().unwrap() = Page::Error(e),
			}
			
			loading.store(false, std::sync::atomic::Ordering::Relaxed);
		});
	}
	
	fn load_mod(&self, mod_id: String) {
		log!("loading mod");
		*self.viewed_mod_size.lock().unwrap() = DownloadSizeStatus::None;
		let loading = self.loading.clone();
		let origin = ORIGINS.get(self.selected_origin.as_str()).unwrap();
		let viewed_mod = self.viewed_mod.clone();
		
		loading.store(true, std::sync::atomic::Ordering::Relaxed);
		std::thread::spawn(move || {
			let resp = origin.mod_page(&mod_id);
			*viewed_mod.lock().unwrap() = Some(resp);
			
			loading.store(false, std::sync::atomic::Ordering::Relaxed);
		});
	}
	
	fn load_size(&self, download_url: String) {
		log!("requesting size {download_url}");
		let mod_size = self.viewed_mod_size.clone();
		*mod_size.lock().unwrap() = DownloadSizeStatus::Requesting(download_url.clone());
		
		std::thread::spawn(move || {
			let size = crate::remote::download_size(&download_url).map(|v| crate::remote::pretty_size(v));
			let mut mod_size = mod_size.lock().unwrap();
			let DownloadSizeStatus::Requesting(url) = mod_size.deref() else {return};
			if *url != download_url {return};
			*mod_size = DownloadSizeStatus::Finished(size);
		});
	}
	
	fn draw_entries<'a>(&'a mut self, ui: &'a mut egui::Ui, entries: &'a[crate::remote::ModEntry], continued: &'a Option<crate::remote::SearchOptions>, search_total_pages: Option<usize>) {
		let content_rating = crate::config().config.browser_content_rating;
		let style = ui.style();
		let column_count = ((ui.available_width() + style.spacing.item_spacing.x) /
			(192.0 + style.spacing.menu_margin.leftf() + style.spacing.menu_margin.rightf() + style.spacing.item_spacing.x))
			.floor().max(1.0) as usize;
		
		let mut index = 0;
		loop {
			let mut exit = false;
			ui.columns(column_count, |uis| {
				let mut column_index = 0;
				while column_index < column_count {
					let ui = &mut uis[column_index];
					
					if index >= entries.len() {
						exit = true;
						
						if let Some(continued) = continued {
							let style = ui.style();
							let resp = egui::Frame::new()
								.fill(style.visuals.extreme_bg_color)
								.corner_radius(style.visuals.menu_corner_radius)
								.inner_margin(style.spacing.menu_margin)
								.show(ui, |ui| {
									let style = ui.style();
									let heading_height = style.text_styles[&egui::TextStyle::Heading].size;
									let height = heading_height +
										style.text_styles[&egui::TextStyle::Body].size +
										style.spacing.item_spacing.y * 2.0 +
										108.0;
									
									let rect = egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(192.0, height));
									let mut ui2 = ui.new_child(egui::UiBuilder::new().max_rect(rect));
									ui2.add_space(height / 2.0 - heading_height / 2.0);
									ui2.vertical_centered_justified(|ui| {
										ui.heading("Continue ⮩");
									});
									ui.allocate_rect(rect, egui::Sense::empty());
								});
							
							if resp.response.interact(egui::Sense::click()).clicked() {
								self.search_options = continued.clone();
								self.search(false);
								free_textures(ui);
							}
						} else if let Some(total_pages) = search_total_pages {
							if self.search_options.page < total_pages - 1 && ui.is_rect_visible(egui::Rect::from_pos(ui.next_widget_position())) && !self.loading_infscroll.load(std::sync::atomic::Ordering::Relaxed) {
								self.search_options.page += 1;
								self.search(true);
							}
						}
						
						return;
					}
					
					let entry = &entries[index];
					if entry.content_rating <= content_rating {
						if draw_mod_entry(ui, entry) {
							self.mod_image_index = 0;
							self.mod_download_index = 0;
							self.load_mod(entry.id.clone());
							free_textures(ui);
						}
						
						column_index += 1;
					}
					
					index += 1;
				}
			});
			
			if exit {
				break
			}
		}
	}
}

impl super::View for Browser {
	fn title(&self) -> &'static str {
		"Browser"
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) {
		// user intervention
		let mut lock = self.download_user_input.lock().unwrap();
		match lock.deref_mut() {
			UserInput::RequiredPick(temp_dir) => {
				
			}
			
			UserInput::DownloadError(err) => {
				if modal(ui, |ui| {
					ui.label(err.to_owned());
				}) {
					*lock = UserInput::None;
				}
			}
			
			_ => {}
		}
		drop(lock);
		
		// page loading spinner
		if self.loading.load(std::sync::atomic::Ordering::Relaxed) {
			ui.centered("loading", crate::ui_ext::Axis::Both, |ui| {
				ui.horizontal(|ui| {
					ui.heading("Loading");
					ui.add(egui::Spinner::new());
				});
			});
			
			return;
		}
		
		let loading_infscroll = self.loading_infscroll.load(std::sync::atomic::Ordering::Relaxed);
		
		// origin selecter
		ui.horizontal(|ui| {
			for k in ORIGINS.keys() {
				if ui.selectable_label(*k == self.selected_origin, *k).clicked() {
					*self.viewed_mod.lock().unwrap() = None;
					self.selected_origin = k.to_string();
					self.search_options.sort_by = ORIGINS[self.selected_origin.as_str()]
						.search_sort_types()
						.get(0)
						.map_or("", |v| v.1)
						.to_string();
					self.load_home();
				}
			}
		});
		
		// selected mod
		let viewed_mod = self.viewed_mod.clone();
		let mut viewed_mod = viewed_mod.lock().unwrap();
		if let Some(entry) = viewed_mod.as_mut() {
			let mut close = false;
			ui.horizontal(|ui| {
				let rtn = ui.button(egui::RichText::new("⮨ Return").heading());
				if rtn.clicked() {
					close = true
				}
				
				if let Ok(entry) = &entry {
					let style = ui.style();
					let clr = style.visuals.text_color();
					let mut text = egui::text::LayoutJob::simple_singleline(format!("{} ({})", entry.name, entry.version), style.text_styles[&egui::TextStyle::Heading].clone(), clr);
					text.append(&format!("By {}", entry.author), 8.0, egui::TextFormat::simple(style.text_styles[&egui::TextStyle::Body].clone(), clr));
					
					ui.vertical_centered(|ui| {
						ui.label(text);
					});
				}
			});
			
			match entry {
				Ok(entry) => {
					egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
						let width = ui.available_width();
						if entry.images.len() > 0 {
							ui.vertical_centered(|ui| {
								ui.add(egui::Image::new(&entry.images[self.mod_image_index])
									.bg_fill(ui.style().visuals.panel_fill)
									.max_size(egui::vec2(width, width / 16.0 * 9.0))
									.fit_to_original_size(100.0));
							});
						}
						
						ui.horizontal(|ui| {
							if entry.images.len() > 1 {
								let style = ui.style();
								ui.add_space(width / 2.0 - style.text_styles[&egui::TextStyle::Heading].size - style.spacing.button_padding.x * 2.0 - style.spacing.item_spacing.x / 2.0);
								if ui.add_enabled(self.mod_image_index > 0, egui::Button::new(egui::RichText::new("⏴").heading())).clicked() {
									self.mod_image_index -= 1;
								}
								
								if ui.add_enabled(self.mod_image_index < entry.images.len() - 1, egui::Button::new(egui::RichText::new("⏵").heading())).clicked() {
									self.mod_image_index += 1;
								}
							}
							
							if entry.download_options.len() > 0 {
								let download_option = &entry.download_options[self.mod_download_index];
								
								ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
									if entry.download_options.len() > 1 {
										ui.menu_button(egui::RichText::new("⏷").heading(), |ui| {
											egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
												for (i, option) in entry.download_options.iter().enumerate() {
													if ui.selectable_label(i == self.mod_download_index, &option.name).clicked() {
														self.mod_download_index = i;
														ui.close_menu();
													}
												}
											});
										});
									}
									
									let mut request = false;
									let size_lock = self.viewed_mod_size.lock().unwrap();
									let size = match size_lock.clone() {
										DownloadSizeStatus::Finished(v) => v,
										DownloadSizeStatus::Requesting(_) => None,
										DownloadSizeStatus::None => {
											if download_option.is_direct {
												request = true;
											}
											
											None
										}
									};
									
									drop(size_lock);
									if request {
										self.load_size(download_option.link.clone());
									}
									
									let download_text = if download_option.is_direct {format!("Download {}", download_option.name)} else {format!("Go to download {} (?)", download_option.name)};
									let download_text = if let Some(size) = size {format!("{download_text} ({})", size)} else {download_text};
									let resp = ui.button(egui::RichText::new(download_text).heading());
									if resp.clicked() {
										println!("{}", download_option.link);
										self.download_mod(&download_option.link, &entry.id, download_option.file_type.clone());
									}
									
									if !download_option.is_direct {
										resp.on_hover_text(format!("{}\nAetherment is unable to download non direct files automatically.\nYou'll be directed towards the download in your webbrowser.", download_option.link));
									} else {
										resp.on_hover_text(&download_option.link);
									}
								});
							}
							
							ui.spacer();
						});
						
						if entry.description_format == crate::remote::TextFormatting::Markdown {
							ui.userspace_loaders(|ui| {
								egui_commonmark::CommonMarkViewer::new()
									.explicit_image_uri_scheme(true)
									.show(ui, &mut self.mod_markdown_cache, &entry.description);
							});
						} else {
							ui.label(&entry.description);
						}
					});
				}
				
				Err(e) => {
					ui.label(format!("{e:#?}"));
				}
			}
			
			if close {
				*viewed_mod = None;
				self.mod_markdown_cache = Default::default();
				ui.free_textures("http://");
				ui.free_textures("https://");
			}
			
			return
		}
		
		// search bar
		ui.horizontal(|ui| {
			let sort_types = ORIGINS[self.selected_origin.as_str()].search_sort_types();
			if sort_types.len() > 0 {
				let preview = sort_types.iter().find(|v| v.1 == self.search_options.sort_by).map_or("Unknown", |v| v.0);
				ui.combo(preview, "", |ui| {
					for (name, id) in sort_types {
						if ui.selectable_label(self.search_options.sort_by == *id, *name).clicked() {
							self.search_options.sort_by = id.to_string();
						}
					}
				});
				
				ui.combo_enum_id(&mut self.search_options.sort_order, "order");
			}
			
			// ui.set_width(ui.available_width());
			ui.text_edit_singleline(&mut self.search_options.query);
			
			if ui.add_enabled(!loading_infscroll, egui::Button::new("Search 🔎")).clicked() {
				self.search_options.content_rating = crate::config().config.browser_content_rating;
				self.search_options.page = 0;
				self.search(false);
				ui.free_textures("http://");
				ui.free_textures("https://");
			}
		});
		
		// origin mods list
		match self.page.clone().lock().unwrap().deref() {
			Page::Home(categories) => {
				for category in categories {
					if category.entries.len() > 0 {
						ui.add_space(32.0);
						ui.heading(&category.name);
						ui.spacer();
						self.draw_entries(ui, &category.entries, &category.continued, None);
					}
				}
			}
			
			Page::Search(result) => {
				ui.add_space(32.0);
				ui.heading(if result.query.is_empty() {"Results".to_string()} else {format!("Results for '{}'", result.query)});
				ui.spacer();
				self.draw_entries(ui, &result.entries, &None, Some(result.total_pages));
				
				if loading_infscroll {
					ui.add_space(32.0);
					ui.centered("loading", crate::ui_ext::Axis::Horizontal, |ui| {
						ui.horizontal(|ui| {
							ui.heading("Loading");
							ui.add(egui::Spinner::new());
						});
					});
					ui.add_space(32.0);
				} else if self.search_options.page >= result.total_pages - 1 {
					ui.add_space(32.0);
					ui.centered("loading", crate::ui_ext::Axis::Horizontal, |ui| {
						ui.heading("You've reached the end");
					});
					ui.add_space(32.0);
				}
			}
			
			Page::Error(e) => {
				ui.label(format!("{e:#?}"));
			}
		}
	}
}

fn modal(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> bool {
	let mut close = false;
	egui::Modal::new(egui::Id::new("dialogresult")).show(ui.ctx(), |ui| {
		ui.set_max_width(600.0);
		content(ui);
		
		ui.vertical_centered(|ui| {
			ui.add_space(32.0);
			if ui.button("Close").clicked() {
				close = true;
			}
		});
	});
	
	close
}

fn draw_mod_entry(ui: &mut egui::Ui, entry: &crate::remote::ModEntry) -> bool {
	let style = ui.style();
	let resp = egui::Frame::new()
		.fill(style.visuals.extreme_bg_color)
		.corner_radius(style.visuals.menu_corner_radius)
		.inner_margin(style.spacing.menu_margin)
		.show(ui, |ui| {
			ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
			ui.set_max_width(192.0);
			ui.heading(&entry.name);
			
			ui.vertical_centered(|ui| {
				// this way if you return from a modpage to a longass search result, only the visible images will be reloaded
				if ui.is_rect_visible(egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(192.0, 108.0))) {
					ui.add(egui::Image::new(&entry.thumbnail_url)
						.bg_fill(ui.style().visuals.panel_fill)
						.max_size(egui::vec2(192.0, 108.0))
						.fit_to_original_size(10.0));
				} else {
					ui.allocate_exact_size(egui::vec2(192.0, 108.0), egui::Sense::empty());
				}
			});
			
			ui.horizontal(|ui| {
				ui.label(format!("By {}", entry.author));
				
				ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
					if entry.content_rating == crate::remote::ContentRating::Nsfw {
						ui.label("NSFW");
					} else if entry.content_rating == crate::remote::ContentRating::Nsfl {
						ui.label("NSFL");
					}
				});
			});
		});
	
	resp.response.interact(egui::Sense::click()).clicked()
}

fn free_textures(ui: &mut egui::Ui) {
	ui.free_textures("http://");
	ui.free_textures("https://");
}