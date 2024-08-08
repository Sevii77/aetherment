use std::{io::{Read, Seek, Write}, ops::{Deref, DerefMut}};
use binrw::{binrw, BinRead, BinWrite};
use crate::Error;

#[binrw]
#[brw(big, magic = b"EXDF")]
#[derive(Debug, Clone)]
pub struct Exd {
	_version: u16,
	_unk1: u16,
	// #[bw(calc = rows.len() as u32 * 8)]
	rows_size: u32,
	_unk2: [u8; 20],
	#[br(count = rows_size / 8)]
	rows: Vec<(u32, u32)>, // id, offset
	#[br(parse_with = binrw::helpers::until_eof)]
	data: Vec<u8>,
}

impl ironworks::file::File for Exd {
	fn read(mut data: impl ironworks::FileStream) -> super::Result<Self> {
		Exd::read(&mut data).map_err(|e| ironworks::Error::Resource(e.into()))
	}
}

impl Exd {
	pub fn read<T>(reader: &mut T) -> Result<Self, Error>
	where T: Read + Seek {
		Ok(Exd::read_be(reader)?)
	}
	
	pub fn write<T>(&self, writer: &mut T) -> Result<(), Error> where
	T: Write + Seek {
		self.write_be(writer)?;
		
		Ok(())
	}
	
	pub fn get_row_mut(&mut self, row: u32, sub_row: u32) -> Option<&mut [u8]> {
		let row_offset = self.rows.iter().find_map(|(id, offset)| if row == *id {Some(*offset)} else {None})?;
		let data_offset = 4 + 2 + 2 + 4 + 20 + (8 * self.rows.len());
		let row_size = u32::from_be_bytes(self.data[row_offset as usize - data_offset..row_offset as usize - data_offset + 4].try_into().unwrap());
		let row_count = u16::from_be_bytes(self.data[row_offset as usize - data_offset + 4..row_offset as usize - data_offset + 6].try_into().unwrap());
		let sub_row_size = row_size / row_count as u32;
		let offset = row_offset as usize - data_offset + 6;
		let offset = (0..row_count as u32).into_iter().find_map(|sub_id| {
			let o = offset + (sub_row_size * sub_id) as usize;
			if u16::from_be_bytes(self.data[o + sub_row_size as usize - 2..o + sub_row_size as usize].try_into().unwrap()) as u32 == sub_row {Some(o)} else {None}
		})?;
		
		Some(&mut self.data[offset..offset + sub_row_size as usize - 2])
	}
	
	pub fn get_fields_mut(&mut self, row: u32, sub_row: u32, header: &super::Exh) -> Option<Vec<Field>> {
		use super::exh::ColumnKind;
		
		let row = self.get_row_mut(row, sub_row)?;
		let mut fields = Vec::new();
		let mut offset = 0;
		for c in &header.columns {
			fields.push(match c.kind {
				ColumnKind::Bool => {offset += 1; Field::Bool(FieldBool::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 1) as *mut u8, 1)}))}
				ColumnKind::I8 => {offset += 1; Field::I8(FieldI8::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 1) as *mut u8, 1)}))}
				ColumnKind::U8 => {offset += 1; Field::U8(FieldU8::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 1) as *mut u8, 1)}))}
				ColumnKind::I16 => {offset += 2; Field::I16(FieldI16::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 2) as *mut u8, 2)}))}
				ColumnKind::U16 => {offset += 2; Field::U16(FieldU16::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 2) as *mut u8, 2)}))}
				ColumnKind::I32 => {offset += 4; Field::I32(FieldI32::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 4) as *mut u8, 4)}))}
				ColumnKind::U32 => {offset += 4; Field::U32(FieldU32::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 4) as *mut u8, 4)}))}
				ColumnKind::F32 => {offset += 4; Field::F32(FieldF32::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 4) as *mut u8, 4)}))}
				ColumnKind::I64 => {offset += 8; Field::I64(FieldI64::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 8) as *mut u8, 8)}))}
				ColumnKind::U64 => {offset += 8; Field::U64(FieldU64::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + offset - 8) as *mut u8, 8)}))}
				// TODO: support the rest
				_ => return None
			})
		}
		
		Some(fields)
	}
}

#[derive(Debug)]
pub enum Field<'a> {
	// String = 0x0,
	Bool(FieldBool<'a>),
	I8(FieldI8<'a>),
	U8(FieldU8<'a>),
	I16(FieldI16<'a>),
	U16(FieldU16<'a>),
	I32(FieldI32<'a>),
	U32(FieldU32<'a>),
	F32(FieldF32<'a>),
	I64(FieldI64<'a>),
	U64(FieldU64<'a>),
	// PackedBool0 = 0x19,
	// PackedBool1 = 0x1A,
	// PackedBool2 = 0x1B,
	// PackedBool3 = 0x1C,
	// PackedBool4 = 0x1D,
	// PackedBool5 = 0x1E,
	// PackedBool6 = 0x1F,
	// PackedBool7 = 0x20,
}

macro_rules! create_field {
	(base, $n:ident, $t:ident) => {
		#[derive(Debug)]
		pub struct $n<'a> {
			inner: &'a mut [u8],
			val: $t,
		}
		
		impl<'a> Deref for $n<'a> {
			type Target = $t;
		
			fn deref(&self) -> &Self::Target {
				&self.val
			}
		}
		
		impl<'a> DerefMut for $n<'a> {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.val
			}
		}
	};
	
	($n:ident, $t:ident, false) => {
		create_field!(base, $n, $t);
	};
	
	($n:ident, $t:ident) => {
		create_field!(base, $n, $t);
		
		impl<'a> $n<'a> {
			pub(crate) fn new(val: &'a mut [u8]) -> Self {
				Self {
					val: $t::from_be_bytes(val.try_into().unwrap()),
					inner: val,
				}
			}
		}
		
		impl<'a> Drop for $n<'a> {
			fn drop(&mut self) {
				// self.inner.write_all(&self.val.to_be_bytes());
				for (i, v) in self.val.to_be_bytes().into_iter().enumerate() {
					self.inner[i] = v;
				}
			}
		}
	}
}

create_field!(FieldI8, i8);
create_field!(FieldU8, u8);
create_field!(FieldI16, i16);
create_field!(FieldU16, u16);
create_field!(FieldI32, i32);
create_field!(FieldU32, u32);
create_field!(FieldF32, f32);
create_field!(FieldI64, i64);
create_field!(FieldU64, u64);

create_field!(FieldBool, bool, false);
impl<'a> FieldBool<'a> {
	pub(crate) fn new(val: &'a mut [u8]) -> Self {
		Self {
			val: if val[0] == 1 {true} else {false},
			inner: val,
		}
	}
}

impl<'a> Drop for FieldBool<'a> {
	fn drop(&mut self) {
		self.inner[0] = if self.val {1} else {0}
	}
}