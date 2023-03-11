fn main() {
	#[cfg(target_os = "macos")]
	{
		println!("cargo:rustc-link-arg=-framework");
		println!("cargo:rustc-link-arg=Cocoa");
		println!("cargo:rustc-link-arg=-mmacosx-version-min=10.9")
	}
}
