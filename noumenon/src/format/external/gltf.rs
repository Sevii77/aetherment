use std::io::{Read, Seek, Write};
use gltf::json;

pub const EXT: &'static [&'static str] = &["gltf"];

pub trait Gltf<E: std::error::Error> {
	fn read<T>(reader: &mut T) -> Result<Self, E> where Self: Sized, T: Read + Seek;
	fn write<T>(&self, writer: &mut T) -> Result<(), E> where T: Write + Seek;
}

pub fn create_test_model() -> Vec<u8> {
	
	
	Vec::new()
}