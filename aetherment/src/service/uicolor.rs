use std::{collections::BTreeMap, sync::{LazyLock, RwLock}};

use retour::GenericDetour;

static GET_COLOR_SIG: &str = "4C 8B 91 ?? ?? ?? ?? 4C 8B D9 49 8B 02";
static mut GET_COLOR_HOOK: Option<GenericDetour::<unsafe extern "system" fn(usize, bool, u32) -> u32>> = None;
static COLORS: LazyLock<RwLock<BTreeMap<(bool, u32), u32>>> = LazyLock::new(|| RwLock::new(BTreeMap::new()));

// static_detour! {
// 	static GetColor: extern "system" fn(bool, u32) -> u32;
// }

unsafe extern "system" fn get_color(this: usize, use_theme: bool, index: u32) -> u32 {
	'theme: {
		// log!("trying to get color {index}; use theme: {use_theme}");
		
		let Ok(colors) = COLORS.read() else {break 'theme};
		let Some(color) = colors.get(&(use_theme, index)) else {break 'theme};
		
		return *color;
	}
	
	// GetColor.call(use_theme, index)
	GET_COLOR_HOOK.as_ref().unwrap().call(this, use_theme, index)
}

pub unsafe fn initialize() {
	let get_color_ptr = super::scan(GET_COLOR_SIG).unwrap();
	log!("get_color: {:X?}", get_color_ptr);
	// GetColor.initialize(std::mem::transmute(get_color_offset as *const ()), get_color).unwrap().enable().unwrap();
	let hook = GenericDetour::<unsafe extern "system" fn(usize, bool, u32) -> u32>::new(std::mem::transmute(get_color_ptr as *const ()), get_color).unwrap();
	if let Err(err) = hook.enable() {
		log!(err, "Failure enabling hook\n{err:?}");
	}
	GET_COLOR_HOOK = Some(hook);
	
	// clear_colors();
	// set_color(true, 1, [0xFF, 0, 0]);
	// set_color(true, 2, [0xDD, 0, 0]);
	// set_color(true, 3, [0xBB, 0, 0]);
	// set_color(true, 4, [0x99, 0, 0]);
	// set_color(true, 5, [0x77, 0, 0]);
	// set_color(true, 6, [0x55, 0, 0]);
	// set_color(true, 7, [0x33, 0, 0]);
	// set_color(true, 8, [0xFF, 0, 0]);
	// set_color(true, 22, [0xFF, 0, 0]);
	// set_color(true, 50, [0xFF, 0, 0]);
	// set_color(true, 64, [0xFF, 0, 0]);
}

pub unsafe fn disable() {
	if let Some(hook) = GET_COLOR_HOOK.as_ref() {
		if let Err(err) = hook.disable() {
			log!(err, "Failure disabling hook\n{err:?}");
		}
	}
}

pub fn clear_colors() {
	COLORS.try_write().unwrap().clear();
}

pub fn set_color(use_theme: bool, index: u32, [r, g, b]: [u8; 3]) {
	COLORS.try_write().unwrap().insert((use_theme, index), r as u32 | (g as u32) << 8 | (b as u32) << 16 | 0xFF000000);
}

pub fn get_colors() -> BTreeMap<(bool, u32), [u8; 3]> {
	COLORS.read().unwrap().iter().map(|(index, clr)| (
		*index,
		[(clr & 0xFF) as u8, ((clr >> 8) & 0xFF) as u8, ((clr >> 16) & 0xFF) as u8]
	)).collect()
}