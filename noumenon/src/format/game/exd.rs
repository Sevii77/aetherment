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
		let fields_length = header.columns.iter().map(|v| v.kind.len()).sum::<usize>();
		let mut fields = Vec::new();
		
		for c in &header.columns {
			let l = c.kind.len();
			let o = c.offset as usize;
			
			fields.push(match c.kind {
				ColumnKind::String => {
					let o = u32::from_be_bytes(row[o..o + 4].try_into().unwrap()) as usize;
					Field::String(FieldString::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + fields_length + 1 + o) as *mut u8, row.len() - fields_length - o - 1)}))
				},
				
				ColumnKind::Bool => Field::Bool(FieldBool::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::I8 => Field::I8(FieldI8::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::U8 => Field::U8(FieldU8::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::I16 => Field::I16(FieldI16::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::U16 => Field::U16(FieldU16::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::I32 => Field::I32(FieldI32::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::U32 => Field::U32(FieldU32::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::F32 => Field::F32(FieldF32::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::I64 => Field::I64(FieldI64::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::U64 => Field::U64(FieldU64::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				
				ColumnKind::PackedBool0 => Field::PackedBool0(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool1 => Field::PackedBool1(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool2 => Field::PackedBool2(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool3 => Field::PackedBool3(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool4 => Field::PackedBool4(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool5 => Field::PackedBool5(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool6 => Field::PackedBool6(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
				ColumnKind::PackedBool7 => Field::PackedBool7(FieldPacked::new(unsafe{std::slice::from_raw_parts_mut((row.as_mut_ptr() as usize + o) as *mut u8, l)})),
			});
		}
		
		Some(fields)
	}
}

#[derive(Debug)]
pub enum Field<'a> {
	String(FieldString),
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
	PackedBool0(FieldPacked<'a, 0>),
	PackedBool1(FieldPacked<'a, 1>),
	PackedBool2(FieldPacked<'a, 2>),
	PackedBool3(FieldPacked<'a, 3>),
	PackedBool4(FieldPacked<'a, 4>),
	PackedBool5(FieldPacked<'a, 5>),
	PackedBool6(FieldPacked<'a, 6>),
	PackedBool7(FieldPacked<'a, 7>),
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

/// Does not support modifying
#[derive(Debug)]
pub struct FieldString {
	val: String,
}

impl Deref for FieldString {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.val
	}
}

impl DerefMut for FieldString {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.val
	}
}

impl FieldString {
	pub(crate) fn new(val: &mut [u8]) -> Self {
		Self {
			val: crate::NullReader::null_terminated(val).unwrap(),
		}
	}
}

#[derive(Debug)]
pub struct FieldPacked<'a, const B: u8> {
	inner: &'a mut [u8],
	val: bool,
}

impl<'a, const B: u8> Deref for FieldPacked<'a, B> {
	type Target = bool;

	fn deref(&self) -> &Self::Target {
		&self.val
	}
}

impl<'a, const B: u8> DerefMut for FieldPacked<'a, B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.val
	}
}

impl<'a, const B: u8> FieldPacked<'a, B> {
	pub(crate) fn new(val: &'a mut [u8]) -> Self {
		Self {
			val: val[0] & (1 << B) == 1 << B,
			inner: val,
		}
	}
}

impl<'a, const B: u8> Drop for FieldPacked<'a, B> {
	fn drop(&mut self) {
		self.inner[0] = self.inner[0] & !(1 << B) | (if self.val {1} else {0} << B)
	}
}