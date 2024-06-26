use std::io::{Read, Seek, Write, SeekFrom};
use binrw::{binrw, BinRead, BinWrite};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameData {
	pub start_frame: u32,
	pub end_frame: u32,
	// _size: u32,
	// keygroup_count: u32,
	pub keygroups: Vec<KeyGroup>,
}

impl BinRead for FrameData {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let pos = reader.stream_position()?;
		
		let start_frame = u32::read_options(reader, endian, ())?;
		let end_frame = u32::read_options(reader, endian, ())?;
		let size = u32::read_options(reader, endian, ())?;
		let keygroup_count = u32::read_options(reader, endian, ())?;
		let mut keygroups = Vec::with_capacity(keygroup_count as usize);
		for _ in 0..keygroup_count {
			keygroups.push(KeyGroup::read_options(reader, endian, ())?);
		}
		
		reader.seek(SeekFrom::Start(pos + size as u64))?;
		
		Ok(FrameData {
			start_frame,
			end_frame,
			// _size: size,
			// keygroup_count,
			keygroups,
		})
	}
}

impl BinWrite for FrameData {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		self.start_frame.write_options(writer, endian, ())?;
		self.end_frame.write_options(writer, endian, ())?;
		let pos = writer.stream_position()?;
		0u32.write_options(writer, endian, ())?;
		(self.keygroups.len() as u32).write_options(writer, endian, ())?;
		for keygroup in &self.keygroups {
			keygroup.write_options(writer, endian, ())?;
		}
		
		let end = writer.stream_position()?;
		writer.seek(SeekFrom::Start(pos))?;
		((end - pos) as u32).write_options(writer, endian, ())?;
		writer.seek(SeekFrom::Start(end))?;
		
		Ok(())
	}
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeyGroup {
	pub usage: KeyUsage,
	// keygroup_type: KeyGroupType,
	// _size: u16,
	// frame_count: u16,
	pub frames: Keyframes,
}

impl BinRead for KeyGroup {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let pos = reader.stream_position()?;
		
		let usage = KeyUsage::read_options(reader, endian, ())?;
		let keygroup_type = KeyGroupType::read_options(reader, endian, ())?;
		let size = u16::read_options(reader, endian, ())?;
		let frame_count = u16::read_options(reader, endian, ())?;
		let frames = Keyframes::read_options(reader, endian, (keygroup_type, frame_count))?;
		// let mut frames = Vec::with_capacity(frame_count as usize);
		// for _ in 0..frame_count {
		// 	frames.push(Keyframe::read_options(reader, endian, (keygroup_type,))?);
		// }
		
		reader.seek(SeekFrom::Start(pos + size as u64))?;
		
		Ok(KeyGroup {
			usage,
			// keygroup_type,
			// _size: size,
			// frame_count,
			frames,
		})
	}
}

impl BinWrite for KeyGroup {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		self.usage.write_options(writer, endian, ())?;
		// self.frames.get(0).map_or(KeyGroupType::Float1, |v| v.get_type()).write_options(writer, endian, ())?;
		self.frames.get_type().write_options(writer, endian, ())?;
		let pos = writer.stream_position()?;
		0u16.write_options(writer, endian, ())?;
		self.frames.write_options(writer, endian, ())?;
		// (self.frames.len() as u16).write_options(writer, endian, ())?;
		// for frame in &self.frames {
		// 	frame.write_options(writer, endian, ())?;
		// }
		
		let end = writer.stream_position()?;
		writer.seek(SeekFrom::Start(pos))?;
		((end - pos) as u16).write_options(writer, endian, ())?;
		writer.seek(SeekFrom::Start(end))?;
		
		Ok(())
	}
}

// ---------------------------------------- //

#[binrw]
#[brw(little, repr = u16)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyUsage {
	#[default] Position = 0x0,
	Rotation = 0x1,
	Scale = 0x2,
	Alpha = 0x3,
	NodeColor = 0x4,
	TextColor = 0x5,
	EdgeColor = 0x6,
	Number = 0x7,
}

#[binrw]
#[brw(little, repr = u16)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyGroupType {
	#[default] Float1 = 0x0,
	Float2 = 0x1,
	Float3 = 0x2,
	SByte1 = 0x3,
	SByte2 = 0x4,
	SByte3 = 0x5,
	Byte1 = 0x6,
	Byte2 = 0x7,
	Byte3 = 0x8,
	Short1 = 0x9,
	Short2 = 0xA,
	Short3 = 0xB,
	UShort1 = 0xC,
	UShort2 = 0xD,
	UShort3 = 0xE,
	Int1 = 0xF,
	Int2 = 0x10,
	Int3 = 0x11,
	UInt1 = 0x12,
	UInt2 = 0x13,
	UInt3 = 0x14,
	Bool1 = 0x15,
	Bool2 = 0x16,
	Bool3 = 0x17,
	Color = 0x18,
	Label = 0x19,
	// lumina has this but does nothing with it, we'll just let the binread error
	// Number = 0x1A,
}

