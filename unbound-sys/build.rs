extern crate cc;
extern crate tempdir;

use std::env;
use std::path::PathBuf;
use std::process::Stdio;

use tempdir::TempDir;

fn main() {
    println!("cargo:rerun-if-env-changed=UNBOUND_DIR");
    println!("cargo:rerun-if-env-changed=UNBOUND_STATIC");
    let mut cc_args = Vec::new();
    let mode = if env::var_os("UNBOUND_STATIC").is_some() {
        "static"
    } else {
        "dylib"
    };
    if let Ok(dir) = env::var("UNBOUND_DIR").map(PathBuf::from) {
        let include_dir = dir.join("include");
        assert!(
            include_dir.join("unbound.h").is_file(),
            "{} does not contain lib/unbound.h", dir.display()
        );
        let lib_dir = dir.join("lib");
        assert!(
            lib_dir.join("libunbound.a").is_file(),
            "{} does not contain lib/libunbound.a", dir.display()
        );
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-include={}", include_dir.display());
        cc_args.push(format!("-L{}", lib_dir.display()));
        cc_args.push(format!("-I{}", include_dir.display()));
    }
    println!("cargo:rustc-link-lib={}=unbound", mode);
    println!("cargo:rustc-flags=-l {}=unbound", mode);

    let temp = TempDir::new("ufd").expect("temp dir");
    let rl = temp.path().join("rl.c");
    std::fs::write(&rl, r#"
    #include <unbound.h>
    #include <string.h>

    int main(void) {
        struct ub_result result;
        memset(&result, 0, sizeof(struct ub_result));
        return result.was_ratelimited;
    }
"#).expect("write rl.c");
    let have_was_ratelimited = { let mut cmd = cc::Build::new()
        .cargo_metadata(false)
        .get_compiler()
        .to_command();
        cmd
        .current_dir(temp.path())
        .args(&cc_args)
        .arg("-lunbound")
        .arg(rl.to_string_lossy().as_ref());
println!("{:?}", cmd);
cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map(|o| o.status.success())
        .expect("run cc")
    };
    if have_was_ratelimited {
        println!("cargo:rustc-cfg=ub_ctx_has_was_ratelimited");
    }
}
