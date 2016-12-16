use std::env;

fn main() {
    let mode = if env::var_os("UNBOUND_STATIC").is_some() {
        "static"
    } else {
        "dylib"
    };
    println!("cargo:rustc-link-lib={}=unbound", mode);

    if let Some(dir) = env::var("UNBOUND_DIR").ok() {
        println!("cargo:include={}/include", dir);
        println!("cargo:rustc-link-search=native={}/lib", dir);
    }
}
