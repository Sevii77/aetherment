use std::{io::{Read, Seek, Write, SeekFrom}, fmt::Debug};
use binrw::{binrw, BinRead, BinWrite};
use crate::{Error, NullReader, NullWriter};

mod component;
pub use component::*;
mod timeline;
pub use timeline::*;
mod node;
pub use node::*;

pub const EXT: &'static [&'static str] = &["uld"];

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Uld {
	pub main_header: UldHeader,
	pub primary_header: AtkHeader,
	
	pub assets_header: ListHeader,
	pub assets: Vec<UldTexture>,
	
	pub parts_header: ListHeader,
	pub parts_lists: Vec<UldPartsList>,
	
	pub components_header: ListHeader,
	pub components: Vec<UldComponent>,
	
	pub timelines_header: ListHeader,
	pub timelines: Vec<UldTimeline>,
	
	pub second_header: AtkHeader,
	pub widget_header: ListHeader,
	pub widgets: Vec<WidgetData>,
}

// used to load from spack using ironworks
impl ironworks::file::File for Uld {
	fn read<'a>(data: impl Into<std::borrow::Cow<'a, [u8]>>) -> super::Result<Self> {
		Uld::read(&mut std::io::Cursor::new(&data.into())).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

impl Uld {
	pub fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		Ok(Uld::read_le(reader)?)
	}
	
	pub fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.write_le(writer)?;
		
		Ok(())
	}
}

impl BinRead for Uld {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let pos = reader.stream_position()?;
		let main_header = UldHeader::read_options(reader, endian, ())?;
		let primary_pos = reader.stream_position()?;
		let primary_header = AtkHeader::read_options(reader, endian, ())?;
		
		reader.seek(SeekFrom::Start(primary_pos + primary_header.asset_offset as u64))?;
		let assets_header = ListHeader::read_options(reader, endian, ())?;
		let mut assets = Vec::with_capacity(assets_header.element_count as usize);
		for _ in 0..assets_header.element_count {
			assets.push(UldTexture::read_options(reader, endian, (assets_header.version[3],))?);
		}
		
		reader.seek(SeekFrom::Start(primary_pos + primary_header.part_offset as u64))?;
		let parts_header = ListHeader::read_options(reader, endian, ())?;
		let mut parts_lists = Vec::with_capacity(parts_header.element_count as usize);
		for _ in 0..parts_header.element_count {
			parts_lists.push(UldPartsList::read_options(reader, endian, ())?);
		}
		
		reader.seek(SeekFrom::Start(primary_pos + primary_header.component_offset as u64))?;
		let components_header = ListHeader::read_options(reader, endian, ())?;
		let mut components = Vec::with_capacity(components_header.element_count as usize);
		for _ in 0..components_header.element_count {
			components.push(UldComponent::read_options(reader, endian, &components)?);
		}
		
		reader.seek(SeekFrom::Start(primary_pos + primary_header.timeline_offset as u64))?;
		let timelines_header = ListHeader::read_options(reader, endian, ())?;
		let mut timelines = Vec::with_capacity(timelines_header.element_count as usize);
		for _ in 0..timelines_header.element_count {
			timelines.push(UldTimeline::read_options(reader, endian, ())?);
		}
		
		reader.seek(SeekFrom::Start(pos + main_header.widget_offset as u64))?;
		let secondary_pos = reader.stream_position()?;
		let second_header = AtkHeader::read_options(reader, endian, ())?;
		
		reader.seek(SeekFrom::Start(secondary_pos + second_header.widget_offset as u64))?;
		let widget_header = ListHeader::read_options(reader, endian, ())?;
		let mut widgets = Vec::with_capacity(widget_header.element_count as usize);
		for _ in 0..widget_header.element_count {
			widgets.push(WidgetData::read_options(reader, endian, &components[..])?);
		}
		
		Ok(Self {
			main_header,
			primary_header,
			assets_header,
			assets,
			parts_header,
			parts_lists,
			components_header,
			components,
			timelines_header,
			timelines,
			second_header,
			widget_header,
			widgets,
		})
	}
}

