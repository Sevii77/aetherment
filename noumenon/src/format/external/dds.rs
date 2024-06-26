#![allow(dead_code)]

use std::io::{Read, Seek, Write, SeekFrom};
use binrw::BinReaderExt;
use squish::Format as SFormat;

pub const EXT: &'static [&'static str] = &["dds"];

pub trait Dds {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error> where Self: Sized, T: Read + Seek;
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where T: Write + Seek;
}

// https://docs.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide
// b g r a
// we map everything to be r g b a
// ---------------------------------------- //

#[derive(Copy, Clone)]
pub enum Format {
	Unknown,
	L8,
	A8,
	A4R4G4B4,
	A1R5G5B5,
	A8R8G8B8,
	X8R8G8B8,
	Bc1,
	Bc2,
	Bc3,
	Bc5,
	Bc7,
	A16B16G16R16
}

impl Format {
	pub fn convert_from(&self, width: usize, height: usize, data: &[u8]) -> Option<Vec<u8>> { // -> r, g, b, a
		match self {
			Format::L8       => Some(convert_from_l8(data)),
			Format::A8       => Some(convert_from_a8(data)),
			Format::A4R4G4B4 => Some(convert_from_a4r4g4b4(data)),
			Format::A1R5G5B5 => Some(convert_from_a1r5g5b5(data)),
			// Format::A8R8G8B8 => Some(Vec::from(data)),
			Format::A8R8G8B8 => Some(convert_from_a8r8g8b8(data)),
			Format::X8R8G8B8 => Some(convert_from_x8r8g8b8(data)),
			Format::Bc1      => Some(convert_from_compressed(SFormat::Bc1, width, height, data)),
			Format::Bc2      => Some(convert_from_compressed(SFormat::Bc2, width, height, data)),
			Format::Bc3      => Some(convert_from_compressed(SFormat::Bc3, width, height, data)),
			Format::Bc5      => Some(convert_from_compressed(SFormat::Bc5, width, height, data)),
			_                => None,
		}
	}
	
	pub fn convert_to(&self, width: usize, height: usize, data: &[u8]) -> Option<Vec<u8>> {
		match self {
			Format::L8       => Some(convert_to_l8(data)),
			Format::A8       => Some(convert_to_a8(data)),
			Format::A4R4G4B4 => Some(convert_to_a4r4g4b4(data)),
			Format::A1R5G5B5 => Some(convert_to_a1r5g5b5(data)),
			// Format::A8R8G8B8 => Some(Vec::from(data)),
			Format::A8R8G8B8 => Some(convert_to_a8r8g8b8(data)),
			Format::X8R8G8B8 => Some(convert_to_x8r8g8b8(data)),
			Format::Bc1      => Some(convert_to_compressed(SFormat::Bc1, width, height, data)),
			Format::Bc2      => Some(convert_to_compressed(SFormat::Bc2, width, height, data)),
			Format::Bc3      => Some(convert_to_compressed(SFormat::Bc3, width, height, data)),
			Format::Bc5      => Some(convert_to_compressed(SFormat::Bc5, width, height, data)),
			_                => None,
		}
	}
	
	pub fn get<T>(reader: &mut T) -> Format where T: Read + Seek {
		reader.seek(SeekFrom::Start(84)).unwrap();
		let cc: u32 = reader.read_le().unwrap();
		reader.seek(SeekFrom::Start(92)).unwrap();
		let rmask: u32 = reader.read_le().unwrap();
		reader.seek(SeekFrom::Current(8)).unwrap();
		let amask: u32 = reader.read_le().unwrap();
		
		match (cc, rmask, amask) { // eh, good enough
			(0,          0xFF,       0         ) => Format::L8,
			(0,          0,          0xFF      ) => Format::A8,
			(0,          0x0F00,     0xF000    ) => Format::A4R4G4B4,
			(0,          0x7C00,     0x8000    ) => Format::A1R5G5B5,
			(0,          0x00FF0000, 0xFF000000) => Format::A8R8G8B8,
			(0,          0x00FF0000, 0         ) => Format::X8R8G8B8,
			(0x31545844, 0,          0         ) => Format::Bc1,
			(0x33545844, 0,          0         ) => Format::Bc2,
			(0x35545844, 0,          0         ) => Format::Bc3,
			(0x31495441, 0,          0         ) => Format::Bc5,
			// (113,        0,          0         ) => Format::A16B16G16R16,
			_                                    => Format::Unknown,
		}
	}
	
	pub fn flags(&self) -> u32 {
		match self {
			Format::Bc1 | Format::Bc2 | Format::Bc3 | Format::A16B16G16R16 => 0x00081007,
			_ => 0x0000100F,
		}
	}
	
	pub fn flags2(&self) -> u32 {
		match self {
			Format::Bc1 | Format::Bc2 | Format::Bc3 | Format::Bc5 | Format::A16B16G16R16 => 0x4,
			Format::A8R8G8B8 | Format::A4R4G4B4 | Format::A1R5G5B5 => 0x41,
			Format::X8R8G8B8 => 0x40,
			Format::L8 => 0x20000,
			Format::A8 => 0x2,
			_ => 0,
		}
	}
	
	pub fn fourcc(&self) -> u32 {
		match self {
			Format::Bc1 => 0x31545844,
			Format::Bc2 => 0x33545844,
			Format::Bc3 => 0x35545844,
			Format::Bc5 => 0x31495441,
			Format::A16B16G16R16 => 113,
			_ => 0,
		}
	}
	
