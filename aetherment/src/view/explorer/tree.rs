use std::{cmp::Ordering, collections::{HashMap, HashSet}, fs::File, io::{Read, Seek, SeekFrom}, ops::{Deref, DerefMut}, rc::Rc};
use binrw::BinWrite;
use flate2::read::GzDecoder;
use crate::{ui_ext::UiExt, view::explorer::Action};

struct LazyBranch {
	name: Rc<str>,
	offset: u32,
	branches: Option<Vec<LazyBranch>>,
}

pub struct LazyTree {
	data: Vec<u8>,
	branches: Vec<LazyBranch>,
}

impl LazyTree {
	pub fn new() -> Self {
		Self {
			branches: Vec::new(),
			data: Vec::new(),
		}
	}
	
	pub fn load(&mut self, data: impl Into<Vec<u8>>) -> Result<(), crate::resource_loader::BacktraceError> {
		let data = data.into();
		
		self.branches = Self::load_branch(&data, 0)?;
		self.data = data;
		
		Ok(())
	}
	
	fn load_branch(data: &[u8], offset: u32) -> Result<Vec<LazyBranch>, crate::resource_loader::BacktraceError> {
		let mut offset = offset as usize;
		let mut branch = Vec::new();
		
		offset += 2;
		for _ in 0..u16::from_le_bytes(data[offset - 2 .. offset].try_into()?) {
			let name_len = data[offset] as usize;
			offset += 1;
			let name = std::str::from_utf8(&data[offset .. offset + name_len])?;
			offset += name_len;
			let sub_offset = u32::from_le_bytes(data[offset .. offset + 4].try_into()?);
			offset += 4;
			
			branch.push(LazyBranch {
				name: Rc::from(name),
				offset: sub_offset,
				branches: None,
			});
		}
		
		branch.sort_by(|a, b|
			(if a.offset == 0 && b.offset != 0 {Ordering::Greater} else if a.offset != 0 && b.offset == 0 {Ordering::Less} else {Ordering::Equal})
			.then(a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
		);
		
		Ok(branch)
	}
	
	pub fn render(&mut self, ui: &mut egui::Ui, selected_paths: &HashSet<String>) -> Action {
		let mut act = Action::None;
		for branch in &mut self.branches {
			act.or(Self::render_branch(&self.data, ui, branch, branch.name.to_string(), selected_paths));
		}
		
		act
	}
	
	fn render_branch(data: &[u8], ui: &mut egui::Ui, branch: &mut LazyBranch, path: String, selected_paths: &HashSet<String>) -> Action {
		let mut act = Action::None;
		if branch.offset != 0 {
			ui.collapsing(branch.name.as_ref(), |ui| {
				let branches = branch.branches.get_or_insert_with(|| Self::load_branch(data, branch.offset).unwrap());
				for branch in branches {
					act.or(Self::render_branch(data, ui, branch, format!("{path}/{}", branch.name), selected_paths));
				}
			});
		} else {
			let resp = ui.selectable_label(selected_paths.contains(&path), branch.name.as_ref());
			if resp.clicked() {
				act = Action::OpenExisting(path.clone());
			}
			
			resp.context_menu(|ui| {
				if ui.button("Open in new tab").clicked() {
					act = Action::OpenNew(path.clone());
					ui.close_menu();
				}
				
				if ui.button("Replace open tab").clicked() {
					act = Action::OpenExisting(path);
					ui.close_menu();
				}
			});
		}
		
		act
	}
}

// ----------

pub struct Tree {
	pub(crate) open_paths: HashSet<String>,
	
	game_paths: LazyTree,
	game_paths_exist: bool,
	game_paths_error: Option<crate::resource_loader::BacktraceError>,
	game_paths_download_progress: crate::modman::backend::Progress,
}

impl Tree {
	pub fn new() -> Self {
		let mut s = Self {
			open_paths: HashSet::new(),
			
			game_paths: LazyTree::new(),
			game_paths_exist: false,
			game_paths_error: None,
			game_paths_download_progress: crate::modman::backend::Progress::new(),
		};
		
		s.load_game_tree();
		
		s
	}
	
	fn load_game_tree(&mut self) {
		let cache_dir = dirs::cache_dir().ok_or("No Cache Dir (???)").unwrap().join("Aetherment");
		let path = cache_dir.join("paths");
		self.game_paths_exist = path.exists();
		if !self.game_paths_exist {return}
		
		let data = match std::fs::read(path) {
			Ok(v) => v,
			Err(e) => {
				self.game_paths_error = Some(Box::new(e));
				return;
			}
		};
		
		if let Err(e) = self.game_paths.load(data) {
			self.game_paths_error = Some(e);
		}
	}
}

impl super::ExplorerView for Tree {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
	
	fn title(&self) -> String {
		"Tree".to_string()
	}
	
