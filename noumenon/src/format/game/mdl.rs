
use std::{collections::HashMap, fmt::Debug, io::{Cursor, Read, Seek, SeekFrom, Write}};
use binrw::{binrw, BinRead, BinWrite};
use glam::Vec4Swizzles;
use crate::{format::external::gltf::{MaterialBake, MaterialBakeTexture}, NullReader};

pub const EXT: &'static [&'static str] = &["mdl"];

pub type Error = binrw::Error;

#[derive(Debug, Clone)]
pub struct Lod {
	pub meshes: Vec<Mesh>,
}

#[derive(Debug, Clone)]
pub struct Mesh {
	pub material: String,
	pub submeshes: Vec<Submesh>,
}

#[derive(Debug, Clone)]
pub struct Submesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,
	pub attributes: Vec<String>,
	pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Default)]
pub struct Vertex {
	pub position: glam::Vec3,
	pub normal: glam::Vec3,
	pub tangent: glam::Vec4,
	pub uv: glam::Vec4,
	pub color: glam::Vec4,
	pub blends: [Blend; 8],
}

#[derive(Debug, Clone, Default)]
pub struct Blend {
	pub bone: u8,
	pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct Shape {
	pub name: String,
	pub values: Vec<ShapeValue>,
}

#[derive(Debug, Clone)]
pub struct ShapeValue {
	pub index: u16,
	pub new_vertex: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Mdl {
	pub lods: Vec<Lod>,
	pub bones: Vec<String>,
}

impl Mdl {
	pub fn absolute_mtrl_path(material_path: &str, variant: usize) -> String {
		if !material_path.starts_with("/") {
			return material_path.to_string();
		}
		
		let type1 = &material_path[4..=4];
		let id1 = &material_path[5..9];
		let type2 = &material_path[9..=9];
		let id2 = &material_path[10..14];
		
		match (type1, type2) {
			("c", "a") => format!("chara/accessory/a{id2}/material/v{variant:04}{material_path}"),
			("c", "e") => format!("chara/equipment/e{id2}/material/v{variant:04}{material_path}"),
			("c", "b") => format!("chara/human/c{id1}/obj/body/b{id2}/material/v{variant:04}{material_path}"),
			("c", "f") => format!("chara/human/c{id1}/obj/face/f{id2}/material{material_path}"),
			("c", "h") => format!("chara/human/c{id1}/obj/hair/h{id2}/material/v{variant:04}{material_path}"),
			("c", "t") => format!("chara/human/c{id1}/obj/tail/t{id2}/material/v{variant:04}{material_path}"),
			("c", "z") => format!("chara/human/c{id1}/obj/zear/z{id2}/material{material_path}"),
			("d", "e") => format!("chara/demihuman/d{id1}/obj/equipment/e{id2}/material/v{variant:04}{material_path}"),
			("m", "b") => format!("chara/monster/m{id1}/obj/body/b{id2}/material/v{variant:04}{material_path}"),
			("w", "b") => format!("chara/weapon/w{id1}/obj/body/b{id2}/material/v{variant:04}{material_path}"),
			_ => material_path.to_string()
		}
	}
	
	pub fn skeleton_paths(model_path: &str) -> Vec<String> {
		let path = &model_path[model_path.rfind('/').unwrap_or(0)..];
		
		let type1 = &path[1..=1];
		let id1 = &path[2..6];
		let type2 = &path[6..=6];
		let id2 = &path[7..11];
		
		match (type1, type2) {
			("c", "a") => vec![format!("chara/human/c{id1}/skeleton/base/b0001/skl_c{id1}b0001.sklb")],
			("c", "e") => vec![format!("chara/human/c{id1}/skeleton/base/b0001/skl_c{id1}b0001.sklb")],
			("c", "b") => vec![format!("chara/human/c{id1}/skeleton/base/b0001/skl_c{id1}b0001.sklb")],
			("c", "f") => vec![format!("chara/human/c{id1}/skeleton/face/f{0:04}/skl_c{id1}f{0:04}.sklb", id2.parse::<u32>().unwrap() + 1),
			                   format!("chara/human/c{id1}/skeleton/base/b0001/skl_c{id1}b0001.sklb")],
			("c", "h") => vec![format!("chara/human/c{id1}/skeleton/hair/h{id2}/skl_c{id1}h{id2}.sklb")],
			("c", "t") => vec![format!("chara/human/c{id1}/skeleton/base/b0001/skl_c{id1}b0001.sklb")],
			("c", "z") => vec![format!("chara/human/c{id1}/skeleton/base/b0001/skl_c{id1}b0001.sklb")],
			("d", "e") => vec![format!("chara/demihuman/d{id1}/skeleton/base/b{id2}/skl_d{id1}b{id2}.sklb")],
			("m", "b") => vec![format!("chara/monster/m{id1}/skeleton/base/b{id2}/skl_m{id1}b{id2}.sklb")],
			("w", "b") => vec![format!("chara/weapon/w{id1}/skeleton/base/b{id2}/skl_w{id1}b{id2}.sklb")],
			_ => Vec::new(),
		}
	}
	
