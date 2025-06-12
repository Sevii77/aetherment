use std::io::{BufReader, Read, Seek, Write};
use binrw::{binrw, BinRead, BinWrite};
use image::ImageEncoder;
use image_dds::ImageFormat;

pub const EXT: &'static [&'static str] = &["tex", "atex"];

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0:?}")] Binrw(#[from] binrw::Error),
	#[error("{0:?}")] Dds(#[from] image_dds::ddsfile::Error),
	#[error("{0:?}")] DdsCreate(#[from] image_dds::CreateDdsError),
	#[error("{0:?}")] DdsSurface(#[from] image_dds::error::SurfaceError),
	#[error("{0:?}")] Image(#[from] image::ImageError),
}

pub struct Slice<'a> {
	pub width: u32,
	pub height: u32,
	pub pixels: &'a [u8],
}

pub struct SliceMut<'a> {
	pub width: u32,
	pub height: u32,
	pub pixels: &'a mut [u8],
}

#[binrw]
#[brw(little, repr = u32)]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Format {
	L8 = 0x1130,
	
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
}

impl From<Format> for ImageFormat {
	fn from(value: Format) -> Self {
		match value {
			Format::L8           => ImageFormat::R8Unorm,
			Format::A4R4G4B4     => ImageFormat::Bgra4Unorm,
			Format::A1R5G5B5     => ImageFormat::Bgr5A1Unorm,
			Format::A8R8G8B8     => ImageFormat::Bgra8Unorm,
			Format::X8R8G8B8     => ImageFormat::Bgra8Unorm,
			Format::R32          => ImageFormat::R32Float,
			Format::R16G16       => ImageFormat::Rg16Float,
			Format::R32G32       => ImageFormat::Rg32Float,
			Format::A16B16G16R16 => ImageFormat::Rgba16Float,
			Format::A32B32G32R32 => ImageFormat::Rgba32Float,
			Format::Bc1          => ImageFormat::BC1RgbaUnorm,
			Format::Bc2          => ImageFormat::BC2RgbaUnorm,
			Format::Bc3          => ImageFormat::BC3RgbaUnorm,
			Format::Bc5          => ImageFormat::BC5RgUnorm,
			Format::Bc7          => ImageFormat::BC7RgbaUnorm,
		}
	}
}

impl Format {
	pub fn convert_from(&self, width: u32, height: u32, depth: u32, data: &[u8]) -> Vec<u8> {
		let surface = image_dds::Surface {
			width,
			height,
			depth,
			layers: 1,
			mipmaps: 1,
			image_format: (*self).into(),
			data,
		};
		
		surface.decode_rgba8().unwrap().data
	}
	
	pub fn convert_to(&self, width: u32, height: u32, depth: u32, data: &[u8]) -> Vec<u8> {
		let surface = image_dds::SurfaceRgba8 {
			width,
			height,
			depth,
			layers: 1,
			mipmaps: 1,
			data,
		};
		
		surface.encode((*self).into(), image_dds::Quality::Normal, image_dds::Mipmaps::Disabled).unwrap().data
	}
}

#[derive(Debug, Clone)]
pub struct Tex {
	pub flags: u32,
	pub format: Format,
	pub width: u32,
	pub height: u32,
	pub depth: u32,
	pub mip_levels: u32,
	pub lods: bool,
	
	pub pixels: Vec<u8>,
}

