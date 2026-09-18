fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        // Repeated provider invocations must fit the transaction memory budget.
        println!("cargo:rustc-link-arg=-zstack-size=65536");
    }
}
