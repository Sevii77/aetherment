use eframe::egui;

extern crate aetherment;

mod cli;

fn log(typ: aetherment::LogType, msg: &str) {
	let typ = match typ {
		aetherment::LogType::Log => "LOG",
		aetherment::LogType::Error => "ERROR",
		aetherment::LogType::Fatal => "FATAL",
	};
	
	println!("[{typ}] {msg}");
}

struct CoreWrapper(aetherment::Core);

impl eframe::App for CoreWrapper {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(&ctx, |ui| {
			self.0.draw(ui);
		});
	}
}

fn main() {
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
		let _backend = cc.wgpu_render_state.as_ref().unwrap().adapter.get_info().backend;
		Ok(Box::new(CoreWrapper(aetherment::Core::new(
			cc.egui_ctx.clone(),
			log,
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
		))))
	})).unwrap();
}