	pub fn masks(&self) -> (u32, u32, u32, u32) {
		match self {
			Format::A8R8G8B8 => (0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000),
			Format::X8R8G8B8 => (0x000000FF, 0x0000FF00, 0x00FF0000, 0         ),
			Format::A4R4G4B4 => (0x000F,     0x00F0,     0x0F00,     0xF000    ),
			Format::A1R5G5B5 => (0x001F,     0x03E0,     0x7C00,     0x8000    ),
			Format::L8       => (0,          0,          0xFF,       0         ),
			Format::A8       => (0,          0,          0,          0xFF      ),
			_ => (0, 0, 0, 0),
		}
	}
	
	pub fn bitcount(&self) -> u32 {
		match self {
			Format::Bc1 => 4,
			Format::Bc2 | Format::Bc3 | Format::Bc5 | Format::L8 | Format::A8 => 8,
			Format::A4R4G4B4 | Format::A1R5G5B5 => 16,
			Format::A8R8G8B8 | Format::X8R8G8B8 => 32,
			Format::A16B16G16R16 => 64,
			_ => 0,
		}
	}
}

// ---------------------------------------- //

fn convert_from_l8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(1)
		.flat_map(|p| {
			let v = p[0];
			[v, v, v, 255]
		}).collect::<Vec<u8>>()
}

fn convert_to_l8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[0]]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(1)
		.flat_map(|p| {
			[0, 0, 0, p[0]]
		}).collect::<Vec<u8>>()
}

fn convert_to_a8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[0]]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a4r4g4b4(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(2)
		.flat_map(|p| {
			let v = ((p[1] as u16) << 8) + p[0] as u16;
			[
				// ((v & 0x000F) << 4) as u8,
				// ((v & 0x00F0)     ) as u8,
				// ((v & 0x0F00) >> 4) as u8,
				// ((v & 0xF000) >> 8) as u8,
				
				((v & 0x0F00) >> 4) as u8,
				((v & 0x00F0)     ) as u8,
				((v & 0x000F) << 4) as u8,
				((v & 0xF000) >> 8) as u8,
			]
		}).collect::<Vec<u8>>()
}

fn convert_to_a4r4g4b4(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[
				// (p[0] >> 4) + (p[1] & 0xF0),
				// (p[2] >> 4) + (p[3] & 0xF0),
				
				(p[2] >> 4) + (p[1] & 0xF0),
				(p[0] >> 4) + (p[3] & 0xF0),
			]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a1r5g5b5(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(2)
		.flat_map(|p| {
			let v = ((p[1] as u16) << 8) + p[0] as u16;
			[
				// ((v & 0x001F) << 3) as u8,
				// ((v & 0x03E0) >> 2) as u8,
				// ((v & 0x7C00) >> 7) as u8,
				// ((v & 0x8000) >> 8) as u8,
				
				((v & 0x7C00) >> 7) as u8,
				((v & 0x03E0) >> 2) as u8,
				((v & 0x001F) << 3) as u8,
				((v & 0x8000) >> 8) as u8,
			]
		}).collect::<Vec<u8>>()
}

fn convert_to_a1r5g5b5(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[
				// (p[0] >> 3) + ((p[1] << 2) & 0xE0),
				// (p[1] >> 6) + ((p[2] >> 1) & 0x7C) + p[3] & 0x80,
				
				(p[2] >> 3) + ((p[1] << 2) & 0xE0),
				(p[1] >> 6) + ((p[0] >> 1) & 0x7C) + p[3] & 0x80,
			]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a8r8g8b8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[2], p[1], p[0], p[3]]
		}).collect::<Vec<u8>>()
}

fn convert_to_a8r8g8b8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[2], p[1], p[0], p[3]]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_x8r8g8b8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			// [p[0], p[1], p[2], 255]
			[p[2], p[1], p[0], 255]
		}).collect::<Vec<u8>>()
}

fn convert_to_x8r8g8b8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			// [p[0], p[1], p[2], 0]
			[p[2], p[1], p[0], 0]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_compressed(format: SFormat, width: usize, height: usize, data: &[u8]) -> Vec<u8> {
	// dont bother with things smaller than the chunk size (4x4)
	if width < 4 || height < 4 {
		return vec![0; width * height * 4]
	}
	
	let mut output = vec![0u8; width * height * 4];
	format.decompress(data, width, height, &mut output);
	// output.chunks_exact(4)
	// 	.flat_map(|p| {
	// 		[p[2], p[1], p[0], p[3]]
	// 	}).collect::<Vec<u8>>()
	output
}

fn convert_to_compressed(format: SFormat, width: usize, height: usize, data: &[u8]) -> Vec<u8> {
	// let data = data.chunks_exact(4)
	// 	.flat_map(|p| {
	// 		[p[2], p[1], p[0], p[3]]
	// 	}).collect::<Vec<u8>>();
	let mut output = vec![0u8; format.compressed_size(width, height)];
	format.compress(&data, width, height, squish::Params {
		algorithm: squish::Algorithm::IterativeClusterFit,
		weights: squish::COLOUR_WEIGHTS_UNIFORM,
		weigh_colour_by_alpha: true,
	}, &mut output);
	output
}