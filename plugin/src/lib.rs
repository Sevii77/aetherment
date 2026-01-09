#![allow(improper_ctypes_definitions)]

mod ffi {
	pub mod str;
}
mod renderer;
mod penumbradraw;

use std::collections::HashMap;
use ffi::str::{FfiString, FfiStr};

static mut ADDSTYLE: fn(FfiStr) = |_| {};
fn dalamud_add_style(s: &str) {
	unsafe{ADDSTYLE(FfiStr::new(s))}
}

static mut NOTIFICATION: fn(f32, u8, FfiStr) = |_, _, _| {};
fn set_notification(progress: f32, typ: u8, msg: &str) {
	unsafe{NOTIFICATION(progress, typ, FfiStr::new(msg))}
}

// ------------------------------

#[repr(C, packed)]
pub struct Initializers {
	ffi_str_drop: fn(*const u8, usize),
	log: fn(u8, FfiStr),
	set_notification: fn(f32, u8, FfiStr),
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
	pub ui_scale: f32,
	pub set_keyboard_focus: usize,
}

// ------------------------------

struct DalamudLogger(fn(u8, FfiStr));

impl log::Log for DalamudLogger {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		match metadata.target() {
			"aetherment" |
			"renderer" => true,
			_ => false,
		}
	}
	
	fn log(&self, record: &log::Record) {
		if !self.enabled(record.metadata()) {return}
		
		let level = match record.level() {
			log::Level::Error => 255,
			log::Level::Warn => 1,
			log::Level::Info => 2,
			log::Level::Debug => 3,
			log::Level::Trace => 4,
		};
		
		// let msg = format!("[{}] {}", record.metadata().target(), record.args());
		let msg = record.args().to_string();
		
		(self.0)(level, FfiStr::new(&msg));
	}
	
	fn flush(&self) {
		
	}
}

// ------------------------------

pub struct State {
	renderer: renderer::Renderer,
	renderer_3d: ::renderer::Renderer,
	core: aetherment::Core,
	
	penumbradraw: penumbradraw::PenumbraDraw,
	ui_scale: f32,
}

#[no_mangle]
pub extern "C" fn initialize(init: Initializers) -> *mut State {
	use aetherment::modman::backend;
	
	std::panic::set_hook(Box::new(|info| {
		// stolen from the log crate
		let thread = std::thread::current();
		let thread = thread.name().unwrap_or("<unnamed>");
		
		let msg = match info.payload().downcast_ref::<&'static str>() {
			Some(s) => *s,
			None => match info.payload().downcast_ref::<String>() {
				Some(s) => &s[..],
				None => "Box<Any>",
			}
		};
		
		match info.location() {
			Some(location) => {
				log::error!(target: "aetherment", "thread '{}' panicked at '{}': {}:{}",
					thread,
					msg,
					location.file(),
					location.line())
			}
			None => log::error!(target: "aetherment", "thread '{}' panicked at '{}'", thread, msg),
		}
	}));
	
	_ = log::set_boxed_logger(Box::new(DalamudLogger(init.log)));
	log::set_max_level(log::LevelFilter::Debug);
	
	match std::panic::catch_unwind(move || {
		let funcs = init.penumbra_functions;
		let requirement_funcs = init.issue_functions;
		let services_funcs = init.services_functions;
		
		unsafe {
			ffi::str::DROP = init.ffi_str_drop;
			ADDSTYLE = services_funcs.dalamud_add_style;
			NOTIFICATION = init.set_notification;
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
		
		let renderer_egui = renderer::Renderer::new(init.d3d11_device).unwrap();
		let renderer_3d: Box<dyn ::renderer::renderer::RendererInner> = Box::new(::renderer::renderer::D3d11Renderer::new(init.d3d11_device, Box::new(|texture| {
			let texture = texture.as_any().downcast_ref::<::renderer::renderer::D3d11Texture>().unwrap();
			texture.get_view_ptr() as u64
		})));
		
		let core = aetherment::Core::new(
			renderer_egui.egui_ctx(),
			set_notification,
			backend::BackendInitializers::PenumbraIpc(backend::penumbra_ipc::PenumbraFunctions {
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
			}),
			aetherment::modman::requirement::RequirementInitializers {
				ui_resolution: Box::new(requirement_funcs.ui_resolution),
				ui_theme: Box::new(requirement_funcs.ui_theme),
				collection: get_collection,
			},
			aetherment::modman::meta::OptionalInitializers {
				dalamud: Some(dalamud_add_style)
			},
			aetherment::service::ServicesInitializers {
				uicolor: Box::new(move |colors| {
					let colors = colors.iter().map(|((t, i), c)| UiColorsColor{use_theme: *t, index: *i, clr: *c}).collect::<Vec<_>>();
					(services_funcs.set_ui_colors)(colors.as_ptr(), colors.len());
				})
			}
		);
		
		let state = Box::into_raw(Box::new(State {
			penumbradraw: penumbradraw::PenumbraDraw::new(core.mod_manager.clone()),
			core,
			renderer: renderer_egui,
			renderer_3d: renderer_3d,
			ui_scale: 1.0,
		}));
		
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
	
	false
}

#[no_mangle]
pub extern "C" fn draw(state: *mut State, d3d11_device: usize, io: Io) -> usize {
	let state = unsafe{&mut *state};
	
	let r = state.renderer_3d.as_any_mut().downcast_mut::<::renderer::renderer::D3d11Renderer>().unwrap();
	r.update_device(d3d11_device);
	
	state.ui_scale = io.ui_scale;
	
	match state.renderer.draw(d3d11_device, io, |ctx| {
		egui::CentralPanel::default().frame(egui::Frame {
			inner_margin: egui::Margin::same(0),
			outer_margin: egui::Margin::same(0),
			shadow: egui::epaint::Shadow::NONE,
			fill: egui::Color32::TRANSPARENT,
			stroke: egui::Stroke::NONE,
			corner_radius: egui::CornerRadius::ZERO,
		}).show(ctx, |ui| {
			state.core.draw(ui, &state.renderer_3d)
		});
	}) {
		Ok(v) => v,
		Err(err) => {
			log::warn!(target: "aetherment", "Failed drawing: {err:#?}");
			0 as _
		}
	}
}

#[no_mangle]
pub extern "C" fn tick(state: *mut State) {
	let state = unsafe{&mut *state};
	state.core.tick();
}

#[no_mangle]
pub extern "C" fn config_plugin_open_on_launch(_state: *mut State) -> u8 {
	aetherment::config().config.plugin_open_on_launch as u8
}

#[no_mangle]
pub extern "C" fn backend_penumbraipc_modchanged(typ: u8, collection_id: FfiString, mod_id: FfiString) {
	aetherment::modman::backend::penumbra_ipc::subscriber_modchanged(typ, &collection_id.to_string(), &mod_id.to_string());
}

#[no_mangle]
pub extern "C" fn backend_penumbraipc_drawsettings(state: *mut State, mod_id: FfiString) -> u8 {
	let state = unsafe{&mut *state};
	state.penumbradraw.settings(state.ui_scale, mod_id.as_str()) as u8
}