// ---------------------------------------- //

#[derive(Debug, Clone, PartialEq)]
pub enum Keyframes {
	Float1(Vec<Float1Keyframe>),
	Float2(Vec<Float2Keyframe>),
	Float3(Vec<Float3Keyframe>),
	SByte1(Vec<SByte1Keyframe>),
	SByte2(Vec<SByte2Keyframe>),
	SByte3(Vec<SByte3Keyframe>),
	Byte1(Vec<Byte1Keyframe>),
	Byte2(Vec<Byte2Keyframe>),
	Byte3(Vec<Byte3Keyframe>),
	Short1(Vec<Short1Keyframe>),
	Short2(Vec<Short2Keyframe>),
	Short3(Vec<Short3Keyframe>),
	UShort1(Vec<UShort1Keyframe>),
	UShort2(Vec<UShort2Keyframe>),
	UShort3(Vec<UShort3Keyframe>),
	Int1(Vec<Int1Keyframe>),
	Int2(Vec<Int2Keyframe>),
	Int3(Vec<Int3Keyframe>),
	UInt1(Vec<UInt1Keyframe>),
	UInt2(Vec<UInt2Keyframe>),
	UInt3(Vec<UInt3Keyframe>),
	Bool1(Vec<Bool1Keyframe>),
	Bool2(Vec<Bool2Keyframe>),
	Bool3(Vec<Bool3Keyframe>),
	Color(Vec<ColorKeyframe>),
	Label(Vec<LabelKeyframe>),
}

impl Keyframes {
	pub fn get_type(&self) -> KeyGroupType {
		match self {
			Self::Float1(_) => KeyGroupType::Float1,
			Self::Float2(_) => KeyGroupType::Float2,
			Self::Float3(_) => KeyGroupType::Float3,
			Self::SByte1(_) => KeyGroupType::SByte1,
			Self::SByte2(_) => KeyGroupType::SByte2,
			Self::SByte3(_) => KeyGroupType::SByte3,
			Self::Byte1(_) => KeyGroupType::Byte1,
			Self::Byte2(_) => KeyGroupType::Byte2,
			Self::Byte3(_) => KeyGroupType::Byte3,
			Self::Short1(_) => KeyGroupType::Short1,
			Self::Short2(_) => KeyGroupType::Short2,
			Self::Short3(_) => KeyGroupType::Short3,
			Self::UShort1(_) => KeyGroupType::UShort1,
			Self::UShort2(_) => KeyGroupType::UShort2,
			Self::UShort3(_) => KeyGroupType::UShort3,
			Self::Int1(_) => KeyGroupType::Int1,
			Self::Int2(_) => KeyGroupType::Int2,
			Self::Int3(_) => KeyGroupType::Int3,
			Self::UInt1(_) => KeyGroupType::UInt1,
			Self::UInt2(_) => KeyGroupType::UInt2,
			Self::UInt3(_) => KeyGroupType::UInt3,
			Self::Bool1(_) => KeyGroupType::Bool1,
			Self::Bool2(_) => KeyGroupType::Bool2,
			Self::Bool3(_) => KeyGroupType::Bool3,
			Self::Color(_) => KeyGroupType::Color,
			Self::Label(_) => KeyGroupType::Label,
		}
	}
}

impl Default for Keyframes {
	fn default() -> Self {
		Self::Float1(Default::default())
	}
}

