use crate::error::ErrMessage;
use std::path::Path;

pub fn register_extension(path: &Path) -> Result<(), String> {
	#[cfg(target_os = "macos")]
	{
		use core_foundation::url::CFURL;
		use launch_services::register_url;
		use plist::{Dictionary, Value};

		let info_plist = path.join("Contents/Info.plist");
		if !info_plist.exists() {
			Err("Unable to find Info.plist".to_string())?;
		}

		let mut entry = Dictionary::new();

		entry.insert(
			"CFBundleTypeName".to_string(),
			Value::String("Geode mod file".to_string()),
		);
		entry.insert(
			"CFBundleTypeExtensions".to_string(),
			Value::Array(vec![Value::String("geode".to_string())]),
		);
		entry.insert(
			"CFBundleTypeRole".to_string(),
			Value::String("Editor".to_string()),
		);
		entry.insert(
			"CFBundleTypeIconFile".to_string(),
			Value::String("geode-file".to_string()),
		);

		let mut plist_root = Value::from_file(&info_plist).with_msg("Unable to read Info.plist")?;

		plist_root.as_dictionary_mut().unwrap().insert(
			"CFBundleDocumentTypes".to_string(),
			Value::Array(vec![Value::Dictionary(entry)]),
		);

		plist_root
			.to_file_xml(&info_plist)
			.with_msg("Unable to write to Info.plist")?;

		match register_url(&CFURL::from_path(path, true).unwrap(), true) {
			Ok(_) => (),
			Err(i) => Err(format!("Error code {} encountered when registering", i))?,
		};

		std::fs::write(
			path.join("Contents/Resources/geode-file.icns"),
			std::include_bytes!("../assets/geode-file-mac.icns"),
		)
		.with_msg("Unable to copy icon")?;
	}

	// #[cfg(windows)] {
	// 	unimplemented!("implement registering .geode file extension");
	// }

	Ok(())
}

pub fn unregister_extension(path: &Path) -> Result<(), String> {
	#[cfg(target_os = "macos")]
	{
		use core_foundation::url::CFURL;
		use launch_services::register_url;
		use plist::Value;

		let info_plist = path.join("Contents/Info.plist");
		if !info_plist.exists() {
			Err("Unable to find Info.plist".to_string())?;
		}

		let mut plist_root = Value::from_file(&info_plist).with_msg("Unable to read Info.plist")?;
		plist_root
			.as_dictionary_mut()
			.unwrap()
			.remove("CFBundleDocumentTypes");
		plist_root
			.to_file_xml(&info_plist)
			.with_msg("Unable to write to Info.plist")?;

		match register_url(&CFURL::from_path(path, true).unwrap(), true) {
			Ok(_) => (),
			Err(i) => Err(format!("Error code {} encountered when unregistering", i))?,
		};
	}

	// #[cfg(windows)] {
	// 	unimplemented!("implement unregistering .geode file extension");
	// }

	Ok(())
}
