#![allow(dead_code)]
// #![allow(ambiguous_glob_reexports)]
#![allow(unused_imports)]

use std::hash::Hash;

mod ui {
	#[cfg(feature = "egui")]
	pub mod egui;

	#[cfg(feature = "imgui")]
	pub mod imgui;
}

#[cfg(feature = "egui")]
pub use ui::egui::*;

#[cfg(feature = "imgui")]
pub use ui::imgui::*;

pub struct Response {
	pub clicked: bool,
	pub double_clicked: bool,
	pub changed: bool,
	pub held: bool,
	pub hovered: bool,
}

impl Response {
	pub fn new() -> Self {
		Self {
			clicked: false,
			double_clicked: false,
			changed: false,
			held: false,
			hovered: false,
		}
	}
	
	pub fn union(&self, other: &Self) -> Self {
		Self {
			clicked: self.clicked | other.clicked,
			double_clicked: self.double_clicked | self.double_clicked,
			changed: self.changed | other.changed,
			held: self.held | other.held,
			hovered: self.hovered | other.hovered,
		}
	}
}

pub struct Modifiers {
	pub alt: bool,
	pub ctrl: bool,
	pub shift: bool,
}

pub enum Icon {
	Dir,
	DirOpen,
	File,
}

impl std::fmt::Display for Icon {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.str())
	}
}

pub struct WindowArgs<'a> {
	pub title: &'a str,
	pub open: Option<&'a mut bool>,
	pub pos: Arg<[f32; 2]>,
	pub size: Arg<[f32; 2]>,
	pub min_size: [f32; 2],
	pub max_size: [f32; 2],
}

impl<'a> Default for WindowArgs<'a> {
	fn default() -> Self {
		Self {
			title: "Window",
			open: None,
			pos: Arg::Once([50.0; 2]),
			size: Arg::Once([200.0; 2]),
			min_size: [50.0; 2],
			max_size: [99999.0; 2],
		}
	}
}

pub enum Arg<T> {
	Once(T),
	Always(T),
}

// ----------

impl<'a> Ui<'a> {
	pub fn selectable_value<S: AsRef<str>, V: PartialEq>(&mut self, label: S, current: &mut V, value: V) -> Response {
		let resp = self.selectable(label, *current == value);
		if resp.clicked {
			*current = value;
		}
		
		resp
	}
	
	pub fn tabs<S: AsRef<str> + PartialEq + Clone>(&mut self, tabs: &[S], current: &mut S) -> Response {
		let mut resp = Response::new();
		self.horizontal(|ui| {
			for tab in tabs {
				let r = ui.selectable(tab.as_ref(), *current == *tab);
				if r.clicked {
					*current = tab.clone();
				}
				resp = resp.union(&r);
			}
		});
		
		resp
	}
}

// ----------

#[derive(Debug)]
pub enum FilePickerStatus {
	Success(std::path::PathBuf, Vec<std::path::PathBuf>),
	Busy,
	Canceled(std::path::PathBuf),
}

#[derive(Debug, PartialEq)]
pub enum FilePickerMode {
	OpenFile,
	OpenFileMultiple,
	OpenDirectory,
	OpenDirectoryMultiple,
	Save
}

impl FilePickerMode {
	fn is_file(&self) -> bool {
		match self {
			Self::OpenFile | Self::OpenFileMultiple | Self::Save => true,
			_ => false,
		}
	}
	
	fn is_multiple(&self) -> bool {
		match self {
			Self::OpenFileMultiple | Self::OpenDirectoryMultiple => true,
			_ => false,
		}
	}
}

enum FilePickerSort {
	Name,
	Size,
	Modified,
}

struct FileEntry {
	is_file: bool,
	name: String,
	size: u64,
	modified: std::time::SystemTime, // todo use time type
}

pub struct FilePicker {
	title: String,
	dir: std::path::PathBuf,
	selected: Vec<String>,
	list: Vec<FileEntry>,
	sort_method: FilePickerSort,
	
	ext_filters: Vec<String>,
	mode: FilePickerMode,
	