	fn path(&self) -> Option<&str> {
		None
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _renderer: &renderer::Renderer) -> Action {
		let mut act = Action::None;
		ui.collapsing("Files", |ui| {
			let progress = self.game_paths_download_progress.get();
			if progress != 0.0 {
				ui.add(egui::ProgressBar::new(progress)
					.text(format!("{} {:.0}%", self.game_paths_download_progress.get_msg(), (progress * 200.0) % 100.0)));
				
				ui.ctx().request_repaint();
				
				if progress >= 1.0 {
					self.load_game_tree();
					self.game_paths_download_progress.set(0.0);
				}
			}
			
			if !self.game_paths_exist {
				ui.add(egui::Label::new("Paths haven't been downloaded yet.").wrap());
				
				if ui.button("Download paths")
					.on_hover_text("Paths are provided by ResLogger2 (https://rl2.perchbird.dev)")
					.clicked() {
					update_paths(self.game_paths_download_progress.clone());
				}
			} else if let Some(e) = self.game_paths_error.as_ref() {
				ui.add(egui::Label::new("An error was experienced while browsing files. This may be caused by a corrupted paths file.").wrap());
				ui.add(egui::Label::new(egui::RichText::new(format!("{e:#?}")).color(egui::Color32::RED)).wrap());
				
				if ui.button("Redownload paths")
					.on_hover_text("Paths are provided by ResLogger2 (https://rl2.perchbird.dev)")
					.clicked() {
					update_paths(self.game_paths_download_progress.clone());
				}
			} else {
				ui.scope(|ui| {
					ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
					act.or(self.game_paths.render(ui, &self.open_paths));
				});
				
				ui.spacer();
				if ui.button("Redownload paths")
					.on_hover_text("Paths are provided by ResLogger2 (https://rl2.perchbird.dev)")
					.clicked() {
					update_paths(self.game_paths_download_progress.clone());
				}
			}
		});
		
		act
	}
}

// ----------

// TODO: seperate website as to not overload perchbird and know when to download a new version
// also perhabs add a logger to the plugin to contribute
// and probably put the path file creation on the server so it only has to be done once
const PATHSURL: &'static str = "https://rl2.perchbird.dev/download/export/CurrentPathList.gz";

#[derive(Debug, Default)]
struct Branch<'a>(HashMap<&'a str, Branch<'a>>);
impl<'a> Deref for Branch<'a> {
	type Target = HashMap<&'a str, Branch<'a>>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'a> DerefMut for Branch<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

fn update_paths(progress: crate::modman::backend::Progress) {
	progress.set(0.001);
	
	std::thread::spawn(move || {
		if let Err(e) = (|| -> Result<(), crate::resource_loader::BacktraceError> {
			log!("downloading");
			progress.set_msg("Downloading");
			
			let resp = ureq::get(PATHSURL)
				.call()?;
			
			let size = 's: {
				let Some(c) = resp.headers().get("Content-Length") else {break 's 0};
				let Ok(s) = c.to_str() else {break 's 0};
				s.parse::<u32>().unwrap_or(0)
			};
			
			let mut reader = resp
				.into_body()
				.into_reader();
			
			let mut data = Vec::new();
			let mut buf = [0u8; 16384];
			loop {
				let readcount = reader.read(&mut buf)?;
				if readcount == 0 {break}
				data.extend_from_slice(&buf[..readcount]);
				progress.set((data.len() as f32  / size as f32).min(0.5));
			}
			
			log!("decoding");
			progress.set(0.5);
			progress.set_msg("Decoding");
			let mut decoder = GzDecoder::new(&data[..]);
			let mut paths = String::new();
			decoder.read_to_string(&mut paths)?;
			
			log!("creating tree");
			progress.set_msg("Creating Tree");
			let mut total_count = 0;
			let mut tree = Branch::default();
			for path in paths.split("\n") {
				total_count += 1;
				let mut branch = &mut tree;
				for seg in path.split("/") {
					branch = branch.entry(seg).or_insert_with(|| Branch::default());
				}
			}
			
			log!("writing tree");
			progress.set_msg("Writing Tree");
			let mut finished_count = 0;
			let cache_dir = dirs::cache_dir().ok_or("No Cache Dir (???)").unwrap().join("Aetherment");
			_ = std::fs::create_dir_all(&cache_dir);
			let mut writer = std::io::BufWriter::new(File::create(cache_dir.join("paths"))?);
			fn write_branch<W: std::io::Write + Seek>(branch: &Branch, mut writer: &mut W, finished_count: &mut u32, total_count: u32, progress: &crate::modman::backend::Progress) -> Result<(), crate::resource_loader::BacktraceError> {
				let mut offsets = HashMap::new();
				
				(branch.len() as u16).write_le(&mut writer)?;
				for (name, sub_branch) in branch.iter() {
					(name.len() as u8).write_le(&mut writer)?;
					name.as_bytes().write_le(&mut writer)?;
					offsets.insert(writer.stream_position()?, sub_branch);
					0u32.write_le(&mut writer)?; // list offset, we write over this later
					
					*finished_count += 1;
				}
				
				progress.set((*finished_count as f32 / total_count as f32) * 0.5 + 0.5);
				
				// now that we wrote the list we can write the lists of the items
				for (offset, sub_branch) in offsets {
					if sub_branch.len() > 0 {
						// overwrite the offset
						let pos = writer.stream_position()? as u32;
						writer.seek(SeekFrom::Start(offset))?;
						pos.write_le(&mut writer)?;
						writer.seek(SeekFrom::End(0))?;
						write_branch(sub_branch, writer, finished_count, total_count, progress)?;
					}
				}
				
				Ok(())
			}
			
			write_branch(&tree, &mut writer, &mut finished_count, total_count, &progress)?;
			
			Ok(())
		})() {
			log!(err, "Failed fetching paths {e:?}");
		}
		
		progress.set(1.0);
		log!("done");
	});
}