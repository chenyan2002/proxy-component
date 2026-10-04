fn main() {
    // Link with 1-byte wasm pages; libc-alloc's `sbrk` makes wasi-libc's malloc work with them.
    if std::env::var_os("CARGO_FEATURE_ONE_BYTE_PAGE").is_some() {
        println!("cargo:rustc-link-arg=--page-size=1");
    }
}