impl BinWrite for Uld {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		let pos = writer.stream_position()?;
		self.main_header.write_options(writer, endian, ())?;
		let primary_pos = writer.stream_position()?;
		self.primary_header.write_options(writer, endian, ())?;
		let mut primary_header = self.primary_header.clone();
		
		primary_header.asset_offset = (writer.stream_position()? - primary_pos) as u32;
		let mut assets_header = self.assets_header.clone();
		assets_header.element_count = self.assets.len() as u32;
		assets_header.write_options(writer, endian, ())?;
		for asset in &self.assets {
			asset.write_options(writer, endian, (assets_header.version[3],))?;
		}
		
		primary_header.part_offset = (writer.stream_position()? - primary_pos) as u32;
		let mut parts_header = self.parts_header.clone();
		parts_header.element_count = self.parts_lists.len() as u32;
		parts_header.write_options(writer, endian, ())?;
		for part in &self.parts_lists {
			part.write_options(writer, endian, ())?;
		}
		
		primary_header.component_offset = (writer.stream_position()? - primary_pos) as u32;
		let mut components_header = self.components_header.clone();
		components_header.element_count = self.components.len() as u32;
		components_header.write_options(writer, endian, ())?;
		for component in &self.components {
			component.write_options(writer, endian, ())?;
		}
		
		primary_header.timeline_offset = (writer.stream_position()? - primary_pos) as u32;
		let mut timelines_header = self.timelines_header.clone();
		timelines_header.element_count = self.timelines.len() as u32;
		timelines_header.write_options(writer, endian, ())?;
		for timeline in &self.timelines {
			timeline.write_options(writer, endian, ())?;
		}
		
		let secondary_pos = writer.stream_position()?;
		primary_header.widget_offset = (secondary_pos - pos) as u32;
		self.second_header.write_options(writer, endian, ())?;
		let mut second_header = self.second_header.clone();
		
		second_header.widget_offset = (writer.stream_position()? - secondary_pos) as u32;
		let mut widget_header = self.widget_header.clone();
		widget_header.element_count = self.widgets.len() as u32;
		widget_header.write_options(writer, endian, ())?;
		for widget in &self.widgets {
			widget.write_options(writer, endian, ())?;
		}
		
		let pos = writer.stream_position()?;
		writer.seek(SeekFrom::Start(primary_pos))?;
		primary_header.write_options(writer, endian, ())?;
		writer.seek(SeekFrom::Start(secondary_pos))?;
		second_header.write_options(writer, endian, ())?;
		writer.seek(SeekFrom::Start(pos))?;
		
