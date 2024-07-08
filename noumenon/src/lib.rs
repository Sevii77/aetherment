use std::{ops::{Deref, DerefMut}, path::{Path, PathBuf}};
pub use ironworks::file::File;

pub mod format;

// ----------

// this docs doesnt autogenerate based on supported types and extensions (is that even possible?)
/// Convert between external/game formats
/// 
/// Mtrl
/// - mtrl
/// 
/// Uld
/// - uld
/// 
/// Tex
/// - tex / atex
/// - dds
/// - png
/// - tiff / tif
/// - tga
pub enum Convert {
	// Mdl,
	Mtrl(format::game::Mtrl),
	Tex(format::game::Tex),
	Uld(format::game::Uld),
}

impl Convert {
	pub fn from_ext<R>(ext: &str, reader: &mut R) -> Result<Self, Error> where
	R: std::io::Read + std::io::Seek {
		if format::game::mtrl::EXT.contains(&ext) {return Ok(Self::Mtrl(format::game::Mtrl::read(reader)?))}
		
		if format::game::tex::EXT.contains(&ext) {return Ok(Self::Tex(format::game::Tex::read(reader)?))}
		if format::external::dds::EXT.contains(&ext) {return Ok(Self::Tex(<format::game::Tex as format::external::Dds>::read(reader)?))}
		if format::external::png::EXT.contains(&ext) {return Ok(Self::Tex(<format::game::Tex as format::external::Png>::read(reader)?))}
		if format::external::tiff::EXT.contains(&ext) {return Ok(Self::Tex(<format::game::Tex as format::external::Tiff>::read(reader)?))}
		if format::external::tga::EXT.contains(&ext) {return Ok(Self::Tex(<format::game::Tex as format::external::Tga>::read(reader)?))}
		
		if format::game::uld::EXT.contains(&ext) {return Ok(Self::Uld(format::game::Uld::read(reader)?))}
		
		Err(Error::InvalidFormatFrom(ext.to_string()))
	}
	
