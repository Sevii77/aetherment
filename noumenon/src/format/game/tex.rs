#![allow(dead_code)]

use std::io::{Read, Seek, Write, SeekFrom, BufReader};
use binrw::{BinRead, BinReaderExt, BinWrite, binrw};
use image::{codecs::{png::PngEncoder, tiff::TiffEncoder, tga::TgaEncoder}, ImageEncoder, ColorType};
use crate::{Error, format::external::{dds::{Dds, Format as DFormat}, png::Png, tiff::Tiff, tga::Tga}};

pub const EXT: &'static [&'static str] = &["tex", "atex"];

#[repr(C)]
pub struct Pixel {
	pub b: u8,
	pub g: u8,
	pub r: u8,
	pub a: u8,
}

#[binrw]
#[brw(little, repr = u32)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Format {
	Unknown = 0x0,
	
	L8 = 0x1130,
	A8 = 0x1131,
	
	A4R4G4B4 = 0x1440,
	A1R5G5B5 = 0x1441,
	A8R8G8B8 = 0x1450,
	X8R8G8B8 = 0x1451,
	
	R32 = 0x2150,
	R16G16 = 0x2250,
	R32G32 = 0x2260,
	A16B16G16R16 = 0x2460,
	A32B32G32R32 = 0x2470,
	
	Bc1 = 0x3420,
	Bc2 = 0x3430,
	Bc3 = 0x3431,
	Bc5 = 0x6230,
	Bc7 = 0x6432,
	
	D16 = 0x4140,
	D24S8 = 0x4250,
	
	Null = 0x5100,
	Shadow16 = 0x5140,
	Shadow24 = 0x5150,
}

impl From<Format> for DFormat {
	fn from(format: Format) -> Self {
		match format {
			Format::L8           => DFormat::L8,
			Format::A8           => DFormat::A8,
			Format::A4R4G4B4     => DFormat::A4R4G4B4,
			Format::A1R5G5B5     => DFormat::A1R5G5B5,
			Format::A8R8G8B8     => DFormat::A8R8G8B8,
			Format::X8R8G8B8     => DFormat::X8R8G8B8,
			Format::Bc1          => DFormat::Bc1,
			Format::Bc2          => DFormat::Bc2,
			Format::Bc3          => DFormat::Bc3,
			Format::Bc5          => DFormat::Bc5,
			Format::Bc7          => DFormat::Bc7,
			Format::A16B16G16R16 => DFormat::A16B16G16R16,
			_                    => DFormat::Unknown,
		}
	}
}

