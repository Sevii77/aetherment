fn main() {
	println!("cargo:rustc-link-search=./plugin/lib");
	println!("cargo:rustc-link-lib=cimgui");
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=./lib/cimgui.lib");
	
	generate_bindings();
}

fn generate_bindings() {
	bindgen::Builder::default()
		.header("./lib/cimgui.h")
		.raw_line("#![allow(unused)]")
		.raw_line("#![allow(non_snake_case)]")
		.raw_line("#![allow(non_camel_case_types)]")
		.raw_line("#![allow(non_upper_case_globals)]")
		.raw_line("#![allow(unnecessary_transmutes)]")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.clang_arg("--language=c++")
		.clang_arg("-DCIMGUI_DEFINE_ENUMS_AND_STRUCTS")
		.layout_tests(false)
		// .default_enum_style(bindgen::EnumVariation::Rust{non_exhaustive: false})
		.prepend_enum_name(false)
		.generate()
		.unwrap()
		// .to_string()
		.write_to_file(std::path::Path::new("./src/penumbradraw/imgui_bindings.rs"))
		.unwrap();
}