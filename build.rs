
fn main() {
	#[cfg(target_os = "macos")] {
		println!("cargo:rustc-link-arg=-framework");
		println!("cargo:rustc-link-arg=Cocoa");
	}
}