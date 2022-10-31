use std::path::Path;

#[cfg(target_os = "macos")]
unsafe fn try_from_bundle(bundle: &str) -> Option<String> {
	use objc::{*, runtime::Object};
	use objc_foundation::{NSString, INSString};

	let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
	let url: *mut Object = msg_send![workspace, URLForApplicationWithBundleIdentifier: NSString::from_str(bundle)];
	let out: *mut NSString = msg_send![url, path];

	let out_str = (*out).as_str();
	if out_str.is_empty() {
		None
	} else {
		Some(out_str.to_string())
	}
}

pub fn find_path() -> Option<String> {
	#[cfg(target_os = "macos")]
	unsafe {
		try_from_bundle("com.robtop.geometrydashmac")
			.or_else(|| try_from_bundle("com.camden.geometrydashmac")) // ew
			.or_else(|| try_from_bundle("com.camila.geometrydashmac"))
	}

	#[cfg(windows)]
	unimplemented!("Windows lol");
}

pub fn validate_path(path: &Path) -> bool {
	if !path.exists() {
		return false;
	}

	#[cfg(target_os = "macos")]
	return path.is_dir() && path.join(Path::new("Contents/Frameworks/libfmod.dylib")).exists();

	#[cfg(windows)]
	return !path.is_dir() && path.parent().is_some() && path.parent().unwrap().join("libcocos2d.dll").exists();
}