impl Tex {
	pub fn slice<'a>(&'a self, depth: u32, mip: u32) -> Slice<'a> {
		let (width, height, offset, size) = self.slice_range(depth, mip);
		
		Slice {
			width,
			height,
			pixels: &self.pixels[offset..offset + size],
		}
	}
	
	pub fn slice_mut<'a>(&'a mut self, depth: u32, mip: u32) -> SliceMut<'a> {
		let (width, height, offset, size) = self.slice_range(depth, mip);
		
		SliceMut {
			width,
			height,
			pixels: &mut self.pixels[offset..offset + size],
		}
	}
	
	fn slice_range(&self, depth: u32, mip: u32) -> (u32, u32, usize, usize) {
		let mip = mip as u32;
		let width = self.width / 2u32.pow(mip);
		let height = self.height / 2u32.pow(mip);
		(
			width as u32, height as u32,
			(self.width as u64 * self.height as u64 * self.depth as u64 * (8u64.pow(mip) - 1) / if mip == 0 {1} else {7 * 8u64.pow(mip - 1)} + width as u64 * height as u64 * depth as u64 * 4) as usize,
			(width * height * 4) as usize,
		)
	}
	
	pub fn new(width: u32, height: u32, pixels: impl Into<Vec<u8>>) -> Self {
		let pixels = pixels.into();
		assert!(width * height * 4 <= pixels.len() as u32, "Pixel buffer was too small");
		
		Tex {
			flags: 0x00800000,
			format: Format::A8R8G8B8,
			width: width,
			height: height,
			depth: 1,
			mip_levels: 1,
			lods: true,
			pixels: pixels,
		}
	}
}

impl BinRead for Tex {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let flags = u32::read_options(reader, endian, ())?;
		let format = Format::read_options(reader, endian, ())?;
		let width = u16::read_options(reader, endian, ())? as u32;
		let height = u16::read_options(reader, endian, ())? as u32;
		let depth = (u16::read_options(reader, endian, ())? as u32).max(1);
		let mut mip_levels = (u16::read_options(reader, endian, ())? as u32).max(1);
		let lod_offsets = <[u32; 3]>::read_options(reader, endian, ())?;
		let mip_offsets = <[u32; 13]>::read_options(reader, endian, ())?;
		let mut data = Vec::new();
		reader.read_to_end(&mut data)?;
		
		// mips seem to be broken?
		let mut pixels = Vec::new();
		for mip_level in 0..mip_levels as usize {
			let offset = mip_offsets[mip_level] as usize - 80;
			let next_offset = mip_offsets.get(mip_level + 1).map(|v| if *v == 0 {data.len()} else {*v as usize - 80}).unwrap_or(data.len());
			let factor = 2u32.pow(mip_level as u32);
			let width = width / factor;
			let height = height / factor;
			let depth = if depth == 1 {depth} else {depth / factor};
			if width == 0 || height == 0 || depth == 0 {
				mip_levels = mip_level as u32;
				break;
			};
			pixels.extend(format.convert_from(width, height, depth, &data[offset..next_offset]));
		}
		
		Ok(Self {
			flags,
			format,
			width,
			height,
			depth,
			mip_levels,
			lods: lod_offsets[2] != 0,
			pixels,
		})
	}
}

impl BinWrite for Tex {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		self.flags.write_options(writer, endian, ())?;
		self.format.write_options(writer, endian, ())?;
		(self.width as u16).write_options(writer, endian, ())?;
		(self.height as u16).write_options(writer, endian, ())?;
		(self.depth as u16).write_options(writer, endian, ())?;
		(self.mip_levels as u16).write_options(writer, endian, ())?;
		if self.lods {[0u32, 1, 2]} else {[0u32; 3]}.write_options(writer, endian, ())?;
		let mut mip_offsets = [0u32; 13];
		for i in 0..self.mip_levels as usize {
			mip_offsets[i] = 80 + self.width as u32 * self.height as u32 * self.depth as u32 * (8u32.pow(i as u32) - 1) / if i == 0 {1} else {7 * 8u32.pow(i as u32 - 1)};
		}
		mip_offsets.write_options(writer, endian, ())?;
		writer.write_all(&self.format.convert_to(self.width, self.height, self.depth, &self.pixels))?;
		
		Ok(())
	}
}

