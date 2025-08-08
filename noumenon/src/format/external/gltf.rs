use std::{collections::HashMap, io::{Read, Seek, Write}};

pub const EXT: &'static [&'static str] = &["gltf"];

#[derive(Debug, Clone)]
pub struct MaterialBake {
	pub diffuse: Option<MaterialBakeTexture>,
	pub normal: Option<MaterialBakeTexture>,
}

#[derive(Debug, Clone)]
pub struct MaterialBakeTexture {
	pub width: u32,
	pub height: u32,
	pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Bone {
	pub name: String,
	pub parent: Option<String>,
	pub translation: glam::Vec3,
	pub rotation: glam::Quat,
	pub scale: glam::Vec3,
}

pub trait Gltf<E: std::error::Error> {
	fn read<T>(reader: &mut T) -> Result<Self, E> where Self: Sized, T: Read + Seek;
	fn write<T>(&self, writer: &mut T, materials: HashMap<String, MaterialBake>, bones: Vec<Bone>) -> Result<(), E> where T: Write + Seek;
}