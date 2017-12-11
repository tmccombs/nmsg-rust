extern crate cmake;

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    let dst = cmake::Config::new("nng")
        .define("NNG_STATIC_LIB", "ON")
        .define("NNG_ENABLE_DOC", "OFF")
        .define("NNG_TESTS", "OFF")
        .build();

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=mswsock");
    }

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    println!("cargo:rerun-if-changed=nng");
}
