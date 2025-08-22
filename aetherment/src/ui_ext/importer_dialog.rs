#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("No selected path")]
	NoPath,
	
	#[error("Selected file did not have an extension")]
	MissingExtension,
	
	#[error("Io error {0:?}")]
	IoError(#[from] std::io::Error),
	
	#[error("Failed converting file {0:?}")]
	NoumenonError(#[from] noumenon::Error),
}

pub enum DialogResult {
	Busy,
	Cancelled,
	Success(Vec<u8>),
}

pub struct ImporterDialog {
	dialog: egui_file::FileDialog,
	target_ext: String,
}

impl ImporterDialog {
	pub fn new(title: impl AsRef<str>, target_ext: impl Into<String>) -> Self {
		let mut dialog = egui_file::FileDialog::open_file(Some(crate::config().config.file_dialog_path.clone()))
			.title(title.as_ref());
		dialog.open();
		
		Self {
			dialog,
			target_ext: target_ext.into(),
		}
	}
	
	pub fn show(&mut self, ui: &mut egui::Ui) -> Result<DialogResult, Error> {
		let config = crate::config();
		match self.dialog.show(ui.ctx()).state() {
			egui_file::State::Selected => {
				config.config.file_dialog_path = self.dialog.directory().to_path_buf();
				_ = config.save_forced();
				
				let path = self.dialog.path().ok_or(Error::NoPath)?;
				let file = std::fs::File::open(&path)?;
				let ext = path.extension().map(|v| v.to_string_lossy().to_string()).ok_or(Error::MissingExtension)?;
				let converter = noumenon::Convert::from_ext(&ext, &mut std::io::BufReader::new(file))?;
				
				let mut buf = Vec::new();
				converter.convert(&self.target_ext, &mut std::io::Cursor::new(&mut buf), None, None::<fn(&str) -> Option<Vec<u8>>>)?;
				
				Ok(DialogResult::Success(buf))
			}
			
			egui_file::State::Cancelled => {
				config.config.file_dialog_path = self.dialog.directory().to_path_buf();
				_ = config.save_forced();
				
				Ok(DialogResult::Cancelled)
			}
			
			_ => Ok(DialogResult::Busy)
		}
	}
}