use std::{env, path::PathBuf};

fn main() {
    use build_script_cfg::Cfg;
    use search_cuda_tools::find_cuda_root;

    println!("cargo:rereun-if-changed=build.rs");

    let nvidia = Cfg::new("nvidia");
    let Some(mx_home) = find_cuda_root() else {
        return;
    };
    nvidia.define();
    println!(
        "cargo:rustc-link-search=native={}",
        mx_home.join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=hcruntime");
    println!("cargo:rustc-link-lib=dylib=htc-runtime64");

    println!("cargo-rerun-if-changed=wrapper.h");

    println!("{}", format!("-I{}", mx_home.join("include").display()));
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", mx_home.join("include").display()))
        .clang_arg("-x")
        .clang_arg("c++")
        .allowlist_item("hc.*")
        .allowlist_item("HC.*")
        .allowlist_function("hcrtc.*")
        .must_use_type("hcError_t")
        .must_use_type("hcrtcResult")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
