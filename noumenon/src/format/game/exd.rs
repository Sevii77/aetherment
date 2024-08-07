use std::io::{Read, Seek, Write};
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
	
	// pub fn get_fields_mut(&mut self, row: usize, sub_row: usize, header: Exh) -> Option<Vec<&mut Field>> {
	// 	None
	// }
}