use std::{io::Cursor, ops::{Deref, DerefMut}, path::{Path, PathBuf}};
pub use ironworks::file::File;

macro_rules! simple_reader {
	($y:expr, $z:expr) => {
		let reader = $y;
		let endian = $z;
		
		macro_rules! r {
			(move $c:expr) => {{
				reader.seek_relative($c as i64)?
			}};
			
			(seek $c:expr) => {{
				reader.seek(::std::io::SeekFrom::Start($c as u64))?
			}};
			
			(eof) => {{
				let mut v = Vec::new();
				reader.read_to_end(&mut v)?;
				v
			}};
			
			(Vec<$e:ty>, $c:expr) => {{
				let mut v = Vec::with_capacity($c as usize);
				for _ in 0..$c {
					v.push(<$e>::read_options(reader, endian, ())?);
				}
				v
			}};
			
			(f16) => {{
				half::f16::from_bits(r!(u16)).to_f32()
			}};
			
			($e:ty) => {{
				<$e>::read_options(reader, endian, ())?
			}};
			
			($f:ident, $a:tt) => {{
				$f(reader, endian, $a)?
			}};
		}
	};
}

// ----------

pub mod format;

// https://github.com/redstrate/Physis
// i fucking love you for making this redstrate <3
mod havok;

// ----------

// this docs doesnt autogenerate based on supported types and extensions (is that even possible?)
/// Convert between external/game formats
/// 
/// Mdl
/// - mdl
/// - gltf
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
	Mdl(format::game::Mdl),
	Mtrl(format::game::Mtrl),
	Tex(format::game::Tex),
	Hwc(format::game::Hwc),
	Uld(format::game::Uld),
	
	Dds(Vec<u8>),
	Png(Vec<u8>),
	Tiff(Vec<u8>),
	Tga(Vec<u8>),
}

impl Convert {
	pub fn from_ext<R>(ext: &str, reader: &mut R) -> Result<Self, Error> where
	R: std::io::Read + std::io::Seek {
		use format::{game::*, external::*};
		
		if mdl::EXT.contains(&ext) {return Ok(Self::Mdl(<Mdl as Bytes>::read(reader)?))}
		if mtrl::EXT.contains(&ext) {return Ok(Self::Mtrl(<Mtrl as Bytes>::read(reader)?))}
		if tex::EXT.contains(&ext) {return Ok(Self::Tex(<Tex as Bytes>::read(reader)?))}
		if hwc::EXT.contains(&ext) {return Ok(Self::Hwc(<Hwc as Bytes>::read(reader)?))}
		if uld::EXT.contains(&ext) {return Ok(Self::Uld(<Uld as Bytes>::read(reader)?))}
		
		let mut data = Vec::new();
		reader.read_to_end(&mut data)?;
		if dds::EXT.contains(&ext) {return Ok(Self::Dds(data))}
		if png::EXT.contains(&ext) {return Ok(Self::Png(data))}
		if tiff::EXT.contains(&ext) {return Ok(Self::Tiff(data))}
		if tga::EXT.contains(&ext) {return Ok(Self::Tga(data))}
		
		Err(Error::InvalidFormatFrom(ext.to_string()))
	}
	
