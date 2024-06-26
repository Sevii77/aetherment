use eframe::egui;

extern crate aetherment;

mod cli;

fn log(typ: aetherment::LogType, msg: String) {
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
			self.0.draw(&mut aetherment::renderer::Ui::new(ui));
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
		Box::new(CoreWrapper(aetherment::Core::new(log, aetherment::modman::backend::BackendInitializers::None)))
	})).unwrap();
}