		Ok(())
	}
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct UldHeader {
	pub identifier: [u8; 4],
	pub version: [u8; 4],
	
	pub component_offset: u32,
	pub widget_offset: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct AtkHeader {
	pub identifier: [u8; 4],
	pub version: [u8; 4],
	
	pub asset_offset: u32,
	pub part_offset: u32,
	pub component_offset: u32,
	pub timeline_offset: u32,
	pub widget_offset: u32,
	pub rewrite_data_offset: u32,
	pub timeline_size: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct ListHeader {
	pub identifier: [u8; 4],
	pub version: [u8; 4],
	
	element_count: u32,
	pub unk1: i32,
}

// ---------------------------------------- //

#[binrw]
#[brw(little)]
#[brw(import(minor_version: u8))]
#[derive(Debug, Clone)]
pub struct UldTexture {
	pub id: u32,
	#[br(try_map = |v: [u8; 44]| v.null_terminated())]
	#[bw(try_map = |v: &String| v.null_terminated(44))]
	pub path: String,
	pub icon: u32,
	#[brw(if(minor_version >= 1))]
	pub unk1: Option<u32>,
}

// ---------------------------------------- //

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct UldPartsList {
	pub id: u32,
	#[br(temp)]
	#[bw(calc = parts.len() as u32)]
	part_count: u32,
	#[br(temp)]
	#[bw(calc = 0)]
	_offset: u32,
	#[br(count = part_count)]
	pub parts: Vec<UldPart>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct UldPart {
	pub texture_id: u32,
	pub u: u16,
	pub v: u16,
	pub w: u16,
	pub h: u16,
}

// ---------------------------------------- //

#[derive(Debug, Clone)]
pub struct UldComponent {
	pub id: u32,
	pub ignore_input: bool,
	pub drag_arrow: bool,
	pub drop_arrow: bool,
	// pub component_type: ComponentType,
	// _node_count: u32,
	// _size: u16,
	// _offset: u16,
	pub component: Component,
	pub nodes: Vec<NodeData>,
}

impl BinRead for UldComponent {
	type Args<'a> = &'a [UldComponent];
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, components: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let pos = reader.stream_position()?;
		
		let id = u32::read_options(reader, endian, ())?;
		let ignore_input = u8::read_options(reader, endian, ())? != 0;
		let drag_arrow = u8::read_options(reader, endian, ())? != 0;
		let drop_arrow = u8::read_options(reader, endian, ())? != 0;
		let component_type = ComponentType::read_options(reader, endian, ())?;
		let node_count = u32::read_options(reader, endian, ())?;
		let _size = u16::read_options(reader, endian, ())?;
		let offset = u16::read_options(reader, endian, ())?;
		let component = Component::read_options(reader, endian, (component_type, offset - 16))?;
		
		reader.seek(SeekFrom::Start(pos + offset as u64))?;
		let mut nodes = Vec::with_capacity(node_count as usize);
		for _ in 0..node_count {
			nodes.push(NodeData::read_options(reader, endian, components)?);
		}
		
		Ok(Self {
			id,
			ignore_input,
			drag_arrow,
			drop_arrow,
			// component_type,
			// _node_count: node_count,
			// _size: size,
			// _offset: offset,
			component,
			nodes,
		})
	}
}

impl BinWrite for UldComponent {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		self.id.write_options(writer, endian, ())?;
		(if self.ignore_input {1u8} else {0}).write_options(writer, endian, ())?;
		(if self.drag_arrow {1u8} else {0}).write_options(writer, endian, ())?;
		(if self.drop_arrow {1u8} else {0}).write_options(writer, endian, ())?;
		// self.component_type.write_options(writer, endian, ())?;
		self.component.get_type().write_options(writer, endian, ())?;
		(self.nodes.len() as u32).write_options(writer, endian, ())?;
		let pos = writer.stream_position()?;
		0u16.write_options(writer, endian, ())?;
		0u16.write_options(writer, endian, ())?;
		// size.write_options(writer, endian, ())?;
		// offset.write_options(writer, endian, ())?;
		self.component.write_options(writer, endian, ())?;
		let node_pos = writer.stream_position()?;
		for node in &self.nodes {
			node.write_options(writer, endian, ())?;
		}
		
		let end = writer.stream_position()?;
		writer.seek(SeekFrom::Start(pos))?;
		((end - pos) as u16).write_options(writer, endian, ())?;
		((node_pos - pos) as u16).write_options(writer, endian, ())?;
		writer.seek(SeekFrom::Start(end))?;
		
		Ok(())
	}
}

// ---------------------------------------- //

#[derive(Debug, Clone)]
pub struct UldTimeline {
	pub id: u32,
	// _size: u32,
	// _frames1_count: u16,
	// _frames2_count: u16,
	pub frames1: Vec<FrameData>,
	pub frames2: Vec<FrameData>,
}

impl BinRead for UldTimeline {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let pos = reader.stream_position()?;
		
		let id = u32::read_options(reader, endian, ())?;
		let size = u32::read_options(reader, endian, ())?;
		let frames1_count = u16::read_options(reader, endian, ())?;
		let frames2_count = u16::read_options(reader, endian, ())?;
		
		let mut frames1 = Vec::with_capacity(frames1_count as usize);
		for _ in 0..frames1_count {
			frames1.push(FrameData::read_options(reader, endian, ())?);
		}
		
		let mut frames2 = Vec::with_capacity(frames2_count as usize);
		for _ in 0..frames2_count {
			frames2.push(FrameData::read_options(reader, endian, ())?);
		}
		
		reader.seek(SeekFrom::Start(pos + size as u64))?;
		
		Ok(Self {
			id,
			// _size: size,
			// _frames1_count: frames1_count,
			// _frames2_count: frames2_count,
			frames1,
			frames2,
		})
	}
}

