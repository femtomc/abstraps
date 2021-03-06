#![allow(dead_code)]

extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/////
///// MLIR backend
/////

fn llvmconfigshim(arg: &str) -> String {
    let call = format!("llvm-config {}", arg);
    let tg = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &call[..]])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&call[..])
            .output()
            .expect("failed to execute process")
    }
    .stdout;
    let mut s = String::from_utf8_lossy(&tg);
    s.to_mut().pop();
    s.to_string()
}

fn get_system_libcpp() -> Option<&'static str> {
    if cfg!(target_env = "msvc") {
        None
    } else if cfg!(target_os = "macos") || cfg!(target_os = "freebsd") {
        Some("c++")
    } else {
        Some("stdc++")
    }
}

fn llvm_libs() -> Vec<String> {
    let libdir = llvmconfigshim("--libdir");
    let paths = fs::read_dir(libdir).unwrap();
    paths
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(String::from))
            })
        })
        .collect::<Vec<String>>()
}

fn mlir() {
    // Build bindings to MLIR C API.
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    let includedir = llvmconfigshim("--includedir");
    let libdir = llvmconfigshim("--libdir");
    println!("cargo:libdir={}", libdir);
    println!("cargo:rustc-link-search=all={}", libdir);
    println!(
        "cargo:rustc-link-lib=dylib={}",
        get_system_libcpp().unwrap()
    );
    for l in llvm_libs().iter() {
        if l.contains("libMLIR") && l.contains(".a") {
            let len = l.len();
            println!("cargo:rustc-link-lib={}", l[3..(len - 2)].to_string());
        }
    }
    println!("cargo:rustc-link-lib=LLVM");
    println!("cargo:rustc-link-lib=MLIR");
    println!("cargo:rustc-link-lib=LLVMExecutionEngine");
    println!("cargo:rustc-link-lib=MLIRExecutionEngine");
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .clang_arg(format!("-I/{}", includedir))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings.");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/////
///// Feature selection.
/////

fn main() {
    #[cfg(feature = "mlir")]
    mlir();
}
