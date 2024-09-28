use std::sync::LazyLock;

pub(crate) mod uicolor;

extern "system" {
	fn GetModuleHandleW(name: usize) -> usize;
}

static MODULE: LazyLock<(usize, usize)> = LazyLock::new(|| {
	let exe_path = std::env::current_exe().unwrap().to_string_lossy().to_string();
	// let exe_data = std::fs::read(&exe_path).unwrap();
	// not the actual size but idc anymore, scanning the exe fucks shit up, im too dumb for this and dont want to use GetModuleInformation atm
	let module_len = std::fs::read(&exe_path).unwrap().len();
	let base_address = unsafe{GetModuleHandleW(0)};
	
	(base_address, module_len)
});

// TODO: make this better by a lot
pub(crate) fn scan(sig: &str) -> Option<usize> {
	let (ptr, len) = *MODULE;
	let segs = sig.split(' ').map(|v| if v == "??" {None} else {u8::from_str_radix(v, 16).ok()}).collect::<Vec<_>>();
	'a: for o in 0..=len - segs.len() {
		for i in 0..segs.len() {
			if segs[i] != None && segs[i] != Some(unsafe{*((ptr + o + i) as *const u8)}) {
				continue 'a;
			}
		}
		
		return Some(ptr + o);
	}
	
	None
}

pub unsafe fn initialize() {
	log!("init services");
	uicolor::initialize();
}

pub unsafe fn disable() {
	log!("disable services");
	uicolor::disable();
}