use crate::error::ErrMessage;
use crate::gd_path::validate_path;
use crate::register::{register_extension, unregister_extension};
use reqwest::blocking as reqwest;
use serde::Deserialize;
use std::fs;
use std::io::Cursor;
use std::path::Path;

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

#[derive(Deserialize)]
struct GithubReleaseAsset {
	name: String,
	browser_download_url: String,
}

#[derive(Deserialize)]
struct GithubApiResponse {
	assets: Vec<GithubReleaseAsset>,
}

pub fn install_to(path: &Path) -> Result<(), String> {
	if !validate_path(path) {
		Err("Invalid Geometry Dash path".to_string())?;
	}

	let latest_version = serde_json::from_str::<GithubApiResponse>(
		&reqwest::Client::builder()
			.user_agent("github_api/1.0")
			.build()
			.with_msg("Unable to fetch latest release info")?
			.get("https://api.github.com/repos/geode-sdk/geode/releases/latest")
			.send()
			.with_msg("Unable to fetch latest release info")?
			.text()
			.with_msg("Unable to read latest release info")?,
	)
	.with_msg("Request rate-limited")?;

	let mut url_str = None;
	for asset in latest_version.assets {
		if asset.name.contains(if cfg!(target_os = "macos") {
			"mac"
		} else {
			"win"
		}) {
			url_str = Some(asset.browser_download_url);
			break;
		}
	}
	if url_str.is_none() {
		Err(format!(
			"No download for {} found",
			if cfg!(target_os = "macos") {
				"MacOS"
			} else {
				"Windows"
			}
		))?;
	}

	let download_file = reqwest::get(url_str.unwrap()).with_msg("Unable to download Geode")?;

	let dest_path = if cfg!(target_os = "macos") {
		path.join(Path::new("Contents/Frameworks/"))
	} else if cfg!(windows) || cfg!(target_os = "linux") {
		path.parent().unwrap().to_path_buf()
	} else {
		unreachable!();
	};

	#[cfg(target_os = "macos")]
	if !dest_path.join("restore_fmod.dylib").exists() {
		let fmod = dest_path.join("libfmod.dylib");
		fs::rename(fmod, dest_path.join("restore_fmod.dylib"))
			.with_msg("Unable to restore libfmod")?;
	}

	#[cfg(windows)]
	if let Some(mod_loader) = check_for_modloaders(&dest_path) {
		// check_for_modloaders will detect the xinput dll
		// as an unknown mod loader (as it could be something like proxydllloader),
		// however it could also be geode itself, so in that case ignore and install over geode,
		// effectively updating it
		if !dest_path.join("Geode.dll").exists() {
			return Err(format!(
				"It seems like you already have a mod loader ({}) installed! \
				Please uninstall it first before installing Geode.",
				mod_loader
			));
		}
	}

	zip_extract::extract(
		Cursor::new(download_file.bytes().unwrap()),
		&dest_path,
		true,
	)
	.with_msg("Unable to extract archive")?;

	#[cfg(windows)]
	{
		// This file comes with the geode release for developers,
		// however it is not needed by the end user
		let _ = fs::remove_file(dest_path.join("Geode.lib"));
	}

	register_extension(path)?;

	Ok(())
}

pub fn uninstall_from(path: &Path) -> Result<(), String> {
	if !validate_path(path) {
		Err("Invalid Geometry Dash path".to_string())?;
	}

	#[cfg(target_os = "macos")]
	{
		let src_path = path.join(Path::new("Contents/Frameworks/"));

		if !src_path.join("restore_fmod.dylib").exists() {
			Err("Unable to find restored fmod.")?;
		}

		fs::remove_file(src_path.join("libfmod.dylib")).with_msg("Unable to remove libfmod")?;
		fs::remove_file(src_path.join("Geode.dylib")).with_msg("Unable to remove Geode")?;
		fs::remove_file(src_path.join("GeodeBootstrapper.dylib"))
			.with_msg("Unable to remove GeodeBootstrapper")?;
		fs::rename(
			src_path.join("restore_fmod.dylib"),
			src_path.join("libfmod.dylib"),
		)
		.with_msg("Unable to restore fmod")?;
	}
	#[cfg(windows)]
	{
		let src_path: &Path = path.parent().unwrap();
		fs::remove_file(src_path.join("XInput9_1_0.dll"))
			.with_msg("Unable to remove XInput9_1_0")?;
		fs::remove_file(src_path.join("Geode.dll")).with_msg("Unable to remove Geode")?;
		let _ = fs::remove_file(src_path.join("Geode.lib"));
		fs::remove_file(src_path.join("GeodeBootstrapper.dll"))
			.with_msg("Unable to remove GeodeBootstrapper")?;
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
