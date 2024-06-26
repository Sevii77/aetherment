use std::{io::Read, fs::File};

pub type BacktraceError = Box<dyn std::error::Error>;

// #[derive(Debug)]
// pub struct BacktraceError {
// 	#[allow(dead_code)]
// 	err: Box<dyn std::any::Any + Send + 'static>,
// 	backtrace: backtrace::Backtrace,
// }
// 
// // impl BacktraceError {
// // 	pub fn new(err: Box<dyn std::any::Any + Send + 'static>) -> Self {
// // 		Self {
// // 			err,
// // 			backtrace: backtrace::Backtrace::new(),
// // 		}
// // 	}
// // }
// 
// impl<E> From<E> for BacktraceError where
// E: std::error::Error + Send + Sync + 'static {
// 	fn from(err: E) -> Self {
// 		Self {
// 			err: Box::new(err),
// 			backtrace: backtrace::Backtrace::new(),
// 		}
// 	}
// }
// 
// impl std::fmt::Display for BacktraceError {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		write!(f, "{:?}", self.backtrace)
// 	}
// }

#[derive(Debug)]
pub enum ExplorerError {
	Path(String),
	RealPath(String),
	RealPath2(std::path::PathBuf),
	Data,
}

impl std::fmt::Display for ExplorerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Path(path) => write!(f, "Invalid game path: {:?}", path),
			Self::RealPath(path) => write!(f, "Invalid real path: {:?}", path),
			Self::RealPath2(path) => write!(f, "Invalid real path: {:?}", path),
			Self::Data => write!(f, "File is invalid"),
		}
	}
}

impl std::error::Error for ExplorerError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			_ => None,
		}
	}
}

// ----------

pub fn load_file_disk<T>(real_path: &std::path::Path) -> Result<T, BacktraceError> where
T: noumenon::File {
	let mut file = File::open(real_path).map_err(|_| ExplorerError::RealPath2(real_path.to_owned()))?;
	let mut buf = Vec::new();
	file.read_to_end(&mut buf)?;
	Ok(noumenon::File::read(&buf)?)
}

// TODO: ability to load from active mod even if only given a game path (uld resources)
pub fn load_file<T>(path: &str, real_path: Option<&str>) -> Result<T, BacktraceError> where
T: noumenon::File {
	if let Some(real_path) = real_path {
		load_file_disk::<T>(std::path::Path::new(real_path))
	} else {
		Ok(crate::noumenon().as_ref().ok_or(ExplorerError::Path(path.to_string()))?.file::<T>(path)?)
	}
}

// Load json file that can include the UTF-8 BOM
pub fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, crate::resource_loader::BacktraceError> {
	let mut file = File::open(path)?;
	let mut buf = Vec::new();
	file.read_to_end(&mut buf)?;
	if buf.starts_with(&[0xEF, 0xBB, 0xBF]) {
		buf.drain(0..3);
	}
	
	Ok(serde_json::from_slice::<T>(&buf)?)
}