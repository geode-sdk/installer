use std::{path::{PathBuf, Path}, io::Cursor};
use serde::Deserialize;
use std::fs;
use crate::{find_gd::path_contains_gd, extension::{register_extension, unregister_extension}};
use crate::find_gd::gd_dir_from_path;

#[derive(PartialEq)]
pub enum FoundGD {
    None,
    Vanilla(PathBuf),
    Geode(PathBuf),
}

pub fn check_for_gd(path: &Path) -> FoundGD {
    if !path_contains_gd(path) {
        return FoundGD::None;
    }
    #[cfg(windows)]
    if path.parent().unwrap().join("Geode.dll").exists() {
        return FoundGD::Geode(path.to_owned());
    }
    #[cfg(macos)]
    if path.join("Contents/Frameworks/Geode.dylib") {
        return FoundGD::Geode(path.to_owned());
    }
    FoundGD::Vanilla(path.to_owned())
}

#[cfg(windows)]
pub fn check_modloaders(path: &Path) -> Option<String> {
	let path = gd_dir_from_path(path);
    if path.join("ToastedMarshmellow.dll").exists() {
        Some(String::from(
            "You appear to have GD Hacker Mode installed. Please note that \
            GDHM is not compatible with Geode and will be uninstalled. You can \
            find alternative mod menus through the Downloads page in-game."
        ))
    }
    else if path.join("hackpro.dll").exists() {
        Some(String::from(
            "You appear to have Mega Hack installed. Please note that the \
            installer will remove Mega Hack, as Geode is not compatible with \
            Mega Hack v6/v7. If you'd like to use Mega Hack alongside Geode, \
            please wait for Mega Hack v8, which will be compatible."
        ))
    }
    else if path.join("quickldr.dll").exists() {
        Some(String::from(
            "You appear to have QuickLdr installed. Please note that Geode will \
            replace QuickLdr, and Geode won't load the mods you have installed \
            for QuickLdr. Look for Geode-compatible versions / alternatives in \
            the Downloads page in-game."
        ))
    }
    else if path.join("XInput9_1_0.dll").exists() || path.join("gddllloader.dll").exists() {
        Some(String::from(
            "You appear to have some other mod loader installed. Please note that \
            the installer will uninstall this mod loader, and Geode won't load \
            any .DLL mods you have. Look for Geode-compatible \
            versions / alternatives for the mods you use in the Downloads page in-game."
        ))
    }
    else {
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
    tag_name: String,
	assets: Vec<GithubReleaseAsset>,
}

pub async fn install_to(path: &Path) -> Result<(), String> {
	if check_for_gd(path) == FoundGD::None {
		Err("Invalid Geometry Dash path".to_string())?;
	}

	let latest_version = serde_json::from_str::<GithubApiResponse>(
		&reqwest::Client::builder()
			.user_agent("github_api/1.0")
			.build()
			.map_err(|e| "Unable to fetch latest release info: {e}")?
			.get("https://api.github.com/repos/geode-sdk/geode/releases/latest")
			.send()
            .await
			.map_err(|e| format!("Unable to fetch latest release info: {e}"))?
			.text()
            .await
			.map_err(|e| format!("Unable to read latest release info: {e}"))?,
	)
	.map_err(|e| "Request rate-limited - please try again later")?;

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

	let download_file = reqwest::get(url_str.unwrap())
        .await
        .map_err(|e| format!("Unable to download Geode: {e}"))?;

	let dest_path = if cfg!(target_os = "macos") {
		path.join(Path::new("Contents/Frameworks/"))
	} else if cfg!(windows) {
		path.parent().unwrap().to_path_buf()
	} else {
		unreachable!();
	};

	let bytes = download_file.bytes().await
		.map_err(|e| format!("Unable to download Geode: {e}"))?;

	#[cfg(target_os = "macos")]
	if !dest_path.join("restore_fmod.dylib").exists() {
		let fmod = dest_path.join("libfmod.dylib");
		fs::rename(fmod, dest_path.join("restore_fmod.dylib"))
			.map_err(|e| format!("Unable to restore libfmod: {e}"))?;
	}

	#[cfg(windows)]
	{
		// Uninstall existing mod loaders
		drop(fs::remove_file(dest_path.join("XInput9_1_0.dll")));
		drop(fs::remove_file(dest_path.join("gddllloader.dll")));
		drop(fs::remove_file(dest_path.join("quickldr.dll")));
	}

	zip_extract::extract(Cursor::new(bytes), &dest_path, true)
	.map_err(|e| format!("Unable to extract archive: {e}"))?;

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
	if !matches!(check_for_gd(path), FoundGD::Geode(_)) {
		Err("Geode is not installed here".to_string())?;
	}

	#[cfg(target_os = "macos")]
	{
		let src_path = path.join(Path::new("Contents/Frameworks/"));

		if !src_path.join("restore_fmod.dylib").exists() {
			Err("Unable to find restored fmod.")?;
		}

		fs::remove_file(src_path.join("libfmod.dylib")).map_err(|e| format!("Unable to remove libfmod: {e}"))?;
		fs::remove_file(src_path.join("Geode.dylib")).map_err(|e| format!("Unable to remove Geode: {e}"))?;
		fs::remove_file(src_path.join("GeodeBootstrapper.dylib"))
			.map_err(|e| format!("Unable to remove GeodeBootstrapper: {e}"))?;
		fs::rename(
			src_path.join("restore_fmod.dylib"),
			src_path.join("libfmod.dylib"),
		)
		.map_err(|e| format!("Unable to restore fmod: {e}"))?;
	}
	#[cfg(windows)]
	{
		let src_path: &Path = path.parent().unwrap();
		fs::remove_file(src_path.join("XInput9_1_0.dll"))
			.map_err(|e| format!("Unable to remove XInput9_1_0: {e}"))?;
		fs::remove_file(src_path.join("Geode.dll")).map_err(|e| format!("Unable to remove Geode: {e}"))?;
		let _ = fs::remove_file(src_path.join("Geode.lib"));
		fs::remove_file(src_path.join("GeodeBootstrapper.dll"))
			.map_err(|e| format!("Unable to remove GeodeBootstrapper: {e}"))?;
	}

	let geode_dir = if cfg!(target_os = "macos") {
		path.join(Path::new("Contents/geode"))
	} else if cfg!(windows) {
		path.parent().unwrap().join("geode")
	} else {
		unreachable!();
	};

	if geode_dir.exists() {
		fs::remove_dir_all(geode_dir).map_err(|e| format!("Unable to remove Geode directory: {e}"))?;
	}

	unregister_extension(path)?;

	Ok(())
}