	pub fn bake_materials(&self, file_reader: impl Fn(&str) -> Option<Vec<u8>>) -> HashMap<String, MaterialBake> {
		fn read_mtrl(file_reader: &impl Fn(&str) -> Option<Vec<u8>>, path: &str) -> Option<super::Mtrl> {
			let data = file_reader(path)?;
			<super::Mtrl as crate::format::external::Bytes<super::mtrl::Error>>::read(&mut Cursor::new(data)).ok()
		}
		
		fn read_tex(file_reader: &impl Fn(&str) -> Option<Vec<u8>>, path: &str) -> Option<super::Tex> {
			let data = file_reader(path)?;
			<super::Tex as crate::format::external::Bytes<super::tex::Error>>::read(&mut Cursor::new(data)).ok()
		}
		
		let mut textures = HashMap::new();
		for lod in &self.lods {
			for mesh in &lod.meshes {
				let mtrl_path = Mdl::absolute_mtrl_path(&mesh.material, 1);
				if textures.contains_key(&mtrl_path) {continue}
				let Some(mtrl) = read_mtrl(&file_reader, &mtrl_path) else {continue};
				
				// Some meshes have both Diffuse and ColorsetIndex
				// for the one i tested it doesnt seem like it is actually used ingame
				// chara/monster/m0934/obj/body/b0001/model/m0934b0001.mdl
				let mut diffuse_is_diffuse = false;
				let mut diffuse = None;
				let mut normal = None;
				
				for sampler in &mtrl.samplers {
					match super::mtrl::shader_param_name(sampler.id).as_deref() {
						Some("g_SamplerDiffuse") |
						Some("g_SamplerColorMap0") => {
							let Some(tex) = read_tex(&file_reader, &sampler.texture) else {continue};
							diffuse = Some(MaterialBakeTexture{width: tex.width, height: tex.height, data: tex.pixels[..(tex.width * tex.height * 4) as usize].to_vec()});
							diffuse_is_diffuse = true;
						}
						
						Some("g_SamplerIndex") => {
							let Some(tex) = read_tex(&file_reader, &sampler.texture) else {continue};
							let pixels = tex.pixels[..(tex.width * tex.height * 4) as usize]
								.chunks_exact(4)
								.flat_map(|v| {
									let [r, g, _b, a] = v else {unreachable!()};
									let id = (*r as f32 / 17.0).round() as usize;
									let point = *g as f32 / 255.0;
									let row1 = &mtrl.colorsets[0].regular[id * 2];
									let row2 = &mtrl.colorsets[0].regular[id * 2 + 1];
									let clr = row1.diffuse * point + row2.diffuse * (1.0 - point);
									
									[
										(clr.x * 255.0) as u8,
										(clr.y * 255.0) as u8,
										(clr.z * 255.0) as u8,
										*a,
									]
								}).collect::<Vec<u8>>();
							
							if !diffuse_is_diffuse {
								diffuse = Some(MaterialBakeTexture{width: tex.width, height: tex.height, data: pixels});
							}
						}
						
						Some("g_SamplerNormal") |
						Some("g_SamplerNormalMap0") => {
							let Some(tex) = read_tex(&file_reader, &sampler.texture) else {continue};
							normal = Some(MaterialBakeTexture{width: tex.width, height: tex.height, data: tex.pixels[..(tex.width * tex.height * 4) as usize].to_vec()});
						}
						
						_ => {}
					}
				}
				
				textures.insert(mesh.material.clone(), (mtrl.shader.to_ascii_lowercase(), diffuse, normal));
			}
		}
		
		// do what shaders do to get accurate visuals
		// https://docs.google.com/spreadsheets/d/1kIKvVsW3fOnVeTi9iZlBDqJo6GWVn6K6BCUIRldEjhw/edit?gid=1406279597#gid=1406279597
		textures.into_iter()
			.map(|(k, (shader, mut diffuse, mut normal))| {
				// This doesnt actually seem to work, might be because of the render pipeline (probably is), cba for now
				// TODO: make this work
				match (shader.as_str(), &mut diffuse, &mut normal) {
					("character.shpk", Some(diffuse), Some(normal)) |
					("characterlegacy.shpk", Some(diffuse), Some(normal)) => {
						diffuse.data.chunks_exact_mut(4)
							.zip(normal.data.chunks_exact(4))
							.for_each(|(dv, nv)| dv[3] = nv[2])
					}
					
					("bgcolorchange.shpk", Some(diffuse), Some(normal)) => {
						diffuse.data.chunks_exact_mut(4)
							.zip(normal.data.chunks_exact(4))
							.for_each(|(dv, nv)| dv[3] = nv[3])
					}
					
					_ => {}
				}
				
				(
					k,
					MaterialBake {
						diffuse,
						normal,
					}
				)
			}).collect::<HashMap<_, _>>()
	}
}

impl BinRead for Mdl {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		simple_reader!(reader, endian);
		
		let header = r!(HeaderRaw);
		let vertex_declerations = r!(Vec<[VertexElementRaw; 17]>, header.vertex_declaration_count);
		
		let strings_count = r!(u16);
		r!(move 2);
		let strings_size = r!(u32);
		let strings_buf = r!(Vec<u8>, strings_size);
		
