#[macro_export]
macro_rules! log {
	(ftl, $($e:tt)*) => {{
		::log::error!(target: "aetherment", $($e)*);
	}};
	
	(log, $($e:tt)*) => {{
		::log::debug!(target: "aetherment", $($e)*);
	}};
	
	(err, $($e:tt)*) => {{
		::log::warn!(target: "aetherment", $($e)*);
	}};
	
	($($e:tt)*) => {{
		::log::debug!(target: "aetherment", $($e)*);
	}};
}