#![allow(dead_code)]

#[cfg(not(feature = "imgui"))]
fn main() {
	
}

#[cfg(feature = "imgui")]
fn main() {
	println!("cargo:rustc-link-search=./renderer/lib");
	println!("cargo:rustc-link-lib=cimgui");
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=./lib/cimgui.lib");
	
	// generate_bindings();
}

#[cfg(feature = "imgui")]
fn generate_bindings() {
	bindgen::Builder::default()
		.header("./lib/cimgui.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.clang_arg("--language=c++")
		.clang_arg("-DCIMGUI_DEFINE_ENUMS_AND_STRUCTS")
		.layout_tests(false)
		// .default_enum_style(bindgen::EnumVariation::Rust{non_exhaustive: false})
		.prepend_enum_name(false)
		.generate()
		.unwrap()
		// .to_string()
		.write_to_file(std::path::Path::new("./src/ui/imgui/bindings.rs"))
		.unwrap();
}