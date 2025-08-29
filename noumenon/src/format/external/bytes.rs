use std::io::{Read, Seek, Write};

pub trait Bytes {
	fn read<T>(reader: &mut T) -> Result<Self, crate::Error> where Self: Sized, T: Read + Seek;
	fn write<T>(&self, writer: &mut T) -> Result<(), crate::Error> where T: Write + Seek;
}