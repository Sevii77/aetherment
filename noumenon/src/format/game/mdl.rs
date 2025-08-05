
use std::{fmt::Debug, io::{Read, Seek, SeekFrom, Write}};
use binrw::{binrw, BinRead, BinWrite};
use glam::Vec4Swizzles;

pub const EXT: &'static [&'static str] = &["mdl"];

pub type Error = binrw::Error;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Mdl {
	pub meshes: Vec<Vec<Mesh>>,
}

impl BinRead for Mdl {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		// let raw = MdlRaw::read_options(reader, endian, ())?;
		
		// let data_pos = raw.header.runtime_size as usize + size_of::<HeaderRaw>() + raw.header.stack_size as usize;
		// let runtime_padding = data_pos as i64 - reader.stream_position()? as i64;
		// reader.seek_relative(runtime_padding)?;
		
		// println!("{:#?}", raw.vertex_declerations);
		
		macro_rules! r {
			($c:literal) => {{
				reader.seek_relative($c / 8)?
			}};
			
			(Vec<$e:tt>, $c:expr) => {{
				let mut v = Vec::with_capacity($c as usize);
				for _ in 0..$c {
					v.push(<$e>::read_options(reader, endian, ())?);
				}
				v
			}};
			
			($e:tt) => {{
				$e::read_options(reader, endian, ())?
			}};
		}
		
		let header = r!(HeaderRaw);
		let vertex_declerations = r!(Vec<[VertexElementRaw; 17]>, header.vertex_declaration_count);
		
		let strings_count = r!(u16);
		r!(16);
		let strings_size = r!(u32);
		let strings_buf = r!(Vec<u8>, strings_size);
		
		let model_header = r!(ModelHeaderRaw);
		let element_ids = r!(Vec<ElementIdRaw>, model_header.element_id_count);
		let lods = r!(Vec<LodRaw>, header.lod_count);
		let extra_lods = r!(Vec<ExtraLodRaw>, if model_header.flags2.contains(ModelFlags2Raw::EXTRA_LOD_ENABLED) {header.lod_count} else {0});
		let meshes = r!(Vec<MeshRaw>, model_header.mesh_count);
		let attribute_string_offset = r!(Vec<u32>, model_header.attribute_count);
		let submeshes = r!(Vec<SubmeshRaw>, model_header.submesh_count);
		let terrain_shadow_submeshes = r!(Vec<TerrainShadowSubmeshRaw>, model_header.terrain_shadow_submesh_count);
		let material_string_offset = r!(Vec<u32>, model_header.material_count);
		let bone_string_offset = r!(Vec<u32>, model_header.bone_count);
		let bone_table = bone_table_reader(reader, endian, (header.version, model_header.bone_table_count, model_header.bone_table_array_count_total))?;
		let shapes = r!(Vec<ShapeRaw>, model_header.shape_count);
		let shape_meshes = r!(Vec<ShapeMeshRaw>, model_header.shape_mesh_count);
		let shape_values = r!(Vec<ShapeValueRaw>, model_header.shape_value_count);
		
		let submesh_bone_map_size = r!(u32);
		let submesh_bone_map = r!(Vec<u16>, submesh_bone_map_size / 2);
		
		let _padding_size = r!(u8);
		let _padding = r!(Vec<u8>, _padding_size);
		
		let bb = r!(BoundingBoxRaw);
		let model_bb = r!(BoundingBoxRaw);
		let water_bb = r!(BoundingBoxRaw);
		let vertical_fog_bb = r!(BoundingBoxRaw);
		let bones_bb = r!(Vec<BoundingBoxRaw>, model_header.bone_count);
		
		let mut lods_new = Vec::new();
		for (lod_index, lod_raw) in lods.iter().enumerate() {
			let mut meshes_new = Vec::new();
			for mesh_index in lod_raw.mesh_index as usize..(lod_raw.mesh_index + lod_raw.mesh_count) as usize {
				let mesh_raw = &meshes[mesh_index];
				
				// vertices
				let mut vertices = vec![Vertex::default(); mesh_raw.vertex_count as usize];
				let vertex_decl = &vertex_declerations[mesh_index];
				for stream in 0..mesh_raw.vertex_stream_count {
					let offset = header.vertex_offsets[lod_index] as u64 + mesh_raw.vertex_buffer_offset[stream as usize] as u64;
					reader.seek(SeekFrom::Start(offset))?;
					
					for vertex_index in 0..mesh_raw.vertex_count as usize {
						let vertex = &mut vertices[vertex_index];
						for decl in vertex_decl {
							if decl.stream == 255 {break}
							if decl.stream != stream {continue}
							
							macro_rules! r {
								(f16) => {{
									half::f16::from_bits(u16::read_options(reader, endian, ())?).to_f32()
								}};
								
								($e:tt) => {{
									$e::read_options(reader, endian, ())? as f32
								}};
							}
							
							let val = match decl.typ {
								VertexTypeRaw::F32x1 => glam::vec4(r!(f32), 0.0, 0.0, 0.0),
								VertexTypeRaw::F32x2 => glam::vec4(r!(f32), r!(f32), 0.0, 0.0),
								VertexTypeRaw::F32x3 => glam::vec4(r!(f32), r!(f32), r!(f32), 0.0),
								VertexTypeRaw::F32x4 => glam::vec4(r!(f32), r!(f32), r!(f32), r!(f32)),
								VertexTypeRaw::U8x4  => glam::vec4(r!(u8), r!(u8), r!(u8), r!(u8)),
								VertexTypeRaw::F8x4  => glam::vec4(r!(u8) / 255.0, r!(u8) / 255.0, r!(u8) / 255.0, r!(u8) / 255.0),
								VertexTypeRaw::F16x2 => glam::vec4(r!(f16), r!(f16), 0.0, 0.0),
								VertexTypeRaw::F16x4 => glam::vec4(r!(f16), r!(f16), r!(f16), r!(f16)),
								// VertexTypeRaw::U16x2 => [r!(u16), r!(u16), 0.0, 0.0],
								// VertexTypeRaw::U16x4 => [r!(u16), r!(u16), r!(u16), r!(u16)],
							};
							
							match decl.usage {
								VertexUsageRaw::Position =>
									vertex.position = val.xyz(),
								
								VertexUsageRaw::BlendWeights =>
									for i in 0..4 {vertex.blends[i].weight = val[i] / 255.0},
								
								VertexUsageRaw::BlendIndices =>
									for i in 0..4 {vertex.blends[i].bone = val[i] as u8},
								
								VertexUsageRaw::Normal =>
									vertex.normal = val.xyz(),
								
								VertexUsageRaw::Uv =>
									vertex.uv = val,
								
								// VertexUsageRaw::Tangent2 =>
								// 	vertex.position = val[0..3].try_into().unwrap(),
								
								VertexUsageRaw::Tangent1 =>
									vertex.tangent = val,
								
								VertexUsageRaw::Color =>
									vertex.color = val,
								_ => {}
							}
						}
					}
				}
				
				// indices
				let offset = header.index_offsets[lod_index] as u64 + mesh_raw.start_index as u64 * 2;
				reader.seek(SeekFrom::Start(offset))?;
				
				let mut indices = Vec::with_capacity(mesh_raw.index_count as usize);
				for _ in 0..mesh_raw.index_count {
					indices.push(u16::read_options(reader, endian, ())?);
				}
				
				//
				meshes_new.push(Mesh {
					vertices,
					indices,
				});
			}
			
			lods_new.push(meshes_new);
		}
		
		Ok(Self {
			meshes: lods_new,
			// _raw: raw,
		})
	}
}

impl BinWrite for Mdl {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		todo!();
	}
}

impl ironworks::file::File for Mdl {
	fn read(mut data: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
		Mdl::read_le(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

impl crate::format::external::Bytes<Error> for Mdl {
	fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		Ok(Mdl::read_le(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.write_le(writer)?;
		
		Ok(())
	}
}

// ---------------------------------------- //

#[derive(Debug, Clone)]
pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct Vertex {
	pub position: glam::Vec3,
	pub normal: glam::Vec3,
	pub tangent: glam::Vec4,
	pub uv: glam::Vec4,
	pub color: glam::Vec4,
	pub blends: [Blend; 4],
}

#[derive(Debug, Clone, Default)]
pub struct Blend {
	pub bone: u8,
	pub weight: f32,
}

// ---------------------------------------- //

// #[binrw]
// #[brw(little)]
// #[derive(Debug, Clone)]
// pub struct MdlRaw {
// 	pub header: HeaderRaw,
// 	
// 	#[br(count = header.vertex_declaration_count)]
// 	pub vertex_declerations: Vec<[VertexElementRaw; 17]>,
// 	
// 	pub strings_count: u16,
// 	pub _padding: u16,
// 	pub strings_size: u32,
// 	#[br(count = strings_size)]
// 	pub strings_buf: Vec<u8>,
// 	
// 	pub model_header: ModelHeaderRaw,
// 	
// 	#[br(count = model_header.element_id_count)]
// 	pub element_ids: Vec<ElementIdRaw>,
// 	
// 	#[br(count = header.lod_count)]
// 	pub lods: Vec<LodRaw>,
// 	
// 	#[brw(if(model_header.flags2.contains(ModelFlags2Raw::EXTRA_LOD_ENABLED)))]
// 	#[br(count = header.lod_count)]
// 	pub extra_lods: Vec<ExtraLodRaw>,
// 	
// 	#[br(count = model_header.mesh_count)]
// 	pub meshes: Vec<MeshRaw>,
// 	
// 	#[br(count = model_header.attribute_count)]
// 	pub attribute_string_offset: Vec<u32>,
// 	
// 	#[br(count = model_header.submesh_count)]
// 	pub submeshes: Vec<SubmeshRaw>,
// 	
// 	#[br(count = model_header.terrain_shadow_submesh_count)]
// 	pub terrain_shadow_submeshes: Vec<TerrainShadowSubmeshRaw>,
// 	
// 	#[br(count = model_header.material_count)]
// 	pub material_string_offset: Vec<u32>,
// 	
// 	#[br(count = model_header.bone_count)]
// 	pub bone_string_offset: Vec<u32>,
// 	
// 	#[br(parse_with = bone_table_reader, args(header.version, model_header.bone_table_count, model_header.bone_table_array_count_total))]
// 	#[bw(write_with = bone_table_writer, args(header.version))]
// 	pub bone_table: Vec<Vec<u16>>,
// 	
// 	#[br(count = model_header.shape_count)]
// 	pub shapes: Vec<ShapeRaw>,
// 	
// 	#[br(count = model_header.shape_mesh_count)]
// 	pub shape_meshes: Vec<ShapeMeshRaw>,
// 	
// 	#[br(count = model_header.shape_value_count)]
// 	pub shape_values: Vec<ShapeValueRaw>,
// 	
// 	pub submesh_bone_map_size: u32,
// 	#[br(count = submesh_bone_map_size / 2)]
// 	pub submesh_bone_map: Vec<u16>,
// 	
// 	pub _padding2_size: u8,
// 	#[br(count = _padding2_size)]
// 	pub _padding2: Vec<u8>,
// 	
// 	pub bb: BoundingBoxRaw,
// 	pub model_bb: BoundingBoxRaw,
// 	pub water_bb: BoundingBoxRaw,
// 	pub vertical_fog_bb: BoundingBoxRaw,
// 	#[br(count = model_header.bone_count)]
// 	pub bones_bb: Vec<BoundingBoxRaw>,
// }

#[binrw]
#[derive(Debug, Clone)]
pub struct HeaderRaw {
	pub version: u32,
	pub stack_size: u32,
	pub runtime_size: u32,
	pub vertex_declaration_count: u16,
	pub material_count: u16,
	pub vertex_offsets: [u32; 3],
	pub index_offsets: [u32; 3],
	pub vertex_buffer_offsets: [u32; 3],
	pub index_buffer_offsets: [u32; 3],
	pub lod_count: u8,
	pub index_buffer_streaming: u8,
	pub edge_geometry: u8,
	pub _padding: u8,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct VertexElementRaw {
	pub stream: u8,
	pub offset: u8,
	pub typ: VertexTypeRaw,
	pub usage: VertexUsageRaw,
	pub usage_index: u8,
	pub _padding: [u8; 3],
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum VertexTypeRaw {
	F32x1 = 0,
	F32x2 = 1,
	F32x3 = 2,
	F32x4 = 3,
	U8x4  = 5,
	F8x4  = 8,
	F16x2 = 13,
	F16x4 = 14,
	// U16x2 = 16,
	// U16x4 = 17,
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum VertexUsageRaw {
	Position     = 0,
	BlendWeights = 1,
	BlendIndices = 2,
	Normal       = 3,
	Uv           = 4,
	Tangent2     = 5,
	Tangent1     = 6,
	Color        = 7,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ModelHeaderRaw {
	pub radius: f32,
	pub mesh_count: u16,
	pub attribute_count: u16,
	pub submesh_count: u16,
	pub material_count: u16,
	pub bone_count: u16,
	pub bone_table_count: u16,
	pub shape_count: u16,
	pub shape_mesh_count: u16,
	pub shape_value_count: u16,
	pub lod_count: u8,
	#[br(map = |v: u8| ModelFlags1Raw::from_bits(v).unwrap())]
	#[bw(map = |v: &ModelFlags1Raw| v.bits())]
	pub flags1: ModelFlags1Raw,
	pub element_id_count: u16,
	pub terrain_shadow_mesh_count: u8,
	#[br(map = |v: u8| ModelFlags2Raw::from_bits(v).unwrap())]
	#[bw(map = |v: &ModelFlags2Raw| v.bits())]
	pub flags2: ModelFlags2Raw,
	pub model_clip_out_distance: f32,
	pub shadow_clip_out_distance: f32,
	pub culling_grid_count: u16,
	pub terrain_shadow_submesh_count: u16,
	pub flags3: u8, // ?
	pub bg_change_material_index: u8,
	pub bg_crest_change_material_index: u8,
	pub unknown6: u8,
	pub bone_table_array_count_total: u16,
	pub unknown8: u16,
	pub unknown9: u16,
	pub _padding: [u8; 6],
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	pub struct ModelFlags1Raw: u8 {
		const DUST_OCCLUSION_ENABLED = 0x80;
		const SNOW_OCCLUSION_ENABLED = 0x40;
		const RAIN_OCCLUSION_ENABLED = 0x20;
		const UNKNOWN1 = 0x10;
		const LIGHTING_REFLECTION_ENABLED = 0x08;
		const WAVING_ANIMATION_DISABLED = 0x04;
		const LIGHT_SHADOW_DISABLED = 0x02;
		const SHADOW_DISABLED = 0x01;
	}
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	pub struct ModelFlags2Raw: u8 {
		const UNKNOWN2 = 0x80;
		const BG_UV_SCROLL_ENABLED = 0x40;
		const FORCE_NON_RESIDENT_ENABLED = 0x20;
		const EXTRA_LOD_ENABLED = 0x10;
		const SHADOW_MASK_ENABLED = 0x08;
		const FORCE_LOD_RANGE_ENABLED = 0x04;
		const EDGE_GEOMETRY_ENABLED = 0x02;
		const UNKINWO3 = 0x0;
	}
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ElementIdRaw {
	pub element_id: u32,
	pub parent_bone_name: u32,
	pub translation: [f32; 3],
	pub rotation: [f32; 3],
}

#[binrw]
#[derive(Debug, Clone)]
pub struct LodRaw {
	pub mesh_index: u16,
	pub mesh_count: u16,
	pub model_lod_range: f32,
	pub texture_load_range: f32,
	pub water_mesh_index: u16,
	pub water_mesh_count: u16,
	pub shadow_mesh_index: u16,
	pub shadow_mesh_count: u16,
	pub terrain_shadow_mesh_index: u16,
	pub terrain_shadow_mesh_count: u16,
	pub vertical_fog_mesh_index: u16,
	pub vertical_fog_mesh_count: u16,
	
	pub edge_geometry_size: u32,
	pub edge_geometry_data_offset: u32,
	pub polygon_count: u32,
	pub unknown1: u32,
	pub vertex_buffer_size: u32,
	pub index_buffer_size: u32,
	pub vertex_data_offset: u32,
	pub index_data_offset: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ExtraLodRaw {
	pub lightshaft_mesh_index: u16,
	pub lightshaft_mesh_count: u16,
	pub glass_mesh_index: u16,
	pub glass_mesh_count: u16,
	pub material_change_mesh_index: u16,
	pub material_change_mesh_count: u16,
	pub crest_change_mesh_index: u16,
	pub crest_change_mesh_count: u16,
	pub unknown1: u16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub unknown4: u16,
	pub unknown5: u16,
	pub unknown6: u16,
	pub unknown7: u16,
	pub unknown8: u16,
	pub unknown9: u16,
	pub unknown10: u16,
	pub unknown11: u16,
	pub unknown12: u16,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MeshRaw {
	pub vertex_count: u16,
	pub _padding: u16,
	pub index_count: u32,
	pub material_index: u16,
	pub sub_mesh_index: u16,
	pub sub_mesh_count: u16,
	pub bone_table_index: u16,
	pub start_index: u32,
	pub vertex_buffer_offset: [u32; 3],
	pub vertex_buffer_stride: [u8; 3],
	pub vertex_stream_count: u8,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct TerrainShadowMeshRaw {
	pub index_count: u32,
	pub start_index: u32,
	pub vertex_buffer_offset: u32,
	pub vertex_count: u16,
	pub sub_mesh_index: u16,
	pub sub_mesh_count: u16,
	pub vertex_buffer_stride: u8,
	pub _padding: u8
}

#[binrw]
#[derive(Debug, Clone)]
pub struct SubmeshRaw {
	pub index_offset: u32,
	pub index_count: u32,
	pub attribute_index_mask: u32,
	pub bone_start_index: u16,
	pub bone_count: u16,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct TerrainShadowSubmeshRaw {
	pub index_offset: u32,
	pub index_count: u32,
	pub unknown1: u16,
	pub unknown2: u16,
}

#[binrw::parser(reader, endian)]
fn bone_table_reader(version: u32, count: u16, count_total: u16) -> binrw::BinResult<Vec<Vec<u16>>> {
	match version & 0xFF {
		5 => {
			let mut bones_all = Vec::with_capacity(count as usize);
			for _ in 0..count {
				let bones = binrw::BinReaderExt::read_type::<[u16; 64]>(reader, endian)?;
				let count = u32::read_options(reader, endian, ())?;
				bones_all.push(bones[..count as usize].to_vec())
			}
			
			Ok(bones_all)
		}
		
		6 => {
			let mut bones_all = Vec::with_capacity(count as usize);
			for _ in 0..count {
				let pos = reader.stream_position()?;
				let offset = u16::read_options(reader, endian, ())?;
				let count = u16::read_options(reader, endian, ())?;
				reader.seek(SeekFrom::Start(pos + offset as u64 * 4))?;
				let mut bones = vec![0u16; count as usize];
				for i in 0..count {
					bones[i as usize] = u16::read_options(reader, endian, ())?;
				}
				reader.seek(SeekFrom::Start(pos + 4))?;
				bones_all.push(bones);
			}
			
			reader.seek(SeekFrom::Current(count_total as i64 * 2))?;
			
			Ok(bones_all)
		}
		
		_ => {
			Err(binrw::Error::BadMagic{pos: 0, found: Box::new(version & 0xFF)})
		}
	}
}

#[binrw::writer(writer, endian)]
fn bone_table_writer(bones_all: &Vec<Vec<u16>>, version: u32) -> binrw::BinResult<()> {
	match version & 0xFF {
		5 => {
			for bones in bones_all {
				for v in bones {
					(*v).write_options(writer, endian, ())?;
				}
				
				for _ in bones.len()..64 {
					0u16.write_options(writer, endian, ())?;
				}
				
				(bones.len() as u32).write_options(writer, endian, ())?;
			}
			
			Ok(())
		}
		
		6 => {
			let mut offset = bones_all.len();
			for bones in bones_all {
				(offset as u16).write_options(writer, endian, ())?;
				(bones.len() as u16).write_options(writer, endian, ())?;
				offset += (bones.len() + 1) / 2;
			}
			
			for bones in bones_all {
				for v in bones {
					(*v).write_options(writer, endian, ())?;
				}
				
				if bones.len() % 2 == 1 {
					0u16.write_options(writer, endian, ())?;
				}
			}
			
			Ok(())
		}
		
		_ => {
			Err(binrw::Error::BadMagic{pos: 0, found: Box::new(version & 0xFF)})
		}
	}
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ShapeRaw {
	pub string_offset: u32,
	pub mesh_start_index: [u16; 3],
	pub mesh_count: [u16; 3],
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ShapeMeshRaw {
	pub mesh_index_offset: u32,
	pub value_count: u32,
	pub value_offset: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ShapeValueRaw {
	pub base_indices_index: u16,
	pub replacing_vertex_index: u16,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct BoundingBoxRaw {
	pub min: [f32; 4],
	pub max: [f32; 4],
}