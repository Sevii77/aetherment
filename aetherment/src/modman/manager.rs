use std::{sync::{Arc, RwLock}, time::Instant};
use dashmap::{DashMap, DashSet};

#[derive(Clone)]
pub struct Manager {
	pub collections: Arc<DashMap<String, String>>,
	pub metas: Arc<DashMap<Arc<str>, Arc<super::meta::Meta>>>,
	pub settings: Arc<DashMap<Arc<str>, super::settings::Settings>>,
	pub settings_remote: Arc<DashMap<Arc<str>, crate::remote::settings::Settings>>,
	pub aeth_mods: Arc<DashSet<Arc<str>>>,
	
	last_viewed: Arc<RwLock<Instant>>,
	last_interacted: Arc<RwLock<Instant>>,
}

impl Manager {
	pub fn new() -> Self {
		let s = Self {
			collections: Arc::new(DashMap::new()),
			metas: Arc::new(DashMap::new()),
			settings: Arc::new(DashMap::new()),
			settings_remote: Arc::new(DashMap::new()),
			aeth_mods: Arc::new(DashSet::new()),
			
			last_viewed: Arc::new(RwLock::new(Instant::now())),
			last_interacted: Arc::new(RwLock::new(Instant::now())),
		};
		
		s.reload();
		
		s
	}
	
	pub fn reload(&self) {
		let backend = crate::backend();
		backend.load_mods();
		
		self.collections.clear();
		for v in backend.get_collections() {
			self.collections.insert(v.id, v.name);
		}
		
		let mut mods = backend.get_mods();
		mods.sort_unstable();
		
		self.metas.clear();
		for m in &mods {
			self.metas.insert(m.clone(), backend.get_mod_meta(m).unwrap());
		}
		
		self.settings.clear();
		for m in &mods {
			self.settings.insert(m.clone(), crate::modman::settings::Settings::open(&backend.get_mod_meta(m).unwrap(), m));
		}
		
		self.settings_remote.clear();
		for m in &mods {
			self.settings_remote.insert(m.clone(), crate::remote::settings::Settings::open(m));
		}
		
		self.aeth_mods.clear();
		for m in &mods {
			if backend.is_mod_aeth(m) {
				self.aeth_mods.insert(m.clone());
			}
		}
		
		let config = crate::config();
		if !self.collections.contains_key(&config.config.active_collection) && self.collections.len() > 0 {
			config.config.active_collection = self.collections.iter().next().unwrap().key().to_owned();
			_ = config.save_forced();
		}
	}
	
	pub fn update_last_viewed(&self) {
		*self.last_viewed.write().unwrap() = Instant::now();
	}
	
	pub fn update_last_interacted(&self) {
		*self.last_interacted.write().unwrap() = Instant::now();
	}
	
	pub fn should_auto_update(&self) -> bool {
		let config = &crate::config().config;
		let now = Instant::now();
		if now.duration_since(*self.last_viewed.read().unwrap()) > config.auto_apply_last_viewed {return true}
		if now.duration_since(*self.last_interacted.read().unwrap()) > config.auto_apply_last_interacted {return true}
		false
	}
}