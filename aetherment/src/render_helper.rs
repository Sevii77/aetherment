pub trait EnumTools {
	type Iterator: core::iter::Iterator<Item = Self>;
	
	fn to_str(&self) -> &'static str;
	fn to_string(&self) -> String {self.to_str().to_string()}
	fn iter() -> Self::Iterator;
}