fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        // Match the small memory footprint of the deployed Reflector feed.
        println!("cargo:rustc-link-arg=-zstack-size=65536");
    }
}
