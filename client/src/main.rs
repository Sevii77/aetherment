use eframe::egui;

extern crate aetherment;

mod cli;

fn set_notification(_progress: f32, _typ: u8, _msg: &str) {}

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
				set_notification,
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
