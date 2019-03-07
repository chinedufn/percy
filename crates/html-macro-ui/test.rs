// ignore-test - not a test

#![cfg(test)]

extern crate compiletest_rs as compiletest;

use std::env;
use std::fs;
use std::path::PathBuf;

/// Tests that ensure that we print out compile time error messages for different
/// scenarios of malformed html!
///
/// Adapted from
/// https://github.com/rustwasm/wasm-bindgen/blob/c4dcaee1b93b4dff748412f2d31f997e9d7d9273/crates/macro/ui-tests/test.rs
fn main() {
    let mut config = compiletest::Config::default();
    config.mode = "ui".parse().expect("invalid mode");
    let mut me = env::current_exe().unwrap();
    me.pop();
    config.target_rustcflags = Some(format!("-L {}", me.display()));
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config.src_base = src;

    me.pop();
    me.pop();
    config.build_base = me.join("tests/compile-fail");
    drop(fs::remove_dir_all(&config.build_base));
    compiletest::run_tests(&config);
}
