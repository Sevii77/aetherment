use std::{sync::Arc, task::Poll};
use egui::{ahash::HashMap, load::BytesPoll, mutex::Mutex};

const PROTOCOL: &str = "aetherment://";

type Entry = Poll<Result<Arc<[u8]>, String>>;

#[derive(Default)]
pub struct AssetLoader {
	cache: Arc<Mutex<HashMap<String, Entry>>>,
}

impl egui::load::BytesLoader for AssetLoader {
	fn id(&self) -> &str {
		// egui::generate_loader_id!(AethermentLoader)
		"aetherment::AssetLoader"
	}
	
	fn load(&self, ctx: &egui::Context, uri: &str) -> egui::load::BytesLoadResult {
		let Some(uri) = uri.strip_prefix(PROTOCOL) else {return Err(egui::load::LoadError::NotSupported)};
		
		let mut cache = self.cache.lock();
		if let Some(entry) = cache.get(uri).cloned() {
			match entry {
				Poll::Ready(Ok(data)) => Ok(BytesPoll::Ready {
					size: None,
					bytes: egui::load::Bytes::Shared(data),
					mime: None,
				}),
				Poll::Ready(Err(err)) => Err(egui::load::LoadError::Loading(err)),
				Poll::Pending => Ok(BytesPoll::Pending{size: None}),
			}
		} else {
			let Some(p) = uri.find('/') else {
				cache.insert(uri.to_string(), Poll::Ready(Err("Invalid uri".to_string())));
				return Err(egui::load::LoadError::Loading("Invalid uri".to_string()));
			};
			
			cache.insert(uri.to_string(), Poll::Pending);
			drop(cache);
			
			{
				let uri = uri.to_string();
				let ctx = ctx.clone();
				let cache = self.cache.clone();
				std::thread::spawn(move || {
					let modid = &uri[..p];
					let path = &uri[p + 1..];
					let v = Poll::Ready(crate::backend().get_mod_asset(modid, path).map(|v| v.into()).map_err(|v| v.to_string()));
					cache.lock().insert(uri, v);
					ctx.request_repaint();
				});
			}
			
			return Ok(BytesPoll::Pending{size: None});
		}
	}
	
	fn forget(&self, uri: &str) {
		_ = self.cache.lock().remove(uri);
	}
	
	fn forget_all(&self) {
		self.cache.lock().clear();
	}
	
	fn byte_size(&self) -> usize {
		self.cache.lock().values().map(|v| match v {
			Poll::Ready(Ok(v)) => v.len(),
			Poll::Ready(Err(v)) => v.len(),
			Poll::Pending => 0,
		}).sum()
	}
}