impl ironworks::file::File for Tex {
	fn read(mut data: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
		Tex::read_le(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

// ----------

impl crate::format::external::Bytes<Error> for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		Ok(Tex::read_le(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		Ok(self.write_le(writer)?)
	}
}

impl crate::format::external::Dds<Error> for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let dds = image_dds::ddsfile::Dds::read(reader)?;
		let data = image_dds::Surface::from_dds(&dds)?;
		
		Ok(Self {
			flags: 0x00800000, // TODO: care about other stuff like 3d textures
			format: match data.image_format {
				 ImageFormat::R8Unorm      => Format::L8,
				 ImageFormat::Bgra4Unorm   => Format::A4R4G4B4,
				 ImageFormat::Bgr5A1Unorm  => Format::A1R5G5B5,
				 ImageFormat::Bgra8Unorm   => Format::A8R8G8B8,
				 ImageFormat::R32Float     => Format::R32,
				 ImageFormat::Rg16Float    => Format::R16G16,
				 ImageFormat::Rg32Float    => Format::R32G32,
				 ImageFormat::Rgba16Float  => Format::A16B16G16R16,
				 ImageFormat::Rgba32Float  => Format::A32B32G32R32,
				 ImageFormat::BC1RgbaUnorm => Format::Bc1,
				 ImageFormat::BC2RgbaUnorm => Format::Bc2,
				 ImageFormat::BC3RgbaUnorm => Format::Bc3,
				 ImageFormat::BC5RgUnorm   => Format::Bc5,
				 ImageFormat::BC7RgbaUnorm => Format::Bc7,
				 _                         => Format::A8R8G8B8,
			},
			width: dds.header.width,
			height: dds.header.height,
			depth: dds.header.depth.unwrap_or(1),
			mip_levels: dds.header.mip_map_count.unwrap_or(1),
			lods: true,
			pixels: data.decode_rgba8()?.data,
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let surface = image_dds::SurfaceRgba8 {
			width: self.width as u32,
			height: self.height as u32,
			depth: self.depth as u32,
			layers: 1,
			mipmaps: 1,
			data: &self.pixels,
		};
		
		surface.encode_dds(self.format.into(), image_dds::Quality::Normal, image_dds::Mipmaps::Disabled)?
			.write(writer)?;
		
		Ok(())
	}
}

impl crate::format::external::Png<Error> for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let img = image::ImageReader::with_format(BufReader::new(reader), image::ImageFormat::Png)
			.decode()?;
		
		Ok(Self {
			flags: 0x00800000,
			format: Format::A8R8G8B8,
			width: img.width(),
			height: img.height(),
			depth: 1,
			mip_levels: 1,
			lods: true,
			pixels: img.into_rgba8().into_vec(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let img = image::codecs::png::PngEncoder::new(writer);
		img.write_image(
			&self.pixels[0..(self.width * self.height * 4) as usize],
			self.width as u32,
			self.height as u32,
			image::ColorType::Rgba8.into()
		)?;
		
		Ok(())
	}
}

impl crate::format::external::Tiff<Error> for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let img = image::ImageReader::with_format(BufReader::new(reader), image::ImageFormat::Tiff)
			.decode()?;
		
		Ok(Self {
			flags: 0x00800000,
			format: Format::A8R8G8B8,
			width: img.width(),
			height: img.height(),
			depth: 1,
			mip_levels: 1,
			lods: true,
			pixels: img.into_rgba8().into_vec(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let img = image::codecs::tiff::TiffEncoder::new(writer);
		img.write_image(
			&self.pixels[0..(self.width * self.height * 4) as usize],
			self.width as u32,
			self.height as u32,
			image::ColorType::Rgba8.into()
		)?;
		
		Ok(())
	}
}

impl crate::format::external::Tga<Error> for Tex {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		let img = image::ImageReader::with_format(BufReader::new(reader), image::ImageFormat::Tga)
			.decode()?;
		
		Ok(Self {
			flags: 0x00800000,
			format: Format::A8R8G8B8,
			width: img.width(),
			height: img.height(),
			depth: 1,
			mip_levels: 1,
			lods: true,
			pixels: img.into_rgba8().into_vec(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		let img = image::codecs::tga::TgaEncoder::new(writer);
		img.write_image(
			&self.pixels[0..(self.width * self.height * 4) as usize],
			self.width as u32,
			self.height as u32,
			image::ColorType::Rgba8.into()
		)?;
		
		Ok(())
	}
}