impl From<DFormat> for Format {
	fn from(format: DFormat) -> Self {
		match format {
			DFormat::L8           => Format::L8,
			DFormat::A8           => Format::A8,
			DFormat::A4R4G4B4     => Format::A4R4G4B4,
			DFormat::A1R5G5B5     => Format::A1R5G5B5,
			DFormat::A8R8G8B8     => Format::A8R8G8B8,
			DFormat::X8R8G8B8     => Format::X8R8G8B8,
			DFormat::Bc1          => Format::Bc1,
			DFormat::Bc2          => Format::Bc2,
			DFormat::Bc3          => Format::Bc3,
			DFormat::Bc5          => Format::Bc5,
			DFormat::Bc7          => Format::Bc7,
			DFormat::A16B16G16R16 => Format::A16B16G16R16,
			DFormat::Unknown      => Format::Unknown,
		}
	}
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Header {
	pub flags: u32,
	pub format: Format,
	pub width: u16,
	pub height: u16,
	pub depths: u16,
	pub mip_levels: u16,
	pub lod_offsets: [u32; 3],
	pub mip_offsets: [u32; 13],
}

unsafe impl Send for Header {}
impl std::panic::UnwindSafe for Header {}

#[derive(Debug, Clone)]
pub struct Tex {
	pub header: Header,
	pub data: Vec<u8>,
}

unsafe impl Send for Tex {}
impl std::panic::UnwindSafe for Tex {}

// used to load from spack using ironworks
impl ironworks::file::File for Tex {
	fn read(mut data: impl ironworks::FileStream) -> super::Result<Self> {
		Tex::read(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

// image crate, for use in image::imageops for example
// r and b are flipped, as image crate works with rgba while tex is bgra,
// i choose not to correct this for use in reassigning data with the result
impl image::GenericImageView for Tex {
	type Pixel = image::Rgba<u8>;
	
	fn dimensions(&self) -> (u32, u32) {
		(self.header.width as u32, self.header.height as u32)
	}
	
	fn bounds(&self) -> (u32, u32, u32, u32) {
		(0, 0, self.header.width as u32, self.header.height as u32)
	}
	
	fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
		let offset = (self.width() * y * 4 + x * 4) as usize;
		image::Rgba::<u8>([self.data[offset], self.data[offset + 1], self.data[offset + 2], self.data[offset + 3]])
	}
}

impl Tex {
	// pub fn as_pixels(&self) -> &[Pixel] {
	// 	unsafe{::std::slice::from_raw_parts(self.data.as_ptr() as *const _, self.data.len() / 4)}
	// }
	// 
	// pub fn as_pixels_mut(&mut self) -> &mut [Pixel] {
	// 	unsafe{::std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut _, self.data.len() / 4)}
	// }
	// 
	// pub fn overlay_onto(&self, target: &mut Tex) {
	// 	let pixels = self.as_pixels();
	// 	target.as_pixels_mut().iter_mut().enumerate().for_each(|(i, pixel)| {
	// 		let ar = pixel.a as f32 / 255.0;
	// 		let ao = pixels[i].a as f32 / 255.0;
	// 		let a = ao + ar * (1.0 - ao);
	// 		
	// 		pixel.b = ((pixels[i].b as f32 * ao + pixel.b as f32 * ar * (1.0 - ao)) / a) as u8;
	// 		pixel.g = ((pixels[i].g as f32 * ao + pixel.g as f32 * ar * (1.0 - ao)) / a) as u8;
	// 		pixel.r = ((pixels[i].r as f32 * ao + pixel.r as f32 * ar * (1.0 - ao)) / a) as u8;
	// 		pixel.a = (a * 255.0) as u8;
	// 	});
	// }
	
	pub fn slice(&self, miplevel: u16, depth: u16) -> Option<(u16, u16, &[u8])> {
		Self::slice_manual(miplevel, depth, &self.header, 32, &self.data)
	}
	
	fn slice_manual<'a>(miplevel: u16, depth: u16, header: &Header, bitcount: u32, data: &'a [u8]) -> Option<(u16, u16, &'a [u8])> {
		let bytecount = bitcount as f32 / 8.0;
		let factor = 0.5f32.powi(miplevel as i32);
		let (w, h) = (header.width as f32 * factor, header.height as f32 * factor);
		if w < 1.0 || h < 1.0 {return Some((0, 0, &data[0..0]))}
		let slicesize = (w * h * bytecount as f32) as usize;
		let offset = ((header.mip_offsets.get(miplevel as usize)? - 80) as f32 * bytecount / (DFormat::from(header.format).bitcount() as f32 / 8.0)) as usize + slicesize * depth as usize;
		Some((w as u16, h as u16, data.get(offset..(offset + slicesize))?))
	}
	
	pub fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		let header = <Header as BinRead>::read(reader)?;
		
		reader.seek(SeekFrom::End(0))?;
		let mut data = Vec::with_capacity(reader.stream_position()? as usize);
		reader.seek(SeekFrom::Start(80))?;
		reader.read_to_end(&mut data)?;
		
		let format = DFormat::from(header.format.clone());
		let bitcount = format.bitcount();
		let mut decompressed = Vec::with_capacity((data.len() as f32 * 4.0 / (bitcount as f32 / 8.0)) as usize);
		
		for mip in 0..header.mip_levels.max(1) {
			for depth in 0..header.depths.max(1) {
				if let Some((w, h, data)) = Self::slice_manual(mip, depth, &header, bitcount, &data) {
					if w == 0 {break}
					decompressed.extend(format.convert_from(w as usize, h as usize, data).ok_or("invalid format")?);
				}
			}
		}
		
		Ok(Tex {
			data: decompressed,
			header
		})
	}
	
	pub fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.header.write_le(writer)?;
		writer.write_all(&DFormat::from(self.header.format).convert_to(self.header.width as usize, self.header.height as usize, &self.data).ok_or("invalid format")?)?;
		Ok(())
	}
}