impl BinRead for Keyframes {
	type Args<'a> = (KeyGroupType, u16);
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, (typ, count): Self::Args<'_>,) -> binrw::BinResult<Self> {
		Ok(match typ {
			KeyGroupType::Float1 => Self::Float1((0..count).map(|_| Float1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Float2 => Self::Float2((0..count).map(|_| Float2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Float3 => Self::Float3((0..count).map(|_| Float3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::SByte1 => Self::SByte1((0..count).map(|_| SByte1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::SByte2 => Self::SByte2((0..count).map(|_| SByte2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::SByte3 => Self::SByte3((0..count).map(|_| SByte3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Byte1 => Self::Byte1((0..count).map(|_| Byte1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Byte2 => Self::Byte2((0..count).map(|_| Byte2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Byte3 => Self::Byte3((0..count).map(|_| Byte3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Short1 => Self::Short1((0..count).map(|_| Short1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Short2 => Self::Short2((0..count).map(|_| Short2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Short3 => Self::Short3((0..count).map(|_| Short3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::UShort1 => Self::UShort1((0..count).map(|_| UShort1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::UShort2 => Self::UShort2((0..count).map(|_| UShort2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::UShort3 => Self::UShort3((0..count).map(|_| UShort3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Int1 => Self::Int1((0..count).map(|_| Int1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Int2 => Self::Int2((0..count).map(|_| Int2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Int3 => Self::Int3((0..count).map(|_| Int3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::UInt1 => Self::UInt1((0..count).map(|_| UInt1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::UInt2 => Self::UInt2((0..count).map(|_| UInt2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::UInt3 => Self::UInt3((0..count).map(|_| UInt3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Bool1 => Self::Bool1((0..count).map(|_| Bool1Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Bool2 => Self::Bool2((0..count).map(|_| Bool2Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Bool3 => Self::Bool3((0..count).map(|_| Bool3Keyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Color => Self::Color((0..count).map(|_| ColorKeyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
			KeyGroupType::Label => Self::Label((0..count).map(|_| LabelKeyframe::read_options(reader, endian, ())).collect::<binrw::BinResult<_>>()?),
		})
	}
}

impl BinWrite for Keyframes {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		match self {
			Self::Float1(v) => v.write_options(writer, endian, ())?,
			Self::Float2(v) => v.write_options(writer, endian, ())?,
			Self::Float3(v) => v.write_options(writer, endian, ())?,
			Self::SByte1(v) => v.write_options(writer, endian, ())?,
			Self::SByte2(v) => v.write_options(writer, endian, ())?,
			Self::SByte3(v) => v.write_options(writer, endian, ())?,
			Self::Byte1(v) => v.write_options(writer, endian, ())?,
			Self::Byte2(v) => v.write_options(writer, endian, ())?,
			Self::Byte3(v) => v.write_options(writer, endian, ())?,
			Self::Short1(v) => v.write_options(writer, endian, ())?,
			Self::Short2(v) => v.write_options(writer, endian, ())?,
			Self::Short3(v) => v.write_options(writer, endian, ())?,
			Self::UShort1(v) => v.write_options(writer, endian, ())?,
			Self::UShort2(v) => v.write_options(writer, endian, ())?,
			Self::UShort3(v) => v.write_options(writer, endian, ())?,
			Self::Int1(v) => v.write_options(writer, endian, ())?,
			Self::Int2(v) => v.write_options(writer, endian, ())?,
			Self::Int3(v) => v.write_options(writer, endian, ())?,
			Self::UInt1(v) => v.write_options(writer, endian, ())?,
			Self::UInt2(v) => v.write_options(writer, endian, ())?,
			Self::UInt3(v) => v.write_options(writer, endian, ())?,
			Self::Bool1(v) => v.write_options(writer, endian, ())?,
			Self::Bool2(v) => v.write_options(writer, endian, ())?,
			Self::Bool3(v) => v.write_options(writer, endian, ())?,
			Self::Color(v) => v.write_options(writer, endian, ())?,
			Self::Label(v) => v.write_options(writer, endian, ())?,
		}
		
		Ok(())
	}
}

// ---------------------------------------- //

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BaseKeyframeData {
	pub time: u32,
	pub offset: u16,
	pub interpolation: u8,
	pub unk1: u8,
	pub acceleration: f32,
	pub deceleration: f32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Float1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: f32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Float2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [f32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Float3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [f32; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SByte1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: i8,
	pub padding: [u8; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SByte2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [i8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SByte3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [i8; 3],
	pub padding: u8,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Byte1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: u8,
	pub padding: [u8; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Byte2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [u8; 2],
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Byte3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [u8; 3],
	pub padding: u8,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Short1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: i16,
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Short2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [i16; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Short3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [i16; 3],
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UShort1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: u16,
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UShort2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [u16; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UShort3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [u16; 3],
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Int1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Int2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [i32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Int3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [i32; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UInt1Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UInt2Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [u32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UInt3Keyframe {
	pub keyframe: BaseKeyframeData,
	pub value: [u32; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bool1Keyframe {
	pub keyframe: BaseKeyframeData,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub value: bool,
	pub padding: [u8; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bool2Keyframe {
	pub keyframe: BaseKeyframeData,
	#[br(map = |v: [u8; 2]| [v[0] != 0, v[1] != 0])]
	#[bw(map = |v: &[bool; 2]| [if v[0] {1u8} else {0}, if v[1] {1u8} else {0}])]
	pub value: [bool; 2],
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bool3Keyframe {
	pub keyframe: BaseKeyframeData,
	#[br(map = |v: [u8; 3]| [v[0] != 0, v[1] != 0, v[2] != 0])]
	#[bw(map = |v: &[bool; 3]| [if v[0] {1u8} else {0}, if v[1] {1u8} else {0}, if v[2] {1u8} else {0}])]
	pub value: [bool; 3],
	pub padding: u8,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ColorKeyframe {
	pub keyframe: BaseKeyframeData,
	pub multiply_red: i16,
	pub multiply_green: i16,
	pub multiply_blue: i16,
	pub add_red: i16,
	pub add_green: i16,
	pub add_blue: i16,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LabelKeyframe {
	pub keyframe: BaseKeyframeData,
	pub label_id: u16,
	pub label_command: u8,
	pub jump_id: u8,
}