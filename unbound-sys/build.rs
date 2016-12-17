extern crate libbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let mode = if env::var_os("UNBOUND_STATIC").is_some() {
        "static"
    } else {
        "dylib"
    };
    println!("cargo:rustc-link-lib={}=unbound", mode);

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let header = if let Some(dir) = env::var("UNBOUND_DIR").ok() {
        println!("cargo:include={}/include", dir);
        println!("cargo:rustc-link-search=native={}/lib", dir);
        format!("{}/include/unbound.h", dir)
    } else {
        "unbound.h".into()
    };

    let bindings = libbindgen::Builder::default()
        .no_unstable_rust()
        .ctypes_prefix("::libc")
        .header(header)
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
