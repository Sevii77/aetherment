pub mod external {
	pub mod bytes;
	pub use bytes::Bytes;
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
}

pub mod game {
	pub mod tex;
	pub use tex::Tex;
	pub mod mtrl;
	pub use mtrl::Mtrl;
	pub mod uld;
	pub use uld::Uld;
	pub mod exd;
	pub use exd::Exd;
	pub mod exh;
	pub use exh::Exh;
	
	// pub mod mdl;
	// pub use mdl::Mdl;
}