	quicks: Vec<(&'static str, std::path::PathBuf)>,
	date_formatter: Vec<time::format_description::BorrowedFormatItem<'static>>,
}

// TODO: improve
impl FilePicker {
	pub fn new(title: &str, dir: &std::path::Path, ext_filters: &[&str], mode: FilePickerMode) -> Self {
		let mut s = Self {
			title: title.to_string(),
			dir: dir.to_owned(),
			selected: Vec::new(),
			list: Vec::new(),
			sort_method: FilePickerSort::Name,
			
			ext_filters: ext_filters.into_iter().map(|v| v.to_string()).collect(),
			mode,
			
			quicks: vec![
				("Home", dirs::home_dir()),
				("Desktop", dirs::desktop_dir()),
				("Documents", dirs::document_dir()),
				("Downloads", dirs::download_dir()),
				("Pictures", dirs::picture_dir()),
				("Videos", dirs::video_dir()),
			].into_iter().filter_map(|(a, b)| b.map(|v| (a, v))).collect(),
			date_formatter: time::format_description::parse_borrowed::<1>("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap(),
		};
		
		s.set_dir(dir);
		
		s
	}
	
	pub fn show(&mut self, ui: &mut Ui) -> FilePickerStatus {
		let width = 600.0;
		let height = 400.0;
		let mut r = None;
		let mut open = true;
		ui.window(WindowArgs {
			title: &self.title.clone(),
			open: Some(&mut open),
			size: Arg::Once([width, height]),
			..Default::default()
		}, |ui| {
			let mut path = self.dir.to_string_lossy().to_string();
			
			// current path
			ui.horizontal(|ui| {
				if ui.button("^").clicked {
					if let Some(parent) = self.dir.parent() {
						self.set_dir(&parent.to_owned());
						return;
					}
				}
				
				// let w = ui.available_size()[0];
				ui.set_width(width);
				if ui.input_text("", &mut path).changed {
					let dir = std::path::Path::new(&*path);
					self.dir = dir.to_owned();
					self.set_dir(dir);
				}
			});
			
			// contents
			ui.child("contents", [width, height], |ui| {
				ui.splitter("contents", 0.2, |ui_left, ui_right| {
					// quick access
					for (name, dir) in &self.quicks {
						if ui_left.selectable(name, self.dir == *dir).clicked {
							self.set_dir(&dir.clone());
							break;
						}
					}
					ui_left.mark_next_splitter();
					
					// current directory
					let size = ui_right.available_size();
					ui_right.child("current_dir", size, |ui| {
						for entry in &self.list {
							let selected = self.selected.contains(&entry.name);
							
							let w = ui.available_size()[0];
							ui.set_width(w);
							let resp = ui.selectable(format!("{} {}", if entry.is_file {Icon::File} else {Icon::DirOpen}, entry.name), selected);
							// let resp = if entry.is_file {
							// 	ui.horizontal(|ui| {
							// 		let w = ui.available_size()[0];
							// 		ui.set_width(w * 0.5);
							// 		let mut resp = ui.selectable(format!("{} {}", Icon::File, entry.name), selected);
							// 		
							// 		ui.set_width(w * 0.2);
							// 		resp = resp.union(&ui.selectable(format!("{}B", entry.size), selected));
							// 		
							// 		let w = ui.available_size()[0];
							// 		ui.set_width(w);
							// 		resp.union(&ui.selectable(time::OffsetDateTime::from(entry.modified).format(&self.date_formatter).unwrap(), selected))
							// 	})
							// } else {
							// 	let w = ui.available_size()[0];
							// 	ui.set_width(w);
							// 	ui.selectable(format!("{} {}", Icon::DirOpen, entry.name), selected)
							// };
							
							if resp.double_clicked && !entry.is_file {
								self.set_dir(&self.dir.join(&entry.name));
								return;
							} if resp.clicked && ((entry.is_file && self.mode.is_file()) || (!entry.is_file && !self.mode.is_file())) {
								if !ui.modifiers().shift || !self.mode.is_multiple() {
									self.selected.clear();
								}
								
								if selected {
									self.selected.retain(|v| v != &entry.name);
								} else {
									self.selected.push(entry.name.clone());
								}
							}
						}
					});
				});
			});
			
			// footer
			ui.horizontal(|ui| {
				if self.mode == FilePickerMode::Save {
					if self.selected.len() == 0 {
						self.selected.push(String::new());
					}
					
					let entry = &mut self.selected[0];
					
					ui.label("File Name");
					let w = ui.available_size()[0] - 100.0;
					ui.set_width(w);
					ui.input_text("", entry);
				} else {
					let w = ui.available_size()[0] - 100.0;
					ui.set_width(w);
					ui.label(self.selected.join("; "));
				}
				
				// let w = ui.available_size()[0];
				// ui.set_width(w - 50.0);
				if ui.button(if self.mode == FilePickerMode::Save {"Save"} else {"Open"}).clicked && self.selected.iter().all(|v| v.len() > 0) {
					r = Some(FilePickerStatus::Success(self.dir.clone(), self.selected.iter().map(|v| self.dir.join(v)).collect()));
				}
			});
		});
		
		if let Some(r) = r {
			r
		} else if open {
			FilePickerStatus::Busy
		} else {
			FilePickerStatus::Canceled(self.dir.clone())
		}
	}
	
	fn set_dir(&mut self, dir: &std::path::Path) {
		if !dir.exists() || !dir.is_dir() {return}
		
		self.dir = dir.to_owned();
		self.list.clear();
		self.selected.clear();
		
		let Ok(read) = std::fs::read_dir(&self.dir) else {return};
		for entry in read {
			let Ok(entry) = entry else {continue};
			let Ok(meta) = entry.metadata() else {continue};
			
			let is_file = meta.is_file();
			let name = entry.file_name().to_string_lossy().to_string();
			if is_file && self.ext_filters.len() > 0 && !self.ext_filters.iter().any(|v| name.ends_with(v)) {continue};
			
			self.list.push(FileEntry {
				is_file,
				name,
				size: if meta.is_file() {meta.len()} else {0},
				modified: meta.modified().unwrap(), // should be fine, we not compiling this for some obscure platform
			})
		}
		
		self.sort();
	}
	
	fn sort(&mut self) {
		self.list.sort_unstable_by(|a, b|
			match self.sort_method {
				FilePickerSort::Name => a.is_file.cmp(&b.is_file),
				FilePickerSort::Size => a.is_file.cmp(&b.is_file).then(a.size.cmp(&b.size)),
				FilePickerSort::Modified => a.is_file.cmp(&b.is_file).then(a.modified.cmp(&b.modified)),
			}.then_with(|| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
		);
	}
}