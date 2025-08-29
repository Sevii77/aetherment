use std::io::{BufReader, Read, Seek, Write};
use binrw::{binrw, BinRead, BinWrite};
use image::ImageEncoder;

pub const EXT: &'static [&'static str] = &["hwc"];

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Hwc {
	pub pixels: [u8; 64 * 64 * 4],
}

impl Hwc {
	pub const WIDTH: u32 = 64;
	pub const HEIGHT: u32 = 64;
}

impl super::Extension for Hwc {
	const EXT: &[&str] = EXT;
}

// ----------

impl crate::format::external::Bytes for Hwc {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error> where
	T: Read + Seek {
		Ok(Hwc::read_le(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where
	T: Write + Seek {
		Ok(self.write_le(writer)?)
	}
}

impl crate::format::external::Png for Hwc {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error> where
	T: Read + Seek {
		let img = image::ImageReader::with_format(BufReader::new(reader), image::ImageFormat::Png)
			.decode()?;
		
		Ok(Self {
			pixels: img.into_rgba8().into_vec()[..64 * 64 * 4].try_into().unwrap(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where
	T: Write + Seek {
		let img = image::codecs::png::PngEncoder::new(writer);
		img.write_image(&self.pixels, 64, 64, image::ColorType::Rgba8.into())?;
		
		Ok(())
	}
}

impl crate::format::external::Tiff for Hwc {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error> where
	T: Read + Seek {
		let img = image::ImageReader::with_format(BufReader::new(reader), image::ImageFormat::Tiff)
			.decode()?;
		
		Ok(Self {
			pixels: img.into_rgba8().into_vec()[..64 * 64 * 4].try_into().unwrap(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where
	T: Write + Seek {
		let img = image::codecs::tiff::TiffEncoder::new(writer);
		img.write_image(&self.pixels, 64, 64, image::ColorType::Rgba8.into())?;
		
		Ok(())
	}
}

impl crate::format::external::Tga for Hwc {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error> where
	T: Read + Seek {
		let img = image::ImageReader::with_format(BufReader::new(reader), image::ImageFormat::Tga)
			.decode()?;
		
		Ok(Self {
			pixels: img.into_rgba8().into_vec()[..64 * 64 * 4].try_into().unwrap(),
		})
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where
	T: Write + Seek {
		let img = image::codecs::tga::TgaEncoder::new(writer);
		img.write_image(&self.pixels, 64, 64, image::ColorType::Rgba8.into())?;
		
		Ok(())
	}
}