use cmake::Config;
use std::{env, fs, path::PathBuf};

fn copy_dir(src: &PathBuf, dst: &PathBuf) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dst_path);
        } else {
            fs::copy(entry.path(), &dst_path).unwrap();
        }
    }
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // EPANET's CMake generates epanet_output_export.h and copies it back into
    // CMAKE_CURRENT_SOURCE_DIR/include. That write would fail when building from
    // the crates.io registry, where the source tree is read-only. Copy the EPANET
    // source into OUT_DIR first so all CMake-generated files land in a writable location.
    let epanet_out = out_path.join("EPANET");
    copy_dir(&PathBuf::from("EPANET"), &epanet_out);

    let dynamic_link = cfg!(feature = "dynamic-link");

    let mut cmake_cfg = Config::new(&epanet_out);
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

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if !dynamic_link {
        // Match the cmake define so bindgen generates bindings without dllimport
        builder = builder.clang_arg("-DDLLEXPORT=");
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
