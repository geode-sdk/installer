
use std::io::Cursor;
use std::path::Path;
use crate::gd_path::validate_path;
use crate::error::ErrMessage;
use reqwest::blocking as reqwest;
use std::fs;

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
		unimplemented!("Windows lol");
	} else {
		unreachable!();
	};

	#[cfg(target_os = "macos")]
	if !dest_path.join("restore_fmod.dylib").exists() {
		let fmod = dest_path.join("libfmod.dylib");
		fs::rename(fmod, dest_path.join("restore_fmod.dylib")).with_msg("Unable to restore libfmod")?;
	}

	#[cfg(windows)]
	unimplemented!("Implement mod loader detection");

	zip_extract::extract(Cursor::new(download_file.bytes().unwrap()), &dest_path, true).with_msg("Unable to extract archive")?;

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
		let src_path: &Path = unimplemented!("Windows lol");
		fs::remove_file(src_path.join("XInput9_1_0.dll")).with_msg("Unable to remove XInput9_1_0")?;
		fs::remove_file(src_path.join("Geode.dll")).with_msg("Unable to remove Geode")?;
		fs::remove_file(src_path.join("Geode.lib")).with_msg("Unable to remove Geode")?;
		fs::remove_file(src_path.join("GeodeBootstrapper.dll")).with_msg("Unable to remove GeodeBootstrapper")?;
	}

	let geode_dir = if cfg!(target_os = "macos") {
		path.join(Path::new("Contents/geode"))
	} else if cfg!(windows) {
		unimplemented!("Windows lol");
	} else {
		unreachable!();
	};

	if geode_dir.exists() {
		fs::remove_dir_all(geode_dir).with_msg("Unable to remove Geode directory")?;
	}

	Ok(())
}
