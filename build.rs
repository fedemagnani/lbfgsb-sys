// build.rs

use std::env::var;
use std::path::PathBuf;
use std::process::Command;

macro_rules! feature(
    ($name:expr) => (var(concat!("CARGO_FEATURE_", $name)).is_ok());
);

macro_rules! variable(
    ($name:expr) => (var($name).unwrap());
);

fn main() {
    let kind = if feature!("STATIC") {
        "static"
    } else {
        "dylib"
    };
    let source = PathBuf::from("fortran");
    let output = PathBuf::from(variable!("OUT_DIR").replace(r"\", "/"));
    let os = if cfg!(target_os = "macos") {
        "Macos"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Linux"
    };
    let make_cmd = if os == "Windows" {
        "mingw32-make"
    } else {
        "make"
    };
    run(Command::new(make_cmd)
        .arg(kind)
        .arg(format!("OUTPUT={}", output.display()))
        .arg(format!("OSNAME={}", os))
        .current_dir(&source));
    println!("cargo:rustc-link-search={}", output.display());

    let mut fc_lib_type = "dylib";
    let target = variable!("TARGET");

    if target == "aarch64-apple-darwin" {
        // Poke $FC$ for dynamic lib folder
        let fc_out = Command::new(variable!("FC"))
            .arg("-print-file-name=libgfortran.a")
            .output()
            .expect("Failed to find libgfortran.a");
        let fc_stdout = String::from_utf8(fc_out.stdout).expect("Invalid path to libgfortran.a");
        let fc_lib_cwd = PathBuf::from(fc_stdout.to_string());
        let fc_lib_pwd = fc_lib_cwd
            .parent()
            .expect("Path to libgfortran.a not found");
        println!("cargo:rustc-link-search={}", fc_lib_pwd.to_str().unwrap());
        // Poke $FC$ for dynamic lib folder
        let fc_out = Command::new(variable!("FC"))
            .arg("-print-file-name=libgcc.a")
            .output()
            .expect("Failed to find libgcc.a");
        let fc_stdout = String::from_utf8(fc_out.stdout).expect("Invalid path to libgcc.a");
        let fc_lib_cwd = PathBuf::from(fc_stdout.to_string());
        let fc_lib_pwd = fc_lib_cwd.parent().expect("Path to libgcc.a not found");
        println!("cargo:rustc-link-search={}", fc_lib_pwd.to_str().unwrap());
        // println!("cargo:rustc-link-search=/opt/homebrew/Cellar/gcc/13.2.0/lib/gcc/current/gcc/aarch64-apple-darwin23/13");
        // println!("cargo:rustc-link-search=/opt/homebrew/Cellar/gcc/13.2.0/lib/gcc/current");
    }

    println!("cargo:rustc-link-lib={}=lbfgs", kind);
    println!("cargo:rustc-link-lib=dylib=gcc");

    if target == "x86_64-apple-darwin" || target == "x86_64-pc-windows-gnu" {
        fc_lib_type = "static";

        // Poke $FC$ for static lib folder
        let fc_out = Command::new(variable!("FC"))
            .arg("-print-file-name=libgfortran.a")
            .output()
            .expect("Failed to find libgfortran.a");
        let fc_stdout = String::from_utf8(fc_out.stdout).expect("Invalid path to libgfortran.a");
        let fc_lib_cwd = PathBuf::from(fc_stdout.to_string());
        let fc_lib_pwd = fc_lib_cwd
            .parent()
            .expect("Path to libgfortran.a not found");
        println!("cargo:rustc-link-search={}", fc_lib_pwd.to_str().unwrap());
    }

    println!("cargo:rustc-link-lib={}=gfortran", fc_lib_type);

    if target == "x86_64-apple-darwin" {
        println!("cargo:rustc-link-lib={}=quadmath", fc_lib_type);
    }
}
fn run(command: &mut Command) {
    println!("Running: {:?}", command);
    match command.status() {
        Ok(status) => {
            if !status.success() {
                panic!("`{:?}` failed: {}", command, status);
            }
        }
        Err(error) => {
            panic!("failed to execute `{:?}`: {}", command, error);
        }
    }
}
