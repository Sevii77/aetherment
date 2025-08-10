// Pretty much only exists for exd

use std::io::{Read, Seek, Write};
use binrw::{binrw, BinRead, BinWrite};

pub type Error = binrw::Error;

#[binrw]
#[brw(big, magic = b"EXHF")]
#[derive(Debug, Clone)]
pub struct Exh {
	_version: u16,
	row_size: u16,
	column_count: u16,
	page_count: u16,
	language_count: u16,
	_unk1: u16,
	_unk2: u8,
	sheet_type: u8,
	_unk3: u16,
	row_count: u32,
	_unk4: [u8; 8],
	
	#[br(count = column_count)]
	pub columns: Vec<Column>,
	#[br(count = page_count)]
	pub pages: Vec<Page>,
	#[br(count = language_count)]
	pub languages: Vec<LanguageSeg>,
}

impl ironworks::file::File for Exh {
	fn read(mut data: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
		<Exh as crate::format::external::Bytes<Error>>::read(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

impl crate::format::external::Bytes<Error> for Exh {
	fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		Ok(Exh::read_be(reader)?)
	}
	
	fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.write_be(writer)?;
		
		Ok(())
	}
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct Column {
	pub kind: ColumnKind,
	pub(crate) offset: u16,
}

#[binrw]
#[brw(big, repr = u16)]
#[repr(u16)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnKind {
	String = 0x0,
	Bool = 0x1,
	I8 = 0x2,
	U8 = 0x3,
	I16 = 0x4,
	U16 = 0x5,
	I32 = 0x6,
	U32 = 0x7,
	// Unk1 = 0x8,
	F32 = 0x9,
	I64 = 0xA,
	U64 = 0xB,
	// Unk2 = 0xC,
	PackedBool0 = 0x19,
	PackedBool1 = 0x1A,
	PackedBool2 = 0x1B,
	PackedBool3 = 0x1C,
	PackedBool4 = 0x1D,
	PackedBool5 = 0x1E,
	PackedBool6 = 0x1F,
	PackedBool7 = 0x20,
}

impl ColumnKind {
	pub fn len(&self) -> usize {
		match self {
			ColumnKind::String => 4,
			ColumnKind::Bool => 1,
			ColumnKind::I8 => 1,
			ColumnKind::U8 => 1,
			ColumnKind::I16 => 2,
			ColumnKind::U16 => 2,
			ColumnKind::I32 => 4,
			ColumnKind::U32 => 4,
			ColumnKind::F32 => 4,
			ColumnKind::I64 => 8,
			ColumnKind::U64 => 8,
			ColumnKind::PackedBool0 => 1,
			ColumnKind::PackedBool1 => 0,
			ColumnKind::PackedBool2 => 0,
			ColumnKind::PackedBool3 => 0,
			ColumnKind::PackedBool4 => 0,
			ColumnKind::PackedBool5 => 0,
			ColumnKind::PackedBool6 => 0,
			ColumnKind::PackedBool7 => 0,
		}
	}
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct Page {
	pub start_id: u32,
	pub row_count: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct LanguageSeg {
	pub language: u8,
	_unk1: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Language {
	None = 0,
	Japanese = 1,
	English = 2,
	German = 3,
	French = 4,
	ChineseSimplified = 5,
	ChineseTraditional = 6,
	Korean = 7,
	ChineseTraditional2 = 8,
}

impl Language {
	pub fn code(&self) -> Option<&'static str> {
		match self {
			Language::None                => None,
			Language::Japanese            => Some("ja"),
			Language::English             => Some("en"),
			Language::German              => Some("de"),
			Language::French              => Some("fr"),
			Language::ChineseSimplified   => Some("chs"),
			Language::ChineseTraditional  => Some("cht"),
			Language::Korean              => Some("ko"),
			Language::ChineseTraditional2 => Some("tc"),
		}
	}
}