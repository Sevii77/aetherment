use std::{fs::File, io::{BufReader, BufWriter, Cursor, Read, Write}, path::PathBuf};

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
				.help("Format of the input file, required if input is stdin")
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
			let Some(noumenon) = aetherment::noumenon() else {
				println!("Game install path set in config is invalid");
				return Ok(());
			};
			
			let gamepath = sub.get_one::<String>("path").ok_or("path is required")?;
			let Ok(data) = noumenon.file::<Vec<u8>>(&gamepath) else {
				println!("Provided file path is invalid");
				return Ok(());
			};
			
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
			
			let converter = aetherment::noumenon_::Convert::from_ext(&gameext, &mut BufReader::new(Cursor::new(data)))?;
			
			if out_file == "-" {
				let mut data = Vec::new();
				converter.convert(&out_format, &mut BufWriter::new(Cursor::new(&mut data)))?;
				std::io::stdout().lock().write_all(&data)?;
			} else {
				converter.convert(&out_format, &mut BufWriter::new(File::create(&out_file)?))?;
			}
			
			// let converter = aetherment::noumenon_::Convert::from_ext(&gameext, &mut BufReader::new(Cursor::new(data)))?;
			// let outfile = std::env::current_dir()?.join(gamepath.split("/").last().unwrap().split(".").next().unwrap()).with_extension(&format);
			// 
			// if let Some(parent) = outfile.parent() {
			// 	std::fs::create_dir_all(parent)?;
			// }
			// 
			// converter.convert(&format, &mut BufWriter::new(std::fs::File::create(outfile)?))?;
		}
		
		Some(("convert", sub)) => {
			let in_file = sub.get_one::<String>("in").ok_or("in is required")?;
			let in_format = match sub.get_one::<String>("informat") {
				Some(v) => v,
				None => in_file.split(".").last().unwrap(),
			};
			
			let out_file = match sub.get_one::<String>("out") {
				Some(v) => v.to_string(),
				None => if in_file == "-" {"-".to_owned()} else {std::path::Path::new(&in_file).with_extension(default_target_ext(in_format)).to_string_lossy().to_string()}
			};
			let out_format = match sub.get_one::<String>("outformat") {
				Some(v) => v,
				None => out_file.split(".").last().unwrap(),
			};
			
			let converter = if in_file == "-" {
				let mut data = Vec::new();
				std::io::stdin().lock().read_to_end(&mut data)?;
				aetherment::noumenon_::Convert::from_ext(&in_format, &mut BufReader::new(Cursor::new(data)))?
			} else {
				aetherment::noumenon_::Convert::from_ext(&in_format, &mut BufReader::new(File::open(in_file)?))?
			};
			
			if out_file == "-" {
				let mut data = Vec::new();
				converter.convert(&out_format, &mut BufWriter::new(Cursor::new(&mut data)))?;
				std::io::stdout().lock().write_all(&data)?;
			} else {
				converter.convert(&out_format, &mut BufWriter::new(File::create(&out_file)?))?;
			}
		}
		
		Some(("pack", sub)) => {
			let path = sub.get_one::<PathBuf>("path").ok_or("path is required")?;
			
			match aetherment::modman::create_mod(path, aetherment::modman::ModCreationSettings {
				current_game_files_hash: true,
			}) {
				Ok(path) => println!("Created modpack at {path:?}"),
				Err(err) => println!("Failed creating modpack\n\n{err:?}"),
			};
		}
		
		Some(("diff", sub)) => {
			let path = sub.get_one::<PathBuf>("path").ok_or("path is required")?;
			let file = File::open(path)?;
			
			match aetherment::modman::check_diff(file) {
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
		"tex" => "png",
		_ => ext,
	}
}