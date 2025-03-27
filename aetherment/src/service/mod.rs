use std::collections::BTreeMap;

pub(crate) mod uicolor;

pub struct ServicesInitializers {
	pub uicolor: Box<dyn Fn(&BTreeMap<(bool, u32), u32>)>,
}

static mut FUNCS: Option<ServicesInitializers> = None;

pub fn initialize(funcs: ServicesInitializers) {
	unsafe{FUNCS = Some(funcs)}
}

pub fn disable() {
	uicolor::clear_colors();
}