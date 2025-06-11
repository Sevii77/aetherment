use std::io::{Read, Seek, Write};

pub const EXT: &'static [&'static str] = &["tiff", "tif"];

pub trait Tiff<E: std::error::Error> {
	fn read<T>(reader: &mut T) -> Result<Self, E> where Self: Sized, T: Read + Seek;
	fn write<T>(&self, writer: &mut T) -> Result<(), E> where T: Write + Seek;
}