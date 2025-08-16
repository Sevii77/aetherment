use std::{sync::Arc, task::Poll};
use egui::{ahash::HashMap, load::BytesPoll, mutex::Mutex};

const PROTOCOLS: &[&str] = &["http://", "https://"];

type Entry = Poll<Result<Arc<[u8]>, String>>;

#[derive(Default)]
pub struct HttpLoader {
	cache: Arc<Mutex<HashMap<String, Entry>>>,
}

impl HttpLoader {
	pub fn clear_cache() {
		let Some(cache_dir) = dirs::cache_dir() else {return};
		let Ok(read_dir) = std::fs::read_dir(cache_dir.join("Aetherment").join("images")) else {return};
		for entry in read_dir {
			let Ok(entry) = entry else {continue};
			_ = std::fs::remove_file(entry.path());
		}
	}
}

impl egui::load::BytesLoader for HttpLoader {
	fn id(&self) -> &str {
		"aetherment::HttpLoader"
	}
	
	fn load(&self, ctx: &egui::Context, uri: &str) -> egui::load::BytesLoadResult {
		if !PROTOCOLS.iter().any(|v| uri.starts_with(v)) {
			return Err(egui::load::LoadError::NotSupported)
		};
		
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
			cache.insert(uri.to_string(), Poll::Pending);
			drop(cache);
			
			let uri = uri.to_string();
			let ctx = ctx.clone();
			let cache = self.cache.clone();
			std::thread::spawn(move || {
				let hash = crate::hash_str(blake3::hash(uri.as_bytes()));
				
				'c: {
					let Some(cache_dir) = dirs::cache_dir() else {break 'c};
					let path = cache_dir.join("Aetherment").join("images").join(&hash);
					if !path.exists() {break 'c};
					let Ok(data) = std::fs::read(path) else {break 'c};
					cache.lock().insert(uri, Poll::Ready(Ok(data.into())));
					ctx.request_repaint();
					return;
				}
				
				let resp = match ureq::get(&uri).call() {
					Ok(v) => v,
					Err(e) => {
						cache.lock().insert(uri, Poll::Ready(Err(e.to_string())));
						ctx.request_repaint();
						return;
					},
				};
				
				let data = match resp.into_body().read_to_vec() {
					Ok(v) => v,
					Err(e) => {
						cache.lock().insert(uri, Poll::Ready(Err(e.to_string())));
						ctx.request_repaint();
						return;
					},
				};
				
				if let Some(cache_dir) = dirs::cache_dir() {
					let cache_dir = cache_dir.join("Aetherment").join("images");
					_ = std::fs::create_dir_all(&cache_dir);
					_ = std::fs::write(cache_dir.join(&hash), &data);
				}
				
				cache.lock().insert(uri, Poll::Ready(Ok(data.into())));
				ctx.request_repaint();
			});
			
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