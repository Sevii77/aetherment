use renderer::*;

pub trait EnumTools {
	type Iterator: core::iter::Iterator<Item = Self>;
	
	fn to_str(&self) -> &'static str;
	fn to_string(&self) -> String {self.to_str().to_string()}
	fn iter() -> Self::Iterator;
}

pub trait UiExt {
	fn combo_enum<S: AsRef<str>, Enum: EnumTools + PartialEq>(&mut self, label: S, val: &mut Enum) -> bool;
}

impl<'a> UiExt for Ui<'a> {
	fn combo_enum<S: AsRef<str>, Enum: EnumTools + PartialEq>(&mut self, label: S, val: &mut Enum) -> bool {
		let mut changed = false;
		self.combo(label, val.to_str(), |ui| {
			for item in Enum::iter() {
				let name = item.to_str();
				changed |= ui.selectable_value(name, val, item).clicked;
			}
		});
		
		changed
	}
}