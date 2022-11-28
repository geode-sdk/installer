
use std::io::Cursor;
use std::path::Path;
use crate::gd_path::validate_path;
use crate::error::ErrMessage;
use crate::register::{register_extension, unregister_extension};
use reqwest::blocking as reqwest;
use std::fs;

#[cfg(windows)]
fn check_for_modloaders(path: &Path) -> Option<&str> {
	if path.join("ToastedMarshmellow.dll").exists() {
		Some("GDHM")
	} else if path.join("hackpro.dll").exists() {
		Some("Mega Hack")
	} else if path.join("quickldr.dll").exists() {
		Some("QuickLdr")
	} else if path.join("XInput9_1_0.dll").exists() {
		Some("Unknown")
	} else if path.join("gddllloader.dll").exists() {
		Some("GD DLL Loader")
	} else {
		None
	}
}

pub fn install_to(path: &Path) -> Result<(), String> {
	if !validate_path(path) {
		Err("Invalid Geometry Dash path".to_string())?;
	}

	let latest_version = reqwest::get("https://raw.githubusercontent.com/geode-sdk/geode/main/VERSION")
		.with_msg("Unable to fetch version")?
		.text()
		.with_msg("Unable to decode version")?;

	let url_str = format!(
		"https://github.com/geode-sdk/geode/releases/download/v{ver}/geode-v{ver}-{platform}.zip",
		ver = latest_version,
		platform = if cfg!(target_os = "macos") { "mac" } else { "win" }
	);

	let download_file = reqwest::get(url_str).with_msg("Unable to download Geode")?;

	let dest_path = if cfg!(target_os = "macos") {
		path.join(Path::new("Contents/Frameworks/"))
	} else if cfg!(windows) {
		path.parent().unwrap().to_path_buf()
	} else {
		unreachable!();
	};

	#[cfg(target_os = "macos")]
	if !dest_path.join("restore_fmod.dylib").exists() {
		let fmod = dest_path.join("libfmod.dylib");
		fs::rename(fmod, dest_path.join("restore_fmod.dylib")).with_msg("Unable to restore libfmod")?;
	}

	#[cfg(windows)]
	if let Some(mod_loader) = check_for_modloaders(&path.parent().unwrap()) {
		return Err(format!(
			"It seems like you already have a mod loader ({}) installed! \
			Please uninstall it first before installing Geode.",
			mod_loader
		));
	}

	zip_extract::extract(Cursor::new(download_file.bytes().unwrap()), &dest_path, true).with_msg("Unable to extract archive")?;

	register_extension(path)?;

	Ok(())
}

pub fn uninstall_from(path: &Path) -> Result<(), String> {
	if !validate_path(path) {
		Err("Invalid Geometry Dash path".to_string())?;
	}

	#[cfg(target_os = "macos")] {
		let src_path = path.join(Path::new("Contents/Frameworks/"));

		if !src_path.join("restore_fmod.dylib").exists() {
			Err("Unable to find restored fmod.")?;
		}

		fs::remove_file(src_path.join("libfmod.dylib")).with_msg("Unable to remove libfmod")?;
		fs::remove_file(src_path.join("Geode.dylib")).with_msg("Unable to remove Geode")?;
		fs::remove_file(src_path.join("GeodeBootstrapper.dylib")).with_msg("Unable to remove GeodeBootstrapper")?;
		fs::rename(src_path.join("restore_fmod.dylib"), src_path.join("libfmod.dylib")).with_msg("Unable to restore fmod")?;
	}
	#[cfg(windows)] {
		let src_path: &Path = path.parent().unwrap();
		fs::remove_file(src_path.join("XInput9_1_0.dll")).with_msg("Unable to remove XInput9_1_0")?;
		fs::remove_file(src_path.join("Geode.dll")).with_msg("Unable to remove Geode")?;
		fs::remove_file(src_path.join("Geode.lib")).with_msg("Unable to remove Geode")?;
		fs::remove_file(src_path.join("GeodeBootstrapper.dll")).with_msg("Unable to remove GeodeBootstrapper")?;
	}

	let geode_dir = if cfg!(target_os = "macos") {
		path.join(Path::new("Contents/geode"))
	} else if cfg!(windows) {
		path.parent().unwrap().join("geode")
	} else {
		unreachable!();
	};

	if geode_dir.exists() {
		fs::remove_dir_all(geode_dir).with_msg("Unable to remove Geode directory")?;
	}

	unregister_extension(path)?;

	Ok(())
}
