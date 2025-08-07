use std::{fmt::Debug, io::{Read, Seek, Write}};
use binrw::{BinRead, BinWrite};

pub const EXT: &'static [&'static str] = &["sklb"];

pub type Error = binrw::Error;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Sklb {
	pub bones: Vec<Bone>,
}

impl BinRead for Sklb {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		simple_reader!(reader, endian);
		
		let _magic = r!(u32);
		let version = r!(u32);
		let new_header = version >= 0x31333030; // 1300
		r!(move if new_header {4} else {2});
		let offset = if new_header {r!(u32)} else {r!(u16) as u32};
		r!(seek offset);
		let havok_data = r!(eof);
		
		let root = crate::havok::HavokBinaryTagFileReader::read(&havok_data);
		let anim = crate::havok::HavokAnimationContainer::new(root.find_object_by_type("hkaAnimationContainer"));
		let skel = &anim.skeletons[0];
		
		let mut bones = Vec::new();
		for (i, bone) in skel.bone_names.iter().enumerate() {
			bones.push(Bone {
				name: bone.to_string(),
				parent: skel.parent_indices[i] as i32,
				translation: glam::Vec3::from_array(skel.reference_pose[i].translation[0..3].try_into().unwrap()),
				rotation: glam::Quat::from_array(skel.reference_pose[i].rotation),
				scale: glam::Vec3::from_array(skel.reference_pose[i].scale[0..3].try_into().unwrap()),
			});
		}
		
		Ok(Self {
			bones,
		})
	}
}

impl BinWrite for Sklb {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		todo!();
	}
}

impl ironworks::file::File for Sklb {
	fn read(mut data: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
		Sklb::read_le(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

impl crate::format::external::Bytes<Error> for Sklb {
	fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		Ok(Sklb::read_le(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.write_le(writer)?;
		
		Ok(())
	}
}

// ----------

#[derive(Debug, Clone)]
pub struct Bone {
	pub name: String,
	pub parent: i32,
	pub translation: glam::Vec3,
	pub rotation: glam::Quat,
	pub scale: glam::Vec3,
}