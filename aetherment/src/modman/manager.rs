use std::{collections::HashSet, sync::Arc};
use dashmap::DashMap;

#[derive(Clone)]
pub struct Manager {
	pub collections: Arc<DashMap<String, String>>,
	pub mods: Arc<Vec<Arc<str>>>,
	pub metas: Arc<DashMap<Arc<str>, Arc<super::meta::Meta>>>,
	pub settings: Arc<DashMap<Arc<str>, super::settings::Settings>>,
	pub settings_remote: Arc<DashMap<Arc<str>, crate::remote::settings::Settings>>,
	pub aeth_mods: Arc<HashSet<Arc<str>>>,
}

impl Manager {
	pub fn new() -> Self {
		let mut s = Self {
			collections: Arc::new(DashMap::new()),
			mods: Arc::new(Vec::new()),
			metas: Arc::new(DashMap::new()),
			settings: Arc::new(DashMap::new()),
			settings_remote: Arc::new(DashMap::new()),
			aeth_mods: Arc::new(HashSet::new()),
		};
		
		s.reload();
		
		s
	}
	
	pub fn reload(&mut self) {
		let backend = crate::backend();
		backend.load_mods();
		
		self.collections = Arc::new(backend.get_collections().into_iter().map(|v| (v.id, v.name)).collect());
		
		let mut mods = backend.get_mods();
		mods.sort_unstable();
		self.mods = Arc::new(mods);
		
		self.metas = Arc::new(self.mods.iter().map(|m| (m.clone(), backend.get_mod_meta(m).unwrap())).collect());
		self.settings = Arc::new(self.mods.iter().map(|m| (m.clone(), crate::modman::settings::Settings::open(&backend.get_mod_meta(m).unwrap(), m))).collect());
		self.settings_remote = Arc::new(self.mods.iter().map(|m| (m.clone(), crate::remote::settings::Settings::open(m))).collect());
		self.aeth_mods = Arc::new(self.mods.iter().filter(|m| backend.is_mod_aeth(m)).map(|v| v.clone()).collect());
		
		let config = crate::config();
		if !self.collections.contains_key(&config.config.active_collection) && self.collections.len() > 0 {
			config.config.active_collection = self.collections.iter().next().unwrap().key().to_owned();
			_ = config.save_forced();
		}
	}
}