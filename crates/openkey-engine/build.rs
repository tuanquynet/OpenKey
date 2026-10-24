use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let engine_dir = manifest_dir.join("../../Sources/OpenKey/engine");
    let cpp_ffi = manifest_dir.join("tests/cpp_ffi.cpp");

    println!("cargo:rerun-if-changed={}", cpp_ffi.display());
    println!("cargo:rerun-if-changed={}", engine_dir.join("Engine.cpp").display());
    println!("cargo:rerun-if-changed={}", engine_dir.join("Vietnamese.cpp").display());
    println!("cargo:rerun-if-changed={}", engine_dir.join("Macro.cpp").display());
    println!("cargo:rerun-if-changed={}", engine_dir.join("SmartSwitchKey.cpp").display());
    println!("cargo:rerun-if-changed={}", engine_dir.join("ConvertTool.cpp").display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-Wno-deprecated-declarations")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-reorder")
        .include(&engine_dir)
        .file(engine_dir.join("Engine.cpp"))
        .file(engine_dir.join("Vietnamese.cpp"))
        .file(engine_dir.join("Macro.cpp"))
        .file(engine_dir.join("SmartSwitchKey.cpp"))
        .file(engine_dir.join("ConvertTool.cpp"))
        .file(&cpp_ffi)
        .cargo_metadata(false)
        .compile("openkey_cpp_ref");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
}
