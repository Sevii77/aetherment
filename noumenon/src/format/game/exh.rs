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
	pub(crate) columns: Vec<Column>,
	#[br(count = page_count)]
	pages: Vec<Page>,
	#[br(count = language_count)]
	languages: Vec<Language>,
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
	pub(crate) kind: ColumnKind,
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
			ColumnKind::PackedBool1 => 1,
			ColumnKind::PackedBool2 => 1,
			ColumnKind::PackedBool3 => 1,
			ColumnKind::PackedBool4 => 1,
			ColumnKind::PackedBool5 => 1,
			ColumnKind::PackedBool6 => 1,
			ColumnKind::PackedBool7 => 1,
		}
	}
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct Page {
	start_id: u32,
	row_count: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct Language {
	language: u8,
	_unk1: u8,
}