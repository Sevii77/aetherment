use std::{fs::File, io::{BufReader, BufWriter, Cursor, Read, Write}, path::{Path, PathBuf}};

use clap::{value_parser, Arg, ArgAction, Command};

pub fn handle_cli() -> Result<(), Box<dyn std::error::Error>> {
	let matches = Command::new("aetherment")
		.about("Final Fantasy XIV modding tool")
		.version("1.0")
		.subcommand_required(true)
		
		.subcommand(Command::new("extract")
			.short_flag('e')
			.about("Extract files from the game")
			.arg(Arg::new("path")
				.help("The game path to the file(s)")
				.required(true)
				.action(ArgAction::Set)
				.num_args(1))
			.arg(Arg::new("out")
				.long("out")
				.help("The path of the output")
				// .required(true)
				.action(ArgAction::Set)
				.num_args(1))
			.arg(Arg::new("outformat")
				.long("outformat")
				.help("Format of the output file, required if output is stdout")
				.required_if_eq("out", "-")
				.action(ArgAction::Set)
				.num_args(1)))
		
		.subcommand(Command::new("convert")
			.short_flag('c')
			.about("Convert files between formats")
			.arg(Arg::new("informat")
				.long("informat")
				.help("Format of the input file, required if input is stdin or if input is a directory")
				.required_if_eq("in", "-")
				.action(ArgAction::Set)
				.num_args(1))
			.arg(Arg::new("outformat")
				.long("outformat")
				.help("Format of the output file, required if output is stdout")
				.required_if_eq("out", "-")
				.action(ArgAction::Set)
				.num_args(1))
			.arg(Arg::new("in")
				.help("The path of the file to convert")
				.required(true)
				.action(ArgAction::Set)
				.num_args(1))
			.arg(Arg::new("out")
				.help("The path of the output")
				// .required(true)
				.action(ArgAction::Set)
				.num_args(1)))
		
		.subcommand(Command::new("pack")
			.about("Create a modpack from the specified mod directory")
			.arg(Arg::new("path")
				.help("The directory of the mod")
				.required(true)
				.value_parser(value_parser!(PathBuf))
				.action(ArgAction::Set)
				.num_args(1)))
		
		.subcommand(Command::new("diff")
			.about("Check which game files changed between mod creation and now")
			.arg(Arg::new("path")
				.help("Path of the mod file")
				.required(true)
				.value_parser(value_parser!(PathBuf))
				.action(ArgAction::Set)
				.num_args(1)))
		
		// .subcommand(Command::new("game-directory")
		// 	.about("Sets the game directory used by commands such as extract")
		// 	.arg(Arg::new("path")
		// 		.help("The directory of the game, this directory contains 'boot' and 'game'")
		// 		.required(true)
		// 		.value_parser(value_parser!(PathBuf))
		// 		.action(ArgAction::Set)
		// 		.num_args(1)))
		
		.get_matches();
	
	match matches.subcommand() {
		Some(("extract", sub)) => {
			let Some(noumenon) = aetherment::noumenon_instance() else {
				println!("Game install path set in config is invalid");
				return Ok(());
			};
			
			let gamepath = sub.get_one::<String>("path").ok_or("path is required")?;
			let data = match noumenon.file::<Vec<u8>>(&gamepath) {
				Ok(v) => v,
				Err(err) => {println!("Failed loading file {err:?}"); return Ok(())}
			};
			
			// let Ok(data) = noumenon.file::<Vec<u8>>(&gamepath) else {
			// 	println!("Provided file path is invalid");
			// 	return Ok(());
			// };
			
			let gameext = gamepath.split(".").last().unwrap().to_owned();
			let defaultext = default_target_ext(&gameext);
			
			let out_file = match sub.get_one::<String>("out") {
				Some(v) => v.to_string(),
				None => std::env::current_dir()?.join(gamepath.split("/").last().unwrap().split(".").next().unwrap()).with_extension(&defaultext).to_string_lossy().to_string(),
			};
			let out_format = match sub.get_one::<String>("outformat") {
				Some(v) => v,
				None => out_file.split(".").last().unwrap(),
			};
			
			match aetherment::noumenon::Convert::from_ext(&gameext, &mut BufReader::new(Cursor::new(&data))) {
				Ok(converter) => {
					fn file_reader(path: &str) -> Option<Vec<u8>> {
						aetherment::noumenon_instance().unwrap().file::<Vec<u8>>(path).ok()
					}
					
					if out_file == "-" {
						let mut data = Vec::new();
						converter.convert(&out_format, &mut BufWriter::new(Cursor::new(&mut data)), Some(gamepath), Some(file_reader))?;
						std::io::stdout().lock().write_all(&data)?;
					} else {
						converter.convert(&out_format, &mut BufWriter::new(File::create(&out_file)?), Some(gamepath), Some(file_reader))?;
					}
				}
				
				Err(_) => {
					if out_file == "-" {
						std::io::stdout().lock().write_all(&data)?;
					} else {
						std::fs::write(&out_file, &data)?;
					}
				}
			}
		}
		
		Some(("convert", sub)) => {
			let in_file = sub.get_one::<String>("in").ok_or("in is required")?;
			let in_is_dir = Path::new(&in_file).is_dir();
			let in_format = match sub.get_one::<String>("informat") {
				Some(v) => v,
				None => if in_is_dir {
						return Err("informat is required if in path is directory")?;
					} else {
						in_file.split(".").last().unwrap()
					},
			};
			
			let out_file = sub.get_one::<String>("out").map(|v| v.to_string());
			
			if in_is_dir {
				let out_format = match sub.get_one::<String>("outformat") {
					Some(v) => v,
					None => default_target_ext(in_format),
				};
				
				fn do_dir(dir: &Path, cur_path: PathBuf, out_path: Option<&str>, in_format: &str, out_format: &str) {
					let Ok(reader) = std::fs::read_dir(dir) else {return};
					
					for entry in reader {
						let Ok(entry) = entry else {continue};
						let path = entry.path();
						
						if path.is_dir() {
							do_dir(&path, cur_path.join(path.file_name().unwrap()), out_path, in_format, out_format);
						} else if path.is_file() && path.extension().map(|v| v.to_str()) == Some(Some(in_format)) {
							let f = match File::open(&path) {
								Ok(v) => v,
								Err(err) => {println!("Failed converting {path:?} ({err:?})"); continue}
							};
							
							let converter = match aetherment::noumenon::Convert::from_ext(&in_format, &mut BufReader::new(f)) {
								Ok(v) => v,
								Err(err) => {println!("Failed converting {path:?} ({err:?})"); continue}
							};
							
							let out_path = match out_path {
								Some(v) => Path::new(v).join(&cur_path).join(path.with_extension(out_format).file_name().unwrap()),
								None => path.with_extension(out_format),
							};
							
							if let Some(parent) = out_path.parent() {
								_ = std::fs::create_dir_all(parent);
							}
							
							let f = match File::create(&out_path) {
								Ok(v) => v,
								Err(err) => {println!("Failed converting {path:?} ({err:?})"); continue}
							};
							
							if let Err(err) = converter.convert(&out_format, &mut BufWriter::new(f), None, None::<fn(&str) -> Option<Vec<u8>>>) {
								println!("Failed converting {path:?} ({err:?})");
								continue;
							}
							
							println!("Converted {path:?} to {out_path:?}");
						}
					}
				}
				
				do_dir(Path::new(in_file), PathBuf::new(), out_file.as_deref(), &in_format, &out_format);
			} else {
				let out_file = match out_file {
					Some(v) => v.to_string(),
					None => if in_file == "-" {
							"-".to_owned()
						} else {
							Path::new(&in_file).with_extension(default_target_ext(in_format)).to_string_lossy().to_string()
						}
				};
				let out_format = match sub.get_one::<String>("outformat") {
					Some(v) => v,
					None => out_file.split(".").last().unwrap(),
				};
				
				let converter = if in_file == "-" {
					let mut data = Vec::new();
					std::io::stdin().lock().read_to_end(&mut data)?;
					aetherment::noumenon::Convert::from_ext(&in_format, &mut BufReader::new(Cursor::new(data)))?
				} else {
					aetherment::noumenon::Convert::from_ext(&in_format, &mut BufReader::new(File::open(in_file)?))?
				};
				
				if out_file == "-" {
					let mut data = Vec::new();
					converter.convert(&out_format, &mut BufWriter::new(Cursor::new(&mut data)), None, None::<fn(&str) -> Option<Vec<u8>>>)?;
					std::io::stdout().lock().write_all(&data)?;
				} else {
					converter.convert(&out_format, &mut BufWriter::new(File::create(&out_file)?), None, None::<fn(&str) -> Option<Vec<u8>>>)?;
				}
			}
		}
		
		Some(("pack", sub)) => {
			let path = sub.get_one::<PathBuf>("path").ok_or("path is required")?;
			
			match aetherment::modman::modpack::create_mod(path, aetherment::modman::modpack::ModCreationSettings {
				current_game_files_hash: true,
			}) {
				Ok(path) => println!("Created modpack at {path:?}"),
				Err(err) => println!("Failed creating modpack\n\n{err:?}"),
			};
		}
		
		Some(("diff", sub)) => {
			let path = sub.get_one::<PathBuf>("path").ok_or("path is required")?;
			let file = File::open(path)?;
			
			match aetherment::modman::modpack::check_diff(file) {
				Ok(changes) => {
					println!("Changed files:\n");
					for path in changes {
						println!("{path}");
					}
				},
				Err(err) => println!("Checking diff failed\n\n{err:?}"),
			};
		}
		
		// Some(("game-directory", sub)) => {
		// 	todo!()
		// }
		
		_ => unreachable!()
	}
	
	Ok(())
}

fn default_target_ext(ext: &str) -> &str {
	match ext {
		"tex"  => "png",
		"atex" => "png",
		"png"  => "tex",
		"tif"  => "tex",
		"tiff" => "tex",
		"tga"  => "tex",
		"dds"  => "tex",
		"mdl"  => "gltf",
		"gltf" => "mdl",
		_ => ext,
	}
}