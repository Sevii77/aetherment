#![allow(improper_ctypes_definitions)]

mod ffi {
	pub mod str;
}
mod renderer;

use std::collections::HashMap;
use ffi::str::{FfiString, FfiStr};

static mut LOG: fn(u8, FfiStr) = |_, _| {};
fn log(typ: aetherment::LogType, msg: &str) {
	unsafe{crate::LOG(typ as _, FfiStr::new(msg))};
}

static mut ADDSTYLE: fn(FfiStr) = |_| {};
fn dalamud_add_style(s: &str) {
	unsafe{ADDSTYLE(FfiStr::new(s))}
}

// ------------------------------

#[repr(C, packed)]
pub struct Initializers {
	ffi_str_drop: fn(*const u8, usize),
	log: fn(u8, FfiStr),
	issue_functions: IssueFunctions,
	penumbra_functions: PenumbraFunctions,
	services_functions: ServicesFunctions,
	d3d11_device: usize,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct IssueFunctions {
	ui_resolution: fn() -> u8,
	ui_theme: fn() -> u8,
}

#[repr(C, packed)]
struct PenumbraGetModSettings {
	exists: bool,
	enabled: bool,
	inherit: bool,
	priority: i32,
	options: FfiString,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PenumbraFunctions {
	// config_dir: FfiStr,
	redraw: fn(),
	redraw_self: fn(),
	is_enabled: fn() -> bool,
	root_path: fn() -> FfiString,
	mod_list: fn() -> FfiString,
	add_mod_entry: fn(FfiStr) -> u8,
	reload_mod: fn(FfiStr) -> u8,
	set_mod_enabled: fn(FfiStr, FfiStr, bool) -> u8,
	set_mod_priority: fn(FfiStr, FfiStr, i32) -> u8,
	set_mod_inherit: fn(FfiStr, FfiStr, bool) -> u8,
	set_mod_settings: fn(FfiStr, FfiStr, FfiStr, FfiStr) -> u8,
	get_mod_settings: fn(FfiStr, FfiStr, bool) -> PenumbraGetModSettings,
	get_collection: fn(u8) -> FfiString,
	get_collections: fn() -> FfiString,
}

#[allow(dead_code)]
#[repr(C, packed)]
struct UiColorsColor {
	use_theme: bool,
	index: u32,
	clr: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ServicesFunctions {
	set_ui_colors: fn(*const UiColorsColor, usize),
	dalamud_add_style: fn(FfiStr),
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Io {
	pub width: f32,
	pub height: f32,
	pub mouse_x: f32,
	pub mouse_y: f32,
	pub wheel_x: f32,
	pub wheel_y: f32,
	pub mods: u32,
	pub input_buf_ptr: usize,
	pub input_buf_len: usize,
	
	pub set_keyboard_focus: usize,
}

// ------------------------------\

pub struct State {
	renderer: renderer::Renderer,
	core: aetherment::Core,
}

#[no_mangle]
pub extern "C" fn initialize(init: Initializers) -> *mut State {
	use aetherment::modman::backend;
	
	std::panic::set_hook(Box::new(|info| {
		log(aetherment::LogType::Fatal, &format!("{}", info));
	}));
	
	match std::panic::catch_unwind(move || {
		let funcs = init.penumbra_functions;
		let requirement_funcs = init.issue_functions;
		let services_funcs = init.services_functions;
		
		unsafe {
			ffi::str::DROP = init.ffi_str_drop;
			LOG = init.log;
			ADDSTYLE = services_funcs.dalamud_add_style;
		};
		
		let get_collection = Box::new(move |collection_type| {
			let v = (funcs.get_collection)(collection_type as _).to_string();
			if !v.contains('\0') {
				aetherment::modman::backend::Collection {
					id: "00000000-0000-0000-0000-000000000000".to_string(),
					name: "None".to_string(),
				}
			} else {
				let mut split = v.split("\0");
				aetherment::modman::backend::Collection {
					id: split.next().unwrap().to_owned(),
					name: split.next().unwrap().to_owned(),
				}
			}
		});
		
		let state = Box::into_raw(Box::new(State {
			// visible: aetherment::config().config.plugin_open_on_launch,
			renderer: renderer::Renderer::new(init.d3d11_device).unwrap(),
			core: aetherment::Core::new(log, backend::BackendInitializers::PenumbraIpc(backend::penumbra_ipc::PenumbraFunctions {
				// config_dir: std::path::PathBuf::from(funcs.config_dir.to_string()),
				redraw: Box::new(funcs.redraw),
				redraw_self: Box::new(funcs.redraw_self),
				is_enabled: Box::new(funcs.is_enabled),
				root_path: Box::new(move || std::path::PathBuf::from((funcs.root_path)().to_string())),
				mod_list: Box::new(move || (funcs.mod_list)().to_string().split('\0').map(|v| v.to_string()).collect()),
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
				get_collection: get_collection.clone(),
				get_collections: Box::new(move || {
					let collections = (funcs.get_collections)().to_string();
					if collections.is_empty() {return Vec::new()};
					
					collections.split("\0\0").filter_map(|v| {
						let mut split = v.split("\0");
						Some(aetherment::modman::backend::Collection {
							id: split.next()?.to_owned(),
							name: split.next()?.to_owned(),
						})
					}).collect()
				}),
				
				// default_collection: Box::new(move || (funcs.default_collection)().to_string()),
				// get_collections: Box::new(move || (funcs.get_collections)().to_string_vec()),
			}), aetherment::modman::requirement::RequirementInitializers {
				ui_resolution: Box::new(requirement_funcs.ui_resolution),
				ui_theme: Box::new(requirement_funcs.ui_theme),
				collection: get_collection,
			}, aetherment::modman::meta::OptionalInitializers {
				dalamud: Some(dalamud_add_style)
			}, aetherment::service::ServicesInitializers {
				uicolor: Box::new(move |colors| {
					let colors = colors.iter().map(|((t, i), c)| UiColorsColor{use_theme: *t, index: *i, clr: *c}).collect::<Vec<_>>();
					(services_funcs.set_ui_colors)(colors.as_ptr(), colors.len());
				})
			}),
		}));
		
		// unsafe{aetherment::service::initialize()};
		
		state
	}) {
		Ok(v) => v,
		Err(_) => 0 as *mut _,
	}
}

#[no_mangle]
pub extern "C" fn destroy(state: *mut State) {
	aetherment::service::disable();
	_ = unsafe{Box::from_raw(state)};
}

#[no_mangle]
pub extern "C" fn command(state: *mut State, _args: FfiString) -> bool {
	let _state = unsafe{&mut *state};
	
	// match args {
	// 	_ => state.visible = !state.visible,
	// }
	
	false
}

#[no_mangle]
pub extern "C" fn draw(state: *mut State, d3d11_device: usize, io: Io) -> usize {
	let state = unsafe{&mut *state};
	
	// let draw = |ui| {
	// 	state.core.draw(ui)
	// }
	
	match state.renderer.draw(d3d11_device, io, |ctx| {
		egui::CentralPanel::default().frame(egui::Frame {
			inner_margin: egui::Margin::same(0.0),
			outer_margin: egui::Margin::same(0.0),
			shadow: egui::epaint::Shadow::NONE,
			fill: egui::Color32::TRANSPARENT,
			stroke: egui::Stroke::NONE,
			rounding: egui::Rounding::ZERO,
		}).show(ctx, |ui| {state.core.draw(ui)});
	}) {
		Ok(v) => v,
		Err(err) => {
			log(aetherment::LogType::Error, &format!("Failed drawing: {err:#?}"));
			0 as _
		}
	}
}

#[no_mangle]
pub extern "C" fn backend_penumbraipc_modchanged(typ: u8, collection_id: FfiString, mod_id: FfiString) {
	aetherment::modman::backend::penumbra_ipc::subscriber_modchanged(typ, &collection_id.to_string(), &mod_id.to_string());
}