impl BinWrite for UldTimeline {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		let pos = writer.stream_position()?;
		
		self.id.write_options(writer, endian, ())?;
		0u32.write_le(writer)?;
		(self.frames1.len() as u16).write_options(writer, endian, ())?;
		(self.frames2.len() as u16).write_options(writer, endian, ())?;
		for frame in &self.frames1 {
			frame.write_options(writer, endian, ())?;
		}
		for frame in &self.frames2 {
			frame.write_options(writer, endian, ())?;
		}
		
		let size = writer.stream_position()? - pos;
		writer.seek(SeekFrom::Start(pos + 4))?;
		size.write_options(writer, endian, ())?;
		
		Ok(())
	}
}

// ---------------------------------------- //

// #[binrw]
// #[brw(little, repr = u32)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum AlignmentType {
	#[default] TopLeft = 0x0,
	Top = 0x1,
	TopRight = 0x2,
	Left = 0x3,
	Center = 0x4,
	Right = 0x5,
	BottomLeft = 0x6,
	Bottom = 0x7,
	BottomRight = 0x8,
	Unk(u32),
}

#[derive(Debug, Clone)]
pub struct WidgetData {
	pub id: u32,
	pub alignment_type: AlignmentType,
	pub x: i16,
	pub y: i16,
	// _node_count: u16,
	// _size: u16,
	pub nodes: Vec<NodeData>,
}

impl BinRead for WidgetData {
	type Args<'a> = &'a [UldComponent];
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, components: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let id = u32::read_options(reader, endian, ())?;
		// let alignment_type = AlignmentType::read_options(reader, endian, ())?;
		let alighment_type = u32::read_options(reader, endian, ())?;
		let x = i16::read_options(reader, endian, ())?;
		let y = i16::read_options(reader, endian, ())?;
		let node_count = u16::read_options(reader, endian, ())?;
		let _size = u16::read_options(reader, endian, ())?;
		
		let mut nodes = Vec::with_capacity(node_count as usize);
		for _ in 0..node_count {
			nodes.push(NodeData::read_options(reader, endian, components)?);
		}
		
		Ok(Self {
			id,
			alignment_type: match alighment_type {
				0x0 => AlignmentType::TopLeft,
				0x1 => AlignmentType::Top,
				0x2 => AlignmentType::TopRight,
				0x3 => AlignmentType::Left,
				0x4 => AlignmentType::Center,
				0x5 => AlignmentType::Right,
				0x6 => AlignmentType::BottomLeft,
				0x7 => AlignmentType::Bottom,
				0x8 => AlignmentType::BottomRight,
				_ => AlignmentType::Unk(alighment_type),
			},
			x,
			y,
			// _node_count: node_count,
			// _size: size,
			nodes,
		})
	}
}

impl BinWrite for WidgetData {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		let pos = writer.stream_position()?;
		
		self.id.write_options(writer, endian, ())?;
		// self.alignment_type.write_options(writer, endian, ())?;
		match self.alignment_type {
			AlignmentType::TopLeft => 0x0u32,
			AlignmentType::Top => 0x1u32,
			AlignmentType::TopRight => 0x2u32,
			AlignmentType::Left => 0x3u32,
			AlignmentType::Center => 0x4u32,
			AlignmentType::Right => 0x5u32,
			AlignmentType::BottomLeft => 0x6u32,
			AlignmentType::Bottom => 0x7u32,
			AlignmentType::BottomRight => 0x8u32,
			AlignmentType::Unk(v) => v,
		}.write_options(writer, endian, ())?;
		self.x.write_options(writer, endian, ())?;
		self.y.write_options(writer, endian, ())?;
		(self.nodes.len() as u16).write_options(writer, endian, ())?;
		0u16.write_options(writer, endian, ())?;
		for node in &self.nodes {
			node.write_options(writer, endian, ())?;
		}
		
		let size = writer.stream_position()? - pos;
		writer.seek(SeekFrom::Start(pos + 14))?;
		size.write_options(writer, endian, ())?;
		
		Ok(())
	}
}