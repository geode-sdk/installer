use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
pub unsafe fn try_from_bundle(bundle: &str) -> Option<String> {
	use objc::{runtime::Object, *};
	use objc_foundation::{INSString, NSString};

	let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
	let url: *mut Object = msg_send![
		workspace,
		URLForApplicationWithBundleIdentifier: NSString::from_str(bundle)
	];
	let out: *mut NSString = msg_send![url, path];

	let out_str = (*out).as_str();
	if out_str.is_empty() {
		None
	} else {
		Some(out_str.to_string())
	}
}

#[cfg(target_os = "linux")]
fn get_linux_steam_path() -> Option<PathBuf> {
	let steam_path = PathBuf::from("~/.steam/root/steamapps/common/Geometry Dash/GeometryDash.exe");
	let flatpak_path = PathBuf::from("~/.var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/Geometry Dash/GeometryDash.exe");

	if steam_path.exists() && steam_path.is_file() {
		return Some(steam_path);
	} else if flatpak_path.exists() && flatpak_path.is_file() {
		return Some(flatpak_path);
	}

	None
}

#[cfg(target_os = "windows")]
fn get_path_from_steam() -> Option<PathBuf> {
	use std::fs::File;
	use std::io::{BufRead, BufReader, Lines, Result};
	use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

	fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
	where
		P: AsRef<Path>,
	{
		let file = File::open(filename)?;
		Ok(BufReader::new(file).lines())
	}

	let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
	let steam_key = hklm
		.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
		.ok()?;
	let install_path: String = steam_key.get_value("InstallPath").ok()?;

	let test_path =
		PathBuf::from(&install_path).join("steamapps/common/Geometry Dash/GeometryDash.exe");

	if test_path.exists() && test_path.is_file() {
		return Some(PathBuf::from(&test_path));
	}

	let config_path = PathBuf::from(&install_path).join("config/config.vdf");

	for line_res in read_lines(&config_path).ok()? {
		let line = line_res.ok()?;
		if line.to_string().contains("BaseInstallFolder_") {
			let end = line.rfind("\"").unwrap();
			let start = line[0..end].rfind("\"").unwrap();
			let result = &line[start + 1..end];
			let path =
				PathBuf::from(&result).join("steamapps/common/Geometry Dash/GeometryDash.exe");

			if path.exists() && path.is_file() {
				return Some(PathBuf::from(&path));
			}
		}
	}

	None
}

pub fn find_path() -> Option<String> {
	#[cfg(target_os = "macos")]
	unsafe {
		try_from_bundle("com.robtop.geometrydashmac")
			.or_else(|| try_from_bundle("com.camden.geometrydashmac")) // ew
			.or_else(|| try_from_bundle("com.camila.geometrydashmac"))
	}

	#[cfg(windows)]
	return get_path_from_steam().map(|s| s.to_str().unwrap().to_string().replace("\\", "/"));

	#[cfg(target_os = "linux")]
	return get_linux_steam_path().map(|s| s.to_str().unwrap().to_string().replace("\\", "/"));
}

pub fn validate_path(path: &Path) -> bool {
	if !path.exists() {
		return false;
	}

	#[cfg(target_os = "macos")]
	return path.is_dir()
		&& path
			.join(Path::new("Contents/Frameworks/DDHidLib.framework"))
			.exists();

	#[cfg(any(windows, target_os = "linux"))]
	return !path.is_dir()
		&& path.parent().is_some()
		&& path.parent().unwrap().join("libcocos2d.dll").exists();
}
