// TODO: support other value types (maybe strings for localization changes? idk)

use std::{borrow::Cow, collections::HashMap, io::Cursor, ops::DerefMut};
use noumenon::format::game::exd::Field;
use serde::{Deserialize, Serialize};
use crate::modman::OptionOrStatic;

#[derive(Debug)]
pub enum CompositeError {
	BannedExdFile,
	NoFileResolverReturnExh,
	NoFileResolverReturnExd,
	InvalidRow{row: u32},
	InvalidColumn{column: u32},
	UnsupportedColumnType{column: u32},
	ValueResolveFailure,
}

impl From<CompositeError> for super::CompositeError {
	fn from(value: CompositeError) -> Self {
		super::CompositeError::Exd(value)
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Exd {
	pub path: String,
	pub rows: HashMap<u32, HashMap<u32, OptionOrStatic<[f32; 4]>>>,
}

impl super::Composite for Exd {
	fn get_files(&self) -> Vec<&str> {
		Vec::new()
	}
	
	fn get_files_game(&self) -> Vec<&str> {
		vec![&self.path]
	}
	
	fn get_options(&self) -> Vec<&str> {
		let mut options = Vec::new();
		for (_, columns) in &self.rows {
			for (_, opt) in columns {
				match opt {
					OptionOrStatic::Option(v) => options.push(v.as_str()),
					OptionOrStatic::OptionSub(v, _) => options.push(v.as_str()),
					OptionOrStatic::OptionMul(v, _) => options.push(v.as_str()),
					OptionOrStatic::OptionGradiant(v, v2, _) => {options.push(v.as_str()); options.push(v2.as_str())},
					OptionOrStatic::Static(_) => {},
				}
			}
		}
		
		options
	}
	
	fn composite<'a>(&self, meta: &crate::modman::meta::Meta, settings: &crate::modman::settings::CollectionSettings, file_resolver: &dyn Fn(&crate::modman::Path) -> Option<Cow<'a, Vec<u8>>>) -> Result<Vec<u8>, super::CompositeError> {
		// No clue what change certain exd files might cause, so block any others for now
		if self.path != "exd/uicolor_0.exd" {return Err(CompositeError::BannedExdFile.into())}
		
		let header_path = crate::modman::Path::Game(format!("{}.exh", &self.path[0..self.path.len() - 6]));
		let header = noumenon::format::game::Exh::read(&mut Cursor::new(file_resolver(&header_path).ok_or(CompositeError::NoFileResolverReturnExh)?.as_ref()))?;
		let sheet_path = crate::modman::Path::Game(self.path.clone());
		let mut sheet = noumenon::format::game::Exd::read(&mut Cursor::new(file_resolver(&sheet_path).ok_or(CompositeError::NoFileResolverReturnExd)?.as_ref()))?;
		
		for (row, columns) in &self.rows {
			for (column, val) in columns {
				let mut fields = sheet.get_fields_mut(*row, 0, &header).ok_or(CompositeError::InvalidRow{row: *row})?;
				let val = val.resolve(meta, settings).ok_or(CompositeError::ValueResolveFailure)?;
				
				match fields.get_mut(*column as usize).ok_or(CompositeError::InvalidColumn{column: *column})? {
					Field::U32(v) => {
						*v.deref_mut() =
							(((val[0] * 255.0).clamp(0.0, 255.0) as u32) << 24) +
							(((val[1] * 255.0).clamp(0.0, 255.0) as u32) << 16) +
							(((val[2] * 255.0).clamp(0.0, 255.0) as u32) << 8) +
							((val[3] * 255.0).clamp(0.0, 255.0) as u32)
					},
					
					_ => return Err(CompositeError::UnsupportedColumnType{column: *column}.into()),
				}
			}
		}
		
		let mut data = Vec::new();
		sheet.write(&mut Cursor::new(&mut data))?;
		Ok(data)
	}
}