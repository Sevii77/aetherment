use eframe::egui;

extern crate aetherment;

mod cli;

struct SimpleLogger;

impl log::Log for SimpleLogger {
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
			log::Level::Error => "FATAL",
			log::Level::Warn => "ERROR",
			log::Level::Info => "LOG",
			log::Level::Debug => "LOG",
			log::Level::Trace => "LOG",
		};
		
		let msg = record.args().to_string();
		
		println!("[{level}] {msg}");
	}
	
	fn flush(&self) {
		
	}
}

struct CoreWrapper {
	core: aetherment::Core,
	renderer: renderer::Renderer,
}

impl eframe::App for CoreWrapper {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(&ctx, |ui| {
			self.core.draw(ui, &self.renderer);
		});
	}
}

fn main() {
	_ = log::set_boxed_logger(Box::new(SimpleLogger));
	log::set_max_level(log::LevelFilter::Debug);
	
	// // let path = "chara/equipment/e6100/model/c0201e6100_top.mdl";
	// let path = "chara/human/c1401/obj/face/f0001/model/c1401f0001_fac.mdl";
	// let mdl = aetherment::noumenon().unwrap().file::<aetherment::noumenon_::format::game::Mdl>(path).unwrap();
	// 
	// fn file_reader(path: &str) -> Option<Vec<u8>> {
	// 	aetherment::noumenon().unwrap().file::<Vec<u8>>(path).ok()
	// }
	// let materials = mdl.bake_materials(file_reader);
	// 
	// let skeletons = aetherment::noumenon_::format::game::Mdl::skeleton_paths(path)
	// 	.into_iter()
	// 	.flat_map(|v| {
	// 		let sklb = aetherment::noumenon().unwrap().file::<aetherment::noumenon_::format::game::Sklb>(&v).unwrap();
	// 		sklb.bones
	// 			.iter()
	// 			.map(|bone| aetherment::noumenon_::format::external::gltf::Bone {
	// 				name: bone.name.clone(),
	// 				parent: if bone.parent >= 0 {Some(sklb.bones[bone.parent as usize].name.clone())} else {None},
	// 				translation: bone.translation,
	// 				rotation: bone.rotation,
	// 				scale: bone.scale,
	// 			}).collect::<Vec<_>>()
	// 	}).collect::<Vec<_>>();
	// 
	// let mut file = std::fs::File::create("./test.glb").unwrap();
	// <aetherment::noumenon_::format::game::Mdl as aetherment::noumenon_::format::external::Gltf<aetherment::noumenon_::format::game::mdl::Error>>::write(&mdl, &mut file, materials, skeletons).unwrap();
	// 
	// return;
	
	if std::env::args().len() > 1 {
		cli::handle_cli().unwrap();
		
		return;
	}
	
	let options = eframe::NativeOptions {
		// initial_window_size: Some(egui::Vec2::new(1280.0, 720.0)),
		wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
			// supported_backends: backends,
			// supported_backends: aetherment::Backends::all(),
			..Default::default()
		},
		..Default::default()
	};
	
	eframe::run_native("Aetherment", options, Box::new(|cc| {
		let rs = cc.wgpu_render_state.as_ref().unwrap();
		let rs_device = rs.device.clone();
		let rs_renderer = rs.renderer.clone();
		let renderer = Box::new(renderer::renderer::WgpuRenderer::new(
			rs.device.clone(),
			rs.queue.clone(),
			Box::new(move |texture| {
				let texture = texture.as_any().downcast_ref::<renderer::renderer::WgpuTexture>().unwrap();
				match rs_renderer.write().register_native_texture(&rs_device, texture.get_view(), eframe::wgpu::FilterMode::Linear) {
					egui::TextureId::Managed(v) | egui::TextureId::User(v) => v
				}
			})));
		
		Ok(Box::new(CoreWrapper {
			core: aetherment::Core::new(
				cc.egui_ctx.clone(),
				aetherment::modman::backend::BackendInitializers::None,
				aetherment::modman::requirement::RequirementInitializers {
					ui_resolution: Box::new(|| 255),
					ui_theme: Box::new(|| 255),
					collection: Box::new(|_| aetherment::modman::backend::Collection{name: "None".to_string(), id: "00000000-0000-0000-0000-000000000000".to_string()}),
				},
				Default::default(),
				aetherment::service::ServicesInitializers {
					uicolor: Box::new(|_| {}),
				},
			),
			renderer,
		}))
	})).unwrap();
}