	pub fn convert<W>(&self, ext: &str, writer: &mut W) -> Result<(), Error> where
	W: std::io::Write + std::io::Seek {
		match self {
			Convert::Mtrl(v) => {
				if format::game::mtrl::EXT.contains(&ext) {return format::game::Mtrl::write(v, writer)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Tex(v) => {
				if format::game::tex::EXT.contains(&ext) {return format::game::Tex::write(v, writer)}
				if format::external::dds::EXT.contains(&ext) {return <format::game::Tex as format::external::Dds>::write(v, writer)}
				if format::external::png::EXT.contains(&ext) {return <format::game::Tex as format::external::Png>::write(v, writer)}
				if format::external::tiff::EXT.contains(&ext) {return <format::game::Tex as format::external::Tiff>::write(v, writer)}
				if format::external::tga::EXT.contains(&ext) {return <format::game::Tex as format::external::Tga>::write(v, writer)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Uld(v) => {
				if format::game::uld::EXT.contains(&ext) {return format::game::Uld::write(v, writer)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
		}
	}
}

// ----------

trait NullReader {
	fn null_terminated(&self) -> Result<String, std::str::Utf8Error>;
}

impl NullReader for [u8] {
	fn null_terminated(&self) -> Result<String, std::str::Utf8Error> {
		let p = std::str::from_utf8(&self)?;
		Ok(if let Some(l) = p.find('\0') {&p[0..l]} else {p}.to_owned())
	}
}

trait NullWriter {
	fn null_terminated(&self, len: usize) -> Result<Vec<u8>, SizeError>;
}

impl NullWriter for String {
	fn null_terminated(&self, len: usize) -> Result<Vec<u8>, SizeError> {
		let mut vec = vec![0; len];
		let bytes = self.as_bytes();
		if bytes.len() > len {return Err(SizeError{len: bytes.len() as u32, max_len: len as u32})}
		bytes.into_iter().enumerate().for_each(|(i, v)| vec[i] = *v);
		Ok(vec)
	}
}

// ----------

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct SizeError {
	len: u32,
	max_len: u32,
}

impl std::fmt::Display for SizeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "string is too long, {} bytes while max is {}", self.len, self.max_len)
	}
}

impl std::error::Error for SizeError {
	fn description(&self) -> &str {
		"string is too long"
	}
}

// ----------

// pub type Error = Box<dyn std::error::Error>;
#[derive(Debug)]
pub enum Error {
	Str(&'static str),
	Io(std::io::Error),
	Binrw(binrw::Error),
	Image(image::ImageError),
	Size(SizeError),
	Utf8(std::str::Utf8Error),
	InvalidFormatFrom(String),
	InvalidFormatTo(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Str(err) => f.write_str(err),
			Self::Io(err) => write!(f, "{:?}", err),
			Self::Binrw(err) => write!(f, "{:?}", err),
			Self::Image(err) => write!(f, "{:?}", err),
			Self::Size(err) => write!(f, "{:?}", err),
			Self::Utf8(err) => write!(f, "{:?}", err),
			Self::InvalidFormatFrom(ext) => write!(f, "Invalid format to convert from {:?}", ext),
			Self::InvalidFormatTo(ext) => write!(f, "Invalid format to convert to {:?}", ext),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Io(err) => err.source(),
			Self::Binrw(err) => err.source(),
			Self::Image(err) => err.source(),
			Self::Size(err) => err.source(),
			Self::Utf8(err) => err.source(),
			_ => None,
		}
	}
}

impl From<&'static str> for Error {
	fn from(err: &'static str) -> Self {
		Self::Str(err)
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Self::Io(err)
	}
}

impl From<binrw::Error> for Error {
	fn from(err: binrw::Error) -> Self {
		Self::Binrw(err)
	}
}

impl From<image::ImageError> for Error {
	fn from(err: image::ImageError) -> Self {
		Self::Image(err)
	}
}

impl From<SizeError> for Error {
	fn from(err: SizeError) -> Self {
		Self::Size(err)
	}
}

impl From<std::str::Utf8Error> for Error {
	fn from(err: std::str::Utf8Error) -> Self {
		Self::Utf8(err)
	}
}

// ----------

struct VoidReader;
impl ironworks::file::File for VoidReader {
	fn read(_data: impl ironworks::FileStream) -> format::game::Result<Self> {
		Ok(VoidReader)
	}
}

// TODO: own game data reader, drop ironworks as it is barely used
pub struct Noumenon(ironworks::Ironworks);

impl Noumenon {
	pub fn exists(&self, path: &str) -> bool {
		self.file::<VoidReader>(path).is_ok()
	}
}

impl Deref for Noumenon {
	type Target = ironworks::Ironworks;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Noumenon {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

// use this for plugin
// std::env::current_exe().unwrap().parent().unwrap().parent().unwrap()
pub fn get_noumenon<P>(gamepath: Option<P>) -> Option<Noumenon> where
P: AsRef<Path> {
	if let Some(gamepath) = gamepath {
		if gamepath.as_ref().exists() && gamepath.as_ref().join("game").exists() {
			return Some(Noumenon(ironworks::Ironworks::new()
				.with_resource(ironworks::sqpack::SqPack::new(ironworks::sqpack::Install::at(gamepath.as_ref())))));
		}
	} else {
		// super basic windows autodetect
		for drive_letter in 'A'..'Z' {
			for path in [":/SquareEnix/FINAL FANTASY XIV - A Realm Reborn",
			":/Program Files (x86)/FINAL FANTASY XIV - A Realm Reborn",
			":/Program Files (x86)/SquareEnix/FINAL FANTASY XIV - A Realm Reborn",
			":/Program Files (x86)/Steam/steamapps/common/FINAL FANTASY XIV Online",
			":/Program Files (x86)/Steam/steamapps/common/FINAL FANTASY XIV - A Realm Reborn",
			":/SteamLibrary/steamapps/common/FINAL FANTASY XIV Online",
			":/SteamLibrary/steamapps/common/FINAL FANTASY XIV - A Realm Reborn"] {
				let try_path = PathBuf::from(format!("{drive_letter}{path}"));
				if try_path.exists() {
					return Some(Noumenon(ironworks::Ironworks::new()
						.with_resource(ironworks::sqpack::SqPack::new(ironworks::sqpack::Install::at(try_path.as_ref())))));
				}
			}
		}
	}
	
	None
}