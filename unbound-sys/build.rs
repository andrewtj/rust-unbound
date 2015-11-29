use std::env;

fn main() {
    let mode = if env::var_os("UNBOUND_STATIC").is_some() {
        "static"
    } else {
        "dylib"
    };
    println!("cargo:rustc-link-lib={}=unbound", mode);

    if let Some(dir) = env::var("UNBOUND_INCLUDE_DIR").ok() {
        println!("cargo:include={}", dir);
    }

    if let Some(dir) = env::var("UNBOUND_LIB_DIR").ok() {
        println!("cargo:rustc-link-search=native={}", dir);
    };
}
