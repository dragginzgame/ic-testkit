//! A command-scoped PocketIC server owner. No installer or binary discovery.

#[cfg(all(unix, not(target_arch = "wasm32")))]
mod native;

#[cfg(all(unix, not(target_arch = "wasm32")))]
fn main() -> std::process::ExitCode {
    native::main()
}

#[cfg(not(all(unix, not(target_arch = "wasm32"))))]
fn main() {
    eprintln!("ic-testkit-server requires a supported Unix host");
    std::process::exit(2);
}
