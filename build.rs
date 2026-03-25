use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    let dynamic_link = cfg!(feature = "dynamic-link");

    let mut cmake_cfg = Config::new("EPANET");
    cmake_cfg.define("CMAKE_BUILD_TYPE", "Release");

    if !dynamic_link {
        cmake_cfg.define("BUILD_SHARED_LIBS", "OFF");
        // Pre-define DLLEXPORT as empty to prevent __declspec(dllimport) on Windows.
        // Both epanet2.h and epanet2_2.h guard with #ifndef DLLEXPORT, so this skips
        // their platform-specific definitions entirely.
        if cfg!(target_os = "windows") {
            cmake_cfg.cflag("/DDLLEXPORT=");
        } else {
            cmake_cfg.cflag("-DDLLEXPORT=");
        }
    }

    // EPANET C code uses math functions (pow, sqrt, etc.) from libm.
    // On Unix, the shared library needs libm linked in so the dynamic linker
    // can resolve these symbols at runtime.
    if cfg!(target_family = "unix") && dynamic_link {
        cmake_cfg.define("CMAKE_C_STANDARD_LIBRARIES", "-lm");
    }

    let dst = cmake_cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    println!("cargo:rustc-link-search=native={}", dst.display());

    if dynamic_link {
        println!("cargo:rustc-link-lib=dylib=epanet2");
    } else {
        println!("cargo:rustc-link-lib=static=epanet2");
    }

    // EPANET C code uses math functions (pow, sqrt, etc.)
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=m");
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if !dynamic_link {
        // Match the cmake define so bindgen generates bindings without dllimport
        builder = builder.clang_arg("-DDLLEXPORT=");
    }

    let bindings = builder
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
