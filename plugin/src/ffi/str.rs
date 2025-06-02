pub(crate) static mut DROP: fn(*const u8, usize) = |_, _| {};

#[repr(C, packed)]
pub struct FfiString {
	ptr: *const u8,
	size: usize,
}

impl FfiString {
	pub fn as_str(&self) -> &str {
		unsafe{std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr, self.size))}
	}
}

impl ToString for FfiString {
	fn to_string(&self) -> std::string::String {
		self.as_str().to_string()
	}
}

impl Drop for FfiString {
	fn drop(&mut self) {
		unsafe{DROP(self.ptr, self.size)}
	}
}

#[repr(C, packed)]
pub struct FfiStr<'s> {
	ptr: *const u8,
	size: usize,
	_phantom: std::marker::PhantomData<&'s ()>,
}

impl<'s> FfiStr<'s> {
	pub fn new(s: &str) -> Self {
		Self {
			ptr: s.as_ptr(),
			size: s.len(),
			_phantom: std::marker::PhantomData,
		}
	}
}

// impl String {
// 	fn new(s: &str) -> Self {
// 		Self(s.as_ptr(), s.len())
// 	}
// 	
// 	fn to_string(&self) -> String {
// 		unsafe{std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.0, self.1)).to_string()}
// 	}
// 	
// 	fn to_string_vec(&self) -> Vec<String> {
// 		self.to_string().split('\0').map(|v| v.to_string()).collect()
// 	}
// }