	pub fn convert<W>(&self, ext: &str, writer: &mut W, file_path: Option<&str>, file_reader: Option<impl Fn(&str) -> Option<Vec<u8>>>) -> Result<(), Error> where
	W: std::io::Write + std::io::Seek {
		use format::{game::*, external::*};
		
		match self {
			Convert::Mdl(v) => {
				if mdl::EXT.contains(&ext) {return Ok(<Mdl as Bytes>::write(v, writer)?)}
				if gltf::EXT.contains(&ext) {
					let Some(file_path) = file_path else {return Err(Error::ParametersRequires)};
					let Some(file_reader) = file_reader else {return Err(Error::ParametersRequires)};
					
					let skeletons = Mdl::skeleton_paths(file_path)
						.into_iter()
						.flat_map(|v| {
							let sklb_data = file_reader(&v).unwrap();
							let sklb = <Sklb as Bytes>::read(&mut std::io::Cursor::new(sklb_data)).unwrap();
							
							sklb.bones
								.iter()
								.map(|bone| gltf::Bone {
									name: bone.name.clone(),
									parent: if bone.parent >= 0 {Some(sklb.bones[bone.parent as usize].name.clone())} else {None},
									translation: bone.translation,
									rotation: bone.rotation,
									scale: bone.scale,
								}).collect::<Vec<_>>()
						}).collect::<Vec<_>>();
					let materials = v.bake_materials(file_reader);
					
					return Ok(<Mdl as Gltf>::write(v, writer, materials, skeletons)?)
				}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Mtrl(v) => {
				if mtrl::EXT.contains(&ext) {return Ok(<Mtrl as Bytes>::write(v, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Tex(v) => {
				if tex::EXT.contains(&ext) {return Ok(<Tex as Bytes>::write(v, writer)?)}
				if dds::EXT.contains(&ext) {return Ok(<Tex as Dds>::write(v, writer)?)}
				if png::EXT.contains(&ext) {return Ok(<Tex as Png>::write(v, writer)?)}
				if tiff::EXT.contains(&ext) {return Ok(<Tex as Tiff>::write(v, writer)?)}
				if tga::EXT.contains(&ext) {return Ok(<Tex as Tga>::write(v, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Hwc(v) => {
				if hwc::EXT.contains(&ext) {return Ok(<Hwc as Bytes>::write(v, writer)?)}
				if png::EXT.contains(&ext) {return Ok(<Hwc as Png>::write(v, writer)?)}
				if tiff::EXT.contains(&ext) {return Ok(<Hwc as Tiff>::write(v, writer)?)}
				if tga::EXT.contains(&ext) {return Ok(<Hwc as Tga>::write(v, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Uld(v) => {
				if uld::EXT.contains(&ext) {return Ok(<Uld as Bytes>::write(v, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Dds(v) => {
				if tex::EXT.contains(&ext) {return Ok(<Tex as Bytes>::write(&<Tex as Dds>::read(&mut Cursor::new(v))?, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Png(v) => {
				if tex::EXT.contains(&ext) {return Ok(<Tex as Bytes>::write(&<Tex as Png>::read(&mut Cursor::new(v))?, writer)?)}
				if hwc::EXT.contains(&ext) {return Ok(<Hwc as Bytes>::write(&<Hwc as Png>::read(&mut Cursor::new(v))?, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Tiff(v) => {
				if tex::EXT.contains(&ext) {return Ok(<Tex as Bytes>::write(&<Tex as Tiff>::read(&mut Cursor::new(v))?, writer)?)}
				if hwc::EXT.contains(&ext) {return Ok(<Hwc as Bytes>::write(&<Hwc as Tiff>::read(&mut Cursor::new(v))?, writer)?)}
				
				Err(Error::InvalidFormatTo(ext.to_string()))
			}
			
			Convert::Tga(v) => {
				if tex::EXT.contains(&ext) {return Ok(<Tex as Bytes>::write(&<Tex as Tga>::read(&mut Cursor::new(v))?, writer)?)}
				if hwc::EXT.contains(&ext) {return Ok(<Hwc as Bytes>::write(&<Hwc as Tga>::read(&mut Cursor::new(v))?, writer)?)}
				
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0:?}")] Io(#[from] std::io::Error),
	#[error("{0:?}")] Binrw(#[from] binrw::Error),
	#[error("{0:?}")] Dds(#[from] image_dds::ddsfile::Error),
	#[error("{0:?}")] DdsCreate(#[from] image_dds::CreateDdsError),
	#[error("{0:?}")] DdsSurface(#[from] image_dds::error::SurfaceError),
	#[error("{0:?}")] Image(#[from] image::ImageError),
	
	#[error("Invalid format to convert from {0:?}")]
	InvalidFormatFrom(String),
	#[error("Invalid format to convert to {0:?}")]
	InvalidFormatTo(String),
	#[error("Arguments path and file_reading are not set but are needed for this file")]
	ParametersRequires,
}

// ----------

pub(crate) fn crc32(buf: &[u8]) -> u32 {
	let mut hasher = crc32fast::Hasher::new_with_initial(0xFFFFFFFF);
	hasher.update(buf);
	!hasher.finalize()
}

struct VoidReader;
impl ironworks::file::File for VoidReader {
	fn read(_data: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
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
		#[cfg(target_family = "windows")]
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