impl Dds for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		// TODO: dont unwrap, return a result
		reader.seek(SeekFrom::Start(12))?;
		let height = reader.read_le::<u32>()? as u16;
		let width = reader.read_le::<u32>()? as u16;
		reader.seek(SeekFrom::Current(4))?;
		let depths = 1.max(reader.read_le::<u32>()? as u16);
		let mip_levels = reader.read_le::<u32>()? as u16;
		
		let format = DFormat::get(reader);
		// println!("{format:?}");
		
		// im sure theres some fancier way but w/e
		let mut mip_offsets = [0u32; 13];
		let mut mip_offset = 80;
		for i in 0..13 {
			mip_offsets[i] = if (i as u16) < mip_levels {
				mip_offset
			} else {
				0
			};
			
			mip_offset += ((width as u32 * height as u32 * 4) as f32 * (0.25f32.powi(i as i32))) as u32;
		}
		
		reader.seek(SeekFrom::End(0))?;
		let mut data = Vec::with_capacity(reader.stream_position()? as usize);
		reader.seek(SeekFrom::Start(128))?;
		reader.read_to_end(&mut data)?;
		
		Ok(Tex {
			data: format.convert_from(width as usize, height as usize, &data).ok_or("invalid format")?,
			header: Header {
				flags: 0x00800000, // TODO: care about other stuff like 3d textures
				format: Format::from(format),
				width,
				height,
				depths,
				mip_levels,
				lod_offsets: [0, 1, 2],
				mip_offsets,
			}
		})
	}
	
	// TODO: gotta use those results...
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let format = DFormat::from(self.header.format);
		
		"DDS ".as_bytes().write_le(writer)?;
		124u32.write_le(writer)?;
		(format.flags() | if self.header.mip_levels > 1 {0x2000} else {0}).write_le(writer)?;
		(self.header.height as u32).write_le(writer)?;
		(self.header.width as u32).write_le(writer)?;
		0u32.write_le(writer)?; // most software calculate the pitch itself, so eh fuck it
		0u32.write_le(writer)?;
		(self.header.mip_levels as u32).write_le(writer)?;
		"Noumenon v1".as_bytes().write_le(writer)?; // combines with the one below should total 44 bytes (reserved)
		[0u8; 33].write_le(writer)?;
		32u32.write_le(writer)?;
		format.flags2().write_le(writer)?;
		format.fourcc().write_le(writer)?;
		format.bitcount().write_le(writer)?;
		let (b, g, r, a) = format.masks();
		r.write_le(writer)?;
		g.write_le(writer)?;
		b.write_le(writer)?;
		a.write_le(writer)?;
		0x1000u32.write_le(writer)?; // TODO: the 2 other flags
		[0u32; 4].write_le(writer)?;
		format.convert_to(self.header.width as usize, self.header.height as usize, &self.data).ok_or("invalid format")?.write_le(writer)?;
		
		Ok(())
	}
}

impl Png for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let img = image::io::Reader::with_format(BufReader::new(reader), image::ImageFormat::Png)
			.decode()?;
		
		Ok(Tex {
			header: Header {
				flags: 0x00800000,
				format: Format::A8R8G8B8,
				width: img.width() as u16,
				height: img.height() as u16,
				depths: 0,
				mip_levels: 1,
				lod_offsets: [0, 1, 2],
				mip_offsets: [80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			},
			data: img.into_rgba8().into_vec(),
			// data: img.into_rgba8()
			// 	.chunks_exact(4)
			// 	.flat_map(|p| [p[2], p[1], p[0], p[3]])
			// 	.collect::<Vec<u8>>(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let img = PngEncoder::new(writer);
		// TODO: possibly convert to a different colortype based on header format, idk
		img.write_image(
			&self.data[0..(self.header.width as usize * self.header.height as usize * 4)],
				// .chunks_exact(4)
				// .flat_map(|p| [p[2], p[1], p[0], p[3]])
				// .collect::<Vec<u8>>(),
			self.header.width as u32,
			self.header.height as u32,
			ColorType::Rgba8
		)?;
		
		Ok(())
	}
}

impl Tiff for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let img = image::io::Reader::with_format(BufReader::new(reader), image::ImageFormat::Tiff)
			.decode()?;
		
		Ok(Tex {
			header: Header {
				flags: 0x00800000,
				format: Format::A8R8G8B8,
				width: img.width() as u16,
				height: img.height() as u16,
				depths: 0,
				mip_levels: 1,
				lod_offsets: [0, 1, 2],
				mip_offsets: [80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			},
			data: img.into_rgba8().into_vec(),
			// data: img.into_rgba8()
			// 	.chunks_exact(4)
			// 	.flat_map(|p| [p[2], p[1], p[0], p[3]])
			// 	.collect::<Vec<u8>>(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let img = TiffEncoder::new(writer);
		// TODO: possibly convert to a different colortype based on header format, idk
		img.write_image(
			&self.data[0..(self.header.width as usize * self.header.height as usize * 4)],
				// .chunks_exact(4)
				// .flat_map(|p| [p[2], p[1], p[0], p[3]])
				// .collect::<Vec<u8>>(),
			self.header.width as u32,
			self.header.height as u32,
			ColorType::Rgba8
		)?;
		
		Ok(())
	}
}

impl Tga for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where 
	T: Read + Seek {
		let img = image::io::Reader::with_format(BufReader::new(reader), image::ImageFormat::Tga)
			.decode()?;
		
		Ok(Tex {
			header: Header {
				flags: 0x00800000,
				format: Format::A8R8G8B8,
				width: img.width() as u16,
				height: img.height() as u16,
				depths: 0,
				mip_levels: 1,
				lod_offsets: [0, 1, 2],
				mip_offsets: [80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			},
			data: img.into_rgba8().into_vec(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let img = TgaEncoder::new(writer);
		img.write_image(
			&self.data[0..(self.header.width as usize * self.header.height as usize * 4)],
			self.header.width as u32,
			self.header.height as u32,
			ColorType::Rgba8
		)?;
		
		Ok(())
	}
}