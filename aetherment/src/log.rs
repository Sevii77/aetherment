#[repr(u8)]
pub enum LogType {
	Log = 0,
	Error = 1,
	Fatal = 255,
}

pub(crate) static mut LOG: fn(LogType, &str) = |typ, msg| {
	let typ = match typ {
		LogType::Log => "LOG",
		LogType::Error => "ERROR",
		LogType::Fatal => "FATAL",
	};
	
	println!("[{typ}] {msg}");
};

#[macro_export]
macro_rules! log {
	(ftl, $($e:tt)*) => {{
		let s = format!($($e)*);
		unsafe{crate::log::LOG(crate::log::LogType::Fatal, &s)};
	}};
	
	(log, $($e:tt)*) => {{
		let s = format!($($e)*);
		unsafe{crate::log::LOG(crate::log::LogType::Log, &s)};
	}};
	
	(err, $($e:tt)*) => {{
		let s = format!($($e)*);
		unsafe{crate::log::LOG(crate::log::LogType::Error, &s)};
	}};
	
	($($e:tt)*) => {{
		let s = format!($($e)*);
		unsafe{crate::log::LOG(crate::log::LogType::Log, &s)};
	}};
}