pub mod external {
	pub mod dds;
	pub use dds::Dds;
	pub mod png;
	pub use png::Png;
	pub mod tga;
	pub use tga::Tga;
	pub mod tiff;
	pub use tiff::Tiff;
	
	// pub mod fbx;
	// pub use fbx::Fbx;
	
	// pub mod json;
	// pub use json::Json;
}

pub mod game {
	pub type Result<T, E = ironworks::Error> = std::result::Result<T, E>;
	
	// pub mod mdl;
	// pub use mdl::Mdl;
	pub mod tex;
	pub use tex::Tex;
	pub mod mtrl;
	pub use mtrl::Mtrl;
	pub mod uld;
	pub use uld::Uld;
}