		let model_header = r!(ModelHeaderRaw);
		let element_ids = r!(Vec<ElementIdRaw>, model_header.element_id_count);
		// textools writes 1 lod count but writes 3 lod structs, with the other 2 empty
		// i am not putting SE above doing that themselves, but im blaming textools anyways
		// let lods = r!(Vec<LodRaw>, header.lod_count);
		// let extra_lods = r!(Vec<ExtraLodRaw>, if model_header.flags2.contains(ModelFlags2Raw::EXTRA_LOD_ENABLED) {header.lod_count} else {0});
		let lods = r!(Vec<LodRaw>, 3);
		let extra_lods = r!(Vec<ExtraLodRaw>, if model_header.flags2.contains(ModelFlags2Raw::EXTRA_LOD_ENABLED) {3} else {0});
		let meshes = r!(Vec<MeshRaw>, model_header.mesh_count);
		let attribute_string_offset = r!(Vec<u32>, model_header.attribute_count);
		let terrain_shadow_meshes = r!(Vec<TerrainShadowMeshRaw>, model_header.terrain_shadow_mesh_count);
		let submeshes = r!(Vec<SubmeshRaw>, model_header.submesh_count);
		let terrain_shadow_submeshes = r!(Vec<TerrainShadowSubmeshRaw>, model_header.terrain_shadow_submesh_count);
		let material_string_offset = r!(Vec<u32>, model_header.material_count);
		let bone_string_offset = r!(Vec<u32>, model_header.bone_count);
		let bone_table = r!(bone_table_reader, (header.version, model_header.bone_table_count, model_header.bone_table_array_count_total));
		let shapes = r!(Vec<ShapeRaw>, model_header.shape_count);
		let shape_meshes = r!(Vec<ShapeMeshRaw>, model_header.shape_mesh_count);
		let shape_values = r!(Vec<ShapeValueRaw>, model_header.shape_value_count);
		let submesh_bone_map_size = r!(u32);
		let submesh_bone_map = r!(Vec<u16>, submesh_bone_map_size / 2);
		
		let _padding_size = r!(u8);
		r!(move _padding_size);
		
		let bb = r!(BoundingBoxRaw);
		let model_bb = r!(BoundingBoxRaw);
		let water_bb = r!(BoundingBoxRaw);
		let vertical_fog_bb = r!(BoundingBoxRaw);
		let bones_bb = r!(Vec<BoundingBoxRaw>, model_header.bone_count);
		
