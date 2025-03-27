use std::{collections::BTreeMap, sync::{LazyLock, RwLock}};

static COLORS: LazyLock<RwLock<BTreeMap<(bool, u32), u32>>> = LazyLock::new(|| RwLock::new(BTreeMap::new()));

fn update_colors() {
	let funcs = unsafe{super::FUNCS.as_ref().unwrap()};
	let Ok(colors) = COLORS.read() else {return};
	(funcs.uicolor)(&colors);
}

pub fn clear_colors() {
	COLORS.try_write().unwrap().clear();
	update_colors();
}

pub fn set_color(use_theme: bool, index: u32, [r, g, b]: [u8; 3]) {
	COLORS.try_write().unwrap().insert((use_theme, index), r as u32 | (g as u32) << 8 | (b as u32) << 16 | 0xFF000000);
	update_colors();
}

pub fn remove_color(use_theme: bool, index: u32) {
	COLORS.try_write().unwrap().remove(&(use_theme, index));
	update_colors();
}

pub fn get_colors() -> BTreeMap<(bool, u32), [u8; 3]> {
	COLORS.read().unwrap().iter().map(|(index, clr)| (
		*index,
		[(clr & 0xFF) as u8, ((clr >> 8) & 0xFF) as u8, ((clr >> 16) & 0xFF) as u8]
	)).collect()
}