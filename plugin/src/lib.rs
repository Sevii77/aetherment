// #![allow(improper_ctypes_definitions)]

use std::collections::HashMap;

// using str itself doesnt seem to work, no clue why but oh well
#[repr(packed)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
struct FfiStr(*const u8, usize);
impl FfiStr {
	fn new(s: &str) -> Self {
		Self(s.as_ptr(), s.len())
	}
	
	fn to_string(&self) -> String {
		unsafe{std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.0, self.1)).to_string()}
	}
	
	fn to_string_vec(&self) -> Vec<String> {
		self.to_string().split('\0').map(|v| v.to_string()).collect()
	}
}

static mut LOG: fn(u8, FfiStr) = |_, _| {};
fn log(typ: aetherment::LogType, msg: String) {
	let s = msg.as_str();
	unsafe{crate::LOG(typ as _, FfiStr(s.as_ptr(), s.len()))};
	drop(msg);
}

static mut ADDSTYLE: fn(FfiStr) = |_| {};
fn dalamud_add_style(s: &str) {
	unsafe{ADDSTYLE(FfiStr(s.as_ptr(), s.len()))}
}

pub struct State {
	visible: bool,
	core: aetherment::Core,
}

#[repr(packed)]
pub struct Initializers {
	log: fn(u8, FfiStr),
	penumbra_functions: PenumbraFunctions,
	dalamud_add_style: fn(FfiStr),
}

#[repr(packed)]
struct PenumbraGetModSettings {
	exists: bool,
	enabled: bool,
	inherit: bool,
	priority: i32,
	options: FfiStr,
}

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct PenumbraFunctions {
	config_dir: FfiStr,
	redraw: fn(),
	redraw_self: fn(),
	root_path: fn() -> FfiStr,
	mod_list: fn() -> FfiStr,
	add_mod_entry: fn(FfiStr) -> u8,
	reload_mod: fn(FfiStr) -> u8,
	set_mod_enabled: fn(FfiStr, FfiStr, bool) -> u8,
	set_mod_priority: fn(FfiStr, FfiStr, i32) -> u8,
	set_mod_inherit: fn(FfiStr, FfiStr, bool) -> u8,
	set_mod_settings: fn(FfiStr, FfiStr, FfiStr, FfiStr) -> u8,
	get_mod_settings: fn(FfiStr, FfiStr, bool) -> PenumbraGetModSettings,
	current_collection: fn() -> FfiStr,
	get_collections: fn() -> FfiStr,
}

#[no_mangle]
pub extern fn initialize(init: Initializers) -> *mut State {
	use aetherment::modman::backend;
	
	std::panic::set_hook(Box::new(|info| {
		log(aetherment::LogType::Fatal, format!("{}", info));
	}));
	
	match std::panic::catch_unwind(move || {
		unsafe {
			LOG = init.log;
			ADDSTYLE = init.dalamud_add_style;
		};
		
		let funcs = init.penumbra_functions;
		Box::into_raw(Box::new(State {
			visible: aetherment::config().config.plugin_open_on_launch,
			core: aetherment::Core::new(log, backend::BackendInitializers::PenumbraIpc(backend::penumbra_ipc::PenumbraFunctions {
				config_dir: std::path::PathBuf::from(funcs.config_dir.to_string()),
				redraw: Box::new(funcs.redraw),
				redraw_self: Box::new(funcs.redraw_self),
				root_path: Box::new(move || std::path::PathBuf::from((funcs.root_path)().to_string())),
				mod_list: Box::new(move || (funcs.mod_list)().to_string_vec()),
				add_mod_entry: Box::new(move |id| (funcs.add_mod_entry)(FfiStr::new(id))),
				reload_mod: Box::new(move |id| (funcs.reload_mod)(FfiStr::new(id))),
				set_mod_enabled: Box::new(move |collection, id, enabled| (funcs.set_mod_enabled)(FfiStr::new(collection), FfiStr::new(id), enabled)),
				set_mod_priority: Box::new(move |collection, id, priority| (funcs.set_mod_priority)(FfiStr::new(collection), FfiStr::new(id), priority)),
				set_mod_inherit: Box::new(move |collection, id, inherit| (funcs.set_mod_inherit)(FfiStr::new(collection), FfiStr::new(id), inherit)),
				set_mod_settings: Box::new(move |collection, id, option, suboptions| (funcs.set_mod_settings)(FfiStr::new(collection), FfiStr::new(id), FfiStr::new(option), FfiStr::new(&suboptions.join("\0")))),
				get_mod_settings: Box::new(move |collection, id, inherit| {
					let settings = (funcs.get_mod_settings)(FfiStr::new(collection), FfiStr::new(id), inherit);
					backend::penumbra_ipc::GetModSettings {
						exists: settings.exists,
						enabled: settings.enabled,
						inherit: settings.inherit,
						priority: settings.priority,
						options: {
							let options = settings.options.to_string();
							if options.is_empty() {
								HashMap::new()
							} else {
								let options = options.split("\0\0").map(|v| v.to_string()).collect::<Vec<String>>();
								// let sub_options = options.split("\0").map(|v| v.to_string()).collect();
								options.into_iter().map(|v| {
									let mut sub_options = v.split("\0").map(|v| v.to_string());
									(sub_options.next().unwrap(), sub_options.collect())
								}).collect()
							}
						},
					}
				}),
				current_collection: Box::new(move || {
					let collection = (funcs.current_collection)().to_string();
					if collection.is_empty() {return aetherment::modman::backend::Collection{id: String::new(), name: String::new()}}
					
					let mut split = collection.split("\0");
					aetherment::modman::backend::Collection {
						id: split.next().unwrap().to_owned(),
						name: split.next().unwrap().to_owned(),
					}
				}),
				get_collections: Box::new(move || {
					let collections = (funcs.get_collections)().to_string();
					if collections.is_empty() {return Vec::new()};
					
					collections.split("\0\0").map(|v| {
						let mut split = v.split("\0");
						aetherment::modman::backend::Collection {
							id: split.next().unwrap().to_owned(),
							name: split.next().unwrap().to_owned(),
						}
					}).collect()
				}),
				
				// default_collection: Box::new(move || (funcs.default_collection)().to_string()),
				// get_collections: Box::new(move || (funcs.get_collections)().to_string_vec()),
			}), aetherment::modman::meta::OptionalInitializers {
				dalamud: Some(dalamud_add_style)
			}),
		}))
	}) {
		Ok(v) => v,
		Err(_) => 0 as *mut _,
	}
}

#[no_mangle]
pub extern fn destroy(state: *mut State) {
	_ = unsafe{Box::from_raw(state)};
}

#[no_mangle]
pub extern fn command(state: *mut State, args: &str) {
	let state = unsafe{&mut *state};
	// log(aetherment::LogType::Log, format!("{}", args));
	match args {
		_ => state.visible = !state.visible,
	}
}

#[no_mangle]
pub extern fn draw(state: *mut State) {
	let state = unsafe{&mut *state};
	
	let ui = &mut aetherment::renderer::Ui::new();
	if state.visible {
		ui.window(aetherment::renderer::WindowArgs {
			title: "Aetherment",
			open: Some(&mut state.visible),
			..Default::default()
		}, |ui| state.core.draw(ui));
	}
}