		// create meshes
		let mut lods_new = Vec::new();
		for (lod_index, lod_raw) in lods.iter().enumerate() {
			let mut meshes_new = Vec::new();
			for mesh_index in lod_raw.mesh_index as usize..(lod_raw.mesh_index + lod_raw.mesh_count) as usize {
				let mesh_raw = &meshes[mesh_index];
				
				// shapes
				let mut shapes_tmp = Vec::new();
				for shape_raw in &shapes {
					let name = strings_buf[shape_raw.string_offset as usize..].null_terminated().unwrap();
					let mut replacements = Vec::new();
					
					for shape_mesh_index in shape_raw.mesh_start_index[lod_index]..shape_raw.mesh_start_index[lod_index] + shape_raw.mesh_count[lod_index] {
						let shape_mesh_raw = &shape_meshes[shape_mesh_index as usize];
						if shape_mesh_raw.mesh_index_offset != mesh_raw.start_index {continue};
						for i in shape_mesh_raw.value_offset..shape_mesh_raw.value_offset + shape_mesh_raw.value_count {
							let val = &shape_values[i as usize];
							replacements.push((val.base_indices_index, val.replacing_vertex_index));
						}
					}
					
					if replacements.len() > 0 {
						shapes_tmp.push((name, replacements));
					}
				}
				
				// vertices
				let mut vertices = vec![Vertex::default(); mesh_raw.vertex_count as usize];
				let vertex_decl = &vertex_declerations[mesh_index];
				// textools writes vertex_stream_count as the amount of meshes, and not the actual purpose :mochiwohoo:
				// for stream in 0..mesh_raw.vertex_stream_count {
				for stream in 0..3u8 {
					r!(seek header.vertex_offsets[lod_index] as u64 + mesh_raw.vertex_buffer_offset[stream as usize] as u64);
					
					for vertex_index in 0..mesh_raw.vertex_count as usize {
						let vertex = &mut vertices[vertex_index];
						for decl in vertex_decl {
							if decl.stream == 255 {break}
							if decl.stream != stream {continue}
							
							match (decl.usage, decl.typ) {
								(VertexUsageRaw::BlendWeights, VertexTypeRaw::U16x4) =>
									for i in 0..8 {vertex.blends[i].weight = r!(u8) as f32 / 255.0},
								
								(VertexUsageRaw::BlendWeights, VertexTypeRaw::U8x4) =>
									for i in 0..4 {vertex.blends[i].weight = r!(u8) as f32 / 255.0},
								
								(VertexUsageRaw::BlendIndices, VertexTypeRaw::U16x4) =>
									for i in 0..8 {vertex.blends[i].bone = bone_table[mesh_raw.bone_table_index as usize][r!(u8) as usize] as u8},
								
								(VertexUsageRaw::BlendIndices, VertexTypeRaw::U8x4) =>
									for i in 0..4 {vertex.blends[i].bone = bone_table[mesh_raw.bone_table_index as usize][r!(u8) as usize] as u8},
								
								_ => {
									let val = match decl.typ {
										VertexTypeRaw::F32x1 => glam::vec4(r!(f32), 0.0, 0.0, 0.0),
										VertexTypeRaw::F32x2 => glam::vec4(r!(f32), r!(f32), 0.0, 0.0),
										VertexTypeRaw::F32x3 => glam::vec4(r!(f32), r!(f32), r!(f32), 0.0),
										VertexTypeRaw::F32x4 => glam::vec4(r!(f32), r!(f32), r!(f32), r!(f32)),
										VertexTypeRaw::U8x4  => glam::vec4(r!(u8) as f32, r!(u8) as f32, r!(u8) as f32, r!(u8) as f32),
										VertexTypeRaw::F8x4  => glam::vec4(r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0),
										VertexTypeRaw::F16x2 => glam::vec4(r!(f16), r!(f16), 0.0, 0.0),
										VertexTypeRaw::F16x4 => glam::vec4(r!(f16), r!(f16), r!(f16), r!(f16)),
										VertexTypeRaw::U16x2 => glam::vec4(r!(u16) as f32, r!(u16) as f32, 0.0, 0.0),
										VertexTypeRaw::U16x4 => glam::vec4(r!(u16) as f32, r!(u16) as f32, r!(u16) as f32, r!(u16) as f32),
									};
									
									match decl.usage {
										VertexUsageRaw::Position =>
											vertex.position = val.xyz(),
										
										VertexUsageRaw::BlendWeights =>
											for i in 0..4 {vertex.blends[i].weight = val[i] / 255.0},
										
										VertexUsageRaw::BlendIndices =>
											for i in 0..4 {vertex.blends[i].bone = bone_table[mesh_raw.bone_table_index as usize][val[i] as usize] as u8},
										
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
					}
				}
				
				// split into submeshes
				let mut submeshes_new = Vec::new();
				for submesh_index in mesh_raw.submesh_index as usize..(mesh_raw.submesh_index + mesh_raw.submesh_count) as usize {
					let submesh_raw = &submeshes[submesh_index];
					
					r!(seek header.index_offsets[lod_index] as u64 + submesh_raw.index_offset as u64 * 2);
					let indices = r!(Vec<u16>, submesh_raw.index_count);
					// let bone_map = &submesh_bone_map[submesh_raw.bone_start_index as usize..(submesh_raw.bone_start_index + submesh_raw.bone_count) as usize];
					
					// rewrite indices and vertices to be submesh specific
					let mut indices_new = Vec::new();
					let mut vertices_new = Vec::new();
					let mut vertex_map = HashMap::new(); // old_index, new_vertex
					let mut add_vertex = |index_old: u16| -> u16 {
						*vertex_map.entry(index_old).or_insert_with(|| {
							let v = vertices[index_old as usize].clone();
							// for i in 0..8 {
							// 	v.blends[i].bone = bone_map[v.blends[i].bone as usize] as u8;
							// }
							
							vertices_new.push(v);
							(vertices_new.len() - 1) as u16
						})
					};
					
					for index_old in &indices {
						indices_new.push(add_vertex(*index_old));
					}
					
					// rewrite shapes to be submesh specific
					let index_relative_offset = (submesh_raw.index_offset - mesh_raw.start_index) as u16;
					let mut shapes_new = Vec::new();
					for (name, values_old) in &shapes_tmp {
						let mut values = Vec::new();
						for (index_old, vertex_old) in values_old {
							if *index_old < index_relative_offset || *index_old >= index_relative_offset + submesh_raw.index_count as u16 {continue}
							
							values.push(ShapeValue {
								index: *index_old - index_relative_offset,
								new_vertex: add_vertex(*vertex_old),
							});
						}
						
						if values.len() > 0 {
							shapes_new.push(Shape {
								name: name.to_owned(),
								values,
							});
						}
					}
					
					// attributes
					let mut attributes_new = Vec::new();
					for i in 0..attribute_string_offset.len() {
						let v = 1u32 << i;
						if submesh_raw.attribute_index_mask & v == v {
							attributes_new.push(strings_buf[attribute_string_offset[i] as usize..].null_terminated().unwrap());
						}
					}
					
					submeshes_new.push(Submesh {
						vertices: vertices_new,
						indices: indices_new,
						attributes: attributes_new,
						shapes: shapes_new,
					});
				}
				
				meshes_new.push(Mesh {
					material: strings_buf[material_string_offset[mesh_raw.material_index as usize] as usize..].null_terminated().unwrap(),
					submeshes: submeshes_new,
				});
			}
			
			lods_new.push(Lod {
				meshes: meshes_new,
			});
		}
		
		Ok(Self {
			lods: lods_new,
			bones: bone_string_offset
				.into_iter()
				.map(|v| strings_buf[v as usize..].null_terminated().unwrap())
				.collect(),
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

// ----------

impl crate::format::external::Gltf<Error> for Mdl {
	fn read<T>(reader: &mut T) -> Result<Self, Error> where
	T: Read + Seek {
		todo!()
	}
	
	fn write<T>(&self, writer: &mut T, materials: HashMap<String, MaterialBake>, bones: Vec<crate::format::external::gltf::Bone>) -> Result<(), Error> where
	T: Write + Seek {
		use gltf::json::{self, validation::Checked::Valid};
		
		// TODO: set this to the mode being exported
		let model_name = "Model";
		
		let mut root = json::Root::default();
		root.asset.generator = Some("Aetherment Mdl glTF Exporter v0.1.0".to_string());
		
		let buffer = root.push(json::Buffer {
			name: None,
			byte_length: json::validation::USize64(0), // we set the size afterwards
			uri: None,
			extensions: Default::default(),
			extras: Default::default(),
		});
		
		let mut buf = Vec::new();
		fn write(root: &mut json::Root, buffer: json::Index<json::Buffer>, buf: &mut Vec<u8>, data: &[u8], stride: usize, is_index: bool) -> json::Index<json::buffer::View> {
			let offset = buf.len();
			let data = to_padded_byte_vector(data);
			let size = data.len();
			buf.extend_from_slice(&data);
			root.push(json::buffer::View {
				name: None,
				buffer,
				byte_offset: Some(json::validation::USize64(offset as u64)),
				byte_length: json::validation::USize64(size as u64),
				byte_stride: if is_index || stride == 0 {None} else {Some(json::buffer::Stride(stride))},
				target: Some(json::validation::Checked::Valid(if is_index {json::buffer::Target::ElementArrayBuffer} else {json::buffer::Target::ArrayBuffer})),
				extensions: Default::default(),
				extras: Default::default(),
			})
		}
		
		fn accessor(root: &mut json::Root, buffer_view: json::Index<json::buffer::View>, offset: usize, count: usize, typ: json::accessor::Type, typ_inner: gltf::accessor::DataType, bb: Option<[[f32; 3]; 2]>, name: &str) -> json::Index<json::Accessor> {
			root.push(json::Accessor {
				name: Some(name.to_string()),
				buffer_view: Some(buffer_view),
				byte_offset: Some(json::validation::USize64(offset as u64)),
				count: json::validation::USize64(count as u64),
				type_: Valid(typ),
				component_type: Valid(json::accessor::GenericComponentType(typ_inner)),
				min: bb.map(|v| json::Value::from(v[0].to_vec())),
				max: bb.map(|v| json::Value::from(v[1].to_vec())),
				sparse: None,
				normalized: false,
				extensions: Default::default(),
				extras: Default::default(),
			})
		}
		
		let mut bone_nodes = Vec::new();
		let mut bone_ids = HashMap::new();
		let mut bone_map = HashMap::<String, json::Index<json::Node>>::new();
		let mut bone_matrixes = Vec::new();
		let skin = if bones.len() > 0 {
			for (i, bone) in bones.into_iter().enumerate() {
				let node = root.push(json::Node {
					name: Some(bone.name.clone()),
					translation: Some(bone.translation.into()),
					rotation: Some(json::scene::UnitQuaternion(bone.rotation.into())),
					scale: Some(bone.scale.into()),
					..Default::default()
				});
				
				let mut matrix = glam::Mat4::from_rotation_translation(bone.rotation, bone.translation);
				
				if let Some(parent) = bone.parent {
					if let Some(parent_node_index) = bone_map.get(&parent) {
						let parent_node = &mut root.nodes[parent_node_index.value()];
						parent_node.children.get_or_insert_with(|| Vec::new()).push(node);
						matrix = bone_matrixes[bone_ids[&parent]] * matrix;
					}
				}
				
				bone_nodes.push(node);
				bone_map.insert(bone.name.clone(), node);
				bone_ids.insert(bone.name, i);
				bone_matrixes.push(matrix);
			}
			
			for m in &mut bone_matrixes {
				*m = m.inverse();
			}
			
			let matrix_view = write(&mut root, buffer, &mut buf, bytemuck::cast_slice(&bone_matrixes), 0, false);
			let matrix_access = accessor(&mut root, matrix_view, 0, bone_matrixes.len(), json::accessor::Type::Mat4, gltf::accessor::DataType::F32, None, "inversebindmatrices");
			
			let skin = root.push(json::Skin {
				name: Some(model_name.to_string()),
				inverse_bind_matrices: Some(matrix_access),
				joints: bone_nodes.clone(),
				skeleton: None,
				extensions: Default::default(),
				extras: Default::default(),
			});
			
			Some(skin)
		} else {
			None
		};
		
		let materials = materials
			.into_iter()
			.map(|(path, material)| {
				let name = path.split("/").last().unwrap().to_string();
				
				let mut create_texture = |name: String, texture: MaterialBakeTexture| {
					use image::ImageEncoder;
					
					let mut imgdata = Vec::new();
					let img = image::codecs::png::PngEncoder::new(Cursor::new(&mut imgdata));
					img.write_image(&texture.data, texture.width, texture.height, image::ColorType::Rgba8.into()).unwrap();
					
					let image_view = write(&mut root, buffer, &mut buf, &imgdata, 0, false);
					
					let image = root.push(json::Image {
						name: Some(name.clone()),
						buffer_view: Some(image_view),
						mime_type: Some(json::image::MimeType("image/png".to_string())),
						uri: None,
						extensions: Default::default(),
						extras: Default::default(),
					});
					
					root.push(json::Texture {
						name: Some(name),
						sampler: None,
						source: image,
						extensions: Default::default(),
						extras: Default::default(),
					})
				};
				
				let diffuse = material.diffuse.map(|v| {
					let texture = create_texture(format!("Diffuse_{name}"), v);
					
					json::texture::Info {
						index: texture,
						tex_coord: 0,
						extensions: Default::default(),
						extras: Default::default(),
					}
				});
				
				let normal = material.normal.map(|v| {
					let texture = create_texture(format!("Normal_{name}"), v);
					
					json::material::NormalTexture {
						index: texture,
						tex_coord: 0,
						scale: 1.0,
						extensions: Default::default(),
						extras: Default::default(),
					}
				});
				
				let material = root.push(json::Material {
					name: Some(name),
					normal_texture: normal,
					pbr_metallic_roughness: json::material::PbrMetallicRoughness {
						base_color_texture: diffuse,
						metallic_factor: json::material::StrengthFactor(0.0),
						roughness_factor: json::material::StrengthFactor(0.75),
						..Default::default()
					},
					..Default::default()
				});
				
				(path, material)
			}).collect::<HashMap<_, _>>();
		
		let mut nodes = Vec::new();
		for (lod_index, lod) in self.lods.iter().enumerate() {
			for (mesh_index, mesh) in lod.meshes.iter().enumerate() {
				for (submesh_index, submesh) in mesh.submeshes.iter().enumerate() {
					// TODO: dont add bones if theres no skeleton, and only use 4 if 8 isnt needed (probably anything except faces)
					#[repr(C)]
					#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
					struct Vertex {
						uv1: glam::Vec2,
						uv2: glam::Vec2,
						color_dummy: glam::Vec4, // so that the software doesnt actually render it (blender does the same when you change the material to not use vertex color)
						color1: glam::Vec4,
						normal: glam::Vec3,
						position: glam::Vec3,
						bones: [u8; 8],
						weights: [f32; 8],
					}
					
					let vertices = submesh.vertices
						.iter()
						.map(|v| Vertex {
							uv1: v.uv.xy(),
							uv2: v.uv.zw(),
							color_dummy: glam::Vec4::ONE,
							color1: v.color,
							position: v.position,
							normal: v.normal,
							bones: if self.bones.len() > 0 {
								v.blends.iter().map(|v| bone_ids[&self.bones[v.bone as usize]] as u8).collect::<Vec<_>>().try_into().unwrap()
							} else {[0u8; 8]},
							weights: v.blends.iter().map(|v| v.weight).collect::<Vec<_>>().try_into().unwrap(),
						}).collect::<Vec<_>>();
					
					let mut min = [f32::MAX; 3];
					let mut max = [f32::MIN; 3];
					for v in &vertices {
						for i in 0..3 {
							min[i] = min[i].min(v.position[i]);
							max[i] = max[i].max(v.position[i]);
						}
					}
					
					// TODO: atm it includes vertices for the shapekeys aswell, aka it balloons in side
					// got 1 shape key affecting the entire mesh? enjoy 4x the size instead of 2x
					let vertices_view = write(&mut root, buffer, &mut buf, bytemuck::cast_slice(&vertices), size_of::<Vertex>(), false);
					let uv1_access = accessor(&mut root, vertices_view, 0x0, vertices.len(), json::accessor::Type::Vec2, gltf::accessor::DataType::F32, None, "uv1");
					let uv2_access = accessor(&mut root, vertices_view, 0x8, vertices.len(), json::accessor::Type::Vec2, gltf::accessor::DataType::F32, None, "uv2");
					let color0_access = accessor(&mut root, vertices_view, 0x10, vertices.len(), json::accessor::Type::Vec4, gltf::accessor::DataType::F32, None, "color0");
					let color1_access = accessor(&mut root, vertices_view, 0x20, vertices.len(), json::accessor::Type::Vec4, gltf::accessor::DataType::F32, None, "color1");
					let normal_access = accessor(&mut root, vertices_view, 0x30, vertices.len(), json::accessor::Type::Vec3, gltf::accessor::DataType::F32, None, "normal");
					let position_access = accessor(&mut root, vertices_view, 0x3C, vertices.len(), json::accessor::Type::Vec3, gltf::accessor::DataType::F32, Some([min, max]), "position");
					let bones1_access = accessor(&mut root, vertices_view, 0x48, vertices.len(), json::accessor::Type::Vec4, gltf::accessor::DataType::U8, None, "bones1");
					let bones2_access = accessor(&mut root, vertices_view, 0x4C, vertices.len(), json::accessor::Type::Vec4, gltf::accessor::DataType::U8, None, "bones2");
					let weights1_access = accessor(&mut root, vertices_view, 0x50, vertices.len(), json::accessor::Type::Vec4, gltf::accessor::DataType::F32, None, "weights1");
					let weights2_access = accessor(&mut root, vertices_view, 0x60, vertices.len(), json::accessor::Type::Vec4, gltf::accessor::DataType::F32, None, "weights2");
					
					let indices_view = write(&mut root, buffer, &mut buf, bytemuck::cast_slice(&submesh.indices), 0, true);
					let indices_access = accessor(&mut root, indices_view, 0, submesh.indices.len(), json::accessor::Type::Scalar, gltf::accessor::DataType::U16, None, "indices");
					
					let shapekeys = submesh.shapes
						.iter()
						.map(|shape| {
							#[repr(C)]
							#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
							struct ShapeValue {
								normal: glam::Vec3,
								position: glam::Vec3,
							}
							
							let mut values = vec![ShapeValue {
								normal: glam::Vec3::ZERO,
								position: glam::Vec3::ZERO,
							}; submesh.vertices.len()];
							
							for value in &shape.values {
								let og_vertex = submesh.indices[value.index as usize] as usize;
								let val = &mut values[og_vertex];
								let og_vert = &submesh.vertices[og_vertex];
								let shape_vert = &submesh.vertices[value.new_vertex as usize];
								val.normal = shape_vert.normal - og_vert.normal;
								val.position = shape_vert.position - og_vert.position;
							}
							
							let mut min = [f32::MAX; 3];
							let mut max = [f32::MIN; 3];
							for v in &vertices {
								for i in 0..3 {
									min[i] = min[i].min(v.position[i]);
									max[i] = max[i].max(v.position[i]);
								}
							}
							
							let values_view = write(&mut root, buffer, &mut buf, bytemuck::cast_slice(&values), size_of::<ShapeValue>(), false);
							let normal_access = accessor(&mut root, values_view, 0x0, values.len(), json::accessor::Type::Vec3, gltf::accessor::DataType::F32, None, &shape.name);
							let position_access = accessor(&mut root, values_view, 0xC, values.len(), json::accessor::Type::Vec3, gltf::accessor::DataType::F32, Some([min, max]), &shape.name);
							
							json::mesh::MorphTarget {
								positions: Some(position_access),
								normals: Some(normal_access),
								tangents: None,
							}
						}).collect::<Vec<_>>();
					
					let primitive = json::mesh::Primitive {
						attributes: std::collections::BTreeMap::from_iter(vec![
							(Valid(json::mesh::Semantic::TexCoords(0)), uv1_access),
							(Valid(json::mesh::Semantic::TexCoords(1)), uv2_access),
							(Valid(json::mesh::Semantic::Colors(0)), color0_access),
							(Valid(json::mesh::Semantic::Colors(1)), color1_access),
							(Valid(json::mesh::Semantic::Normals), normal_access),
							(Valid(json::mesh::Semantic::Positions), position_access),
							(Valid(json::mesh::Semantic::Joints(0)), bones1_access),
							(Valid(json::mesh::Semantic::Joints(1)), bones2_access),
							(Valid(json::mesh::Semantic::Weights(0)), weights1_access),
							(Valid(json::mesh::Semantic::Weights(1)), weights2_access),
						]),
						indices: Some(indices_access),
						material: Some(materials[&mesh.material]),
						targets: if shapekeys.len() > 0 {Some(shapekeys)} else {None},
						mode: Valid(gltf::mesh::Mode::Triangles),
						extensions: Default::default(),
						extras: Default::default(),
					};
					
					let mesh = root.push(json::Mesh {
						name: Some(format!("Mesh Lod{lod_index} Mesh{mesh_index} Submesh{submesh_index}")),
						primitives: vec![primitive],
						weights: None,
						extensions: Default::default(),
						extras: Default::default(),
					});
					
					let node = root.push(json::Node {
						name: Some(format!("Lod{lod_index} Mesh{mesh_index} Submesh{submesh_index}")),
						mesh: Some(mesh),
						skin,
						..Default::default()
					});
					
					nodes.push(node);
				}
			}
		}
		
		root.buffers[0].byte_length.0 = buf.len() as u64;
		
		let mut children = if bone_nodes.len() > 0 {vec![bone_nodes[0]]} else {Vec::new()};
		children.extend_from_slice(&nodes);
		
		let parent = root.push(json::Node {
			name: Some(model_name.to_string()),
			children: Some(children),
			..Default::default()
		});
		
		root.push(json::Scene {
			name: None,
			nodes: vec![parent],
			extensions: Default::default(),
			extras: Default::default(),
		});
		
		let json_string = json::serialize::to_string(&root).unwrap();
		let mut json_offset = json_string.len();
		align_to_multiple_of_four(&mut json_offset);
		let glb = gltf::binary::Glb {
			header: gltf::binary::Header {
				magic: *b"glTF",
				version: 2,
				length: (json_offset + buf.len()) as u32,
			},
			bin: Some(std::borrow::Cow::Owned(to_padded_byte_vector(&buf))),
			json: std::borrow::Cow::Owned(json_string.into_bytes()),
		};
		
		glb.to_writer(writer).unwrap();
		
		Ok(())
	}
}

// https://github.com/gltf-rs/gltf/blob/main/examples/export/main.rs#L41
fn align_to_multiple_of_four(n: &mut usize) {
	*n = (*n + 3) & !3;
}

fn to_padded_byte_vector<T: bytemuck::NoUninit>(data: &[T]) -> Vec<u8> {
	let byte_slice: &[u8] = bytemuck::cast_slice(data);
	let mut new_vec: Vec<u8> = byte_slice.to_owned();
	
	while new_vec.len() % 4 != 0 {
		new_vec.push(0); // pad to multiple of four bytes
	}
	
	new_vec
}

// ----------

#[binrw]
#[derive(Debug, Clone)]
struct HeaderRaw {
	version: u32,
	stack_size: u32,
	runtime_size: u32,
	vertex_declaration_count: u16,
	material_count: u16,
	vertex_offsets: [u32; 3],
	index_offsets: [u32; 3],
	vertex_buffer_offsets: [u32; 3],
	index_buffer_offsets: [u32; 3],
	lod_count: u8,
	index_buffer_streaming: u8,
	edge_geometry: u8,
	_padding: u8,
}

#[binrw]
#[derive(Debug, Clone)]
struct VertexElementRaw {
	stream: u8,
	offset: u8,
	typ: VertexTypeRaw,
	usage: VertexUsageRaw,
	usage_index: u8,
	_padding: [u8; 3],
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum VertexTypeRaw {
	F32x1 = 0,
	F32x2 = 1,
	F32x3 = 2,
	F32x4 = 3,
	U8x4  = 5,
	F8x4  = 8,
	F16x2 = 13,
	F16x4 = 14,
	U16x2 = 16,
	U16x4 = 17,
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum VertexUsageRaw {
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
struct ModelHeaderRaw {
	radius: f32,
	mesh_count: u16,
	attribute_count: u16,
	submesh_count: u16,
	material_count: u16,
	bone_count: u16,
	bone_table_count: u16,
	shape_count: u16,
	shape_mesh_count: u16,
	shape_value_count: u16,
	lod_count: u8,
	#[br(map = |v: u8| ModelFlags1Raw::from_bits_retain(v))]
	#[bw(map = |v: &ModelFlags1Raw| v.bits())]
	flags1: ModelFlags1Raw,
	element_id_count: u16,
	terrain_shadow_mesh_count: u8,
	#[br(map = |v: u8| ModelFlags2Raw::from_bits_retain(v))]
	#[bw(map = |v: &ModelFlags2Raw| v.bits())]
	flags2: ModelFlags2Raw,
	model_clip_out_distance: f32,
	shadow_clip_out_distance: f32,
	culling_grid_count: u16,
	terrain_shadow_submesh_count: u16,
	flags3: u8, // ?
	bg_change_material_index: u8,
	bg_crest_change_material_index: u8,
	unknown6: u8,
	bone_table_array_count_total: u16,
	unknown8: u16,
	unknown9: u16,
	_padding: [u8; 6],
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	struct ModelFlags1Raw: u8 {
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
	struct ModelFlags2Raw: u8 {
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
struct ElementIdRaw {
	element_id: u32,
	parent_bone_name: u32,
	translation: [f32; 3],
	rotation: [f32; 3],
}

#[binrw]
#[derive(Debug, Clone)]
struct LodRaw {
	mesh_index: u16,
	mesh_count: u16,
	model_lod_range: f32,
	texture_load_range: f32,
	water_mesh_index: u16,
	water_mesh_count: u16,
	shadow_mesh_index: u16,
	shadow_mesh_count: u16,
	terrain_shadow_mesh_index: u16,
	terrain_shadow_mesh_count: u16,
	vertical_fog_mesh_index: u16,
	vertical_fog_mesh_count: u16,
	
	edge_geometry_size: u32,
	edge_geometry_data_offset: u32,
	polygon_count: u32,
	unknown1: u32,
	vertex_buffer_size: u32,
	index_buffer_size: u32,
	vertex_data_offset: u32,
	index_data_offset: u32,
}

#[binrw]
#[derive(Debug, Clone)]
struct ExtraLodRaw {
	lightshaft_mesh_index: u16,
	lightshaft_mesh_count: u16,
	glass_mesh_index: u16,
	glass_mesh_count: u16,
	material_change_mesh_index: u16,
	material_change_mesh_count: u16,
	crest_change_mesh_index: u16,
	crest_change_mesh_count: u16,
	unknown1: u16,
	unknown2: u16,
	unknown3: u16,
	unknown4: u16,
	unknown5: u16,
	unknown6: u16,
	unknown7: u16,
	unknown8: u16,
	unknown9: u16,
	unknown10: u16,
	unknown11: u16,
	unknown12: u16,
}

#[binrw]
#[derive(Debug, Clone)]
struct MeshRaw {
	vertex_count: u16,
	_padding: u16,
	index_count: u32,
	material_index: u16,
	submesh_index: u16,
	submesh_count: u16,
	bone_table_index: u16,
	start_index: u32,
	vertex_buffer_offset: [u32; 3],
	vertex_buffer_stride: [u8; 3],
	vertex_stream_count: u8,
}

#[binrw]
#[derive(Debug, Clone)]
struct TerrainShadowMeshRaw {
	index_count: u32,
	start_index: u32,
	vertex_buffer_offset: u32,
	vertex_count: u16,
	submesh_index: u16,
	submesh_count: u16,
	vertex_buffer_stride: u8,
	_padding: u8
}

#[binrw]
#[derive(Debug, Clone)]
struct SubmeshRaw {
	index_offset: u32,
	index_count: u32,
	attribute_index_mask: u32,
	bone_start_index: u16,
	bone_count: u16,
}

#[binrw]
#[derive(Debug, Clone)]
struct TerrainShadowSubmeshRaw {
	index_offset: u32,
	index_count: u32,
	unknown1: u16,
	unknown2: u16,
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
struct ShapeRaw {
	string_offset: u32,
	mesh_start_index: [u16; 3],
	mesh_count: [u16; 3],
}

#[binrw]
#[derive(Debug, Clone)]
struct ShapeMeshRaw {
	mesh_index_offset: u32,
	value_count: u32,
	value_offset: u32,
}

#[binrw]
#[derive(Debug, Clone)]
struct ShapeValueRaw {
	base_indices_index: u16,
	replacing_vertex_index: u16,
}

#[binrw]
#[derive(Debug, Clone)]
struct BoundingBoxRaw {
	min: [f32; 4],
	max: [f32; 4],
}