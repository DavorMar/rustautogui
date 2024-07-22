// build.rs
use std::env;

fn main() {
    //required for X11 binding connection
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xtst");
    }
}