use std::fs;

#[cfg(not(feature = "mock"))]
fn prepare_link_library(library_directory: &std::path::Path) {
    println!(
        "cargo:rustc-link-search=native={}",
        library_directory.display()
    );
    println!("cargo:rustc-link-lib=curl-impersonate");
}

#[cfg(not(feature = "mock"))]
fn prepare_runtime_library(library_directory: &std::path::Path) {
    let target_directory =
        build_support::target_directory().expect("failed to determine Cargo target directory");
    let target = build_support::LibcurlTarget::detect_host().expect("unsupported host platform");
    let runtime_library = target_directory.join(build_support::runtime_library_name(target));
    fs::copy(
        library_directory.join(build_support::runtime_library_name(target)),
        &runtime_library,
    )
    .expect("failed to copy libcurl-impersonate runtime library");

    target
    .set_impersonate_runtime_library_path(&runtime_library)
    .expect("failed to configure runtime library path");

    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}

fn main() {
    #[cfg(not(feature = "mock"))]
    {
        println!("cargo:rerun-if-env-changed=LIBCURL_IMPERSONATE_DIR");

        let library_directory = build_support::find_or_install_library()
            .expect("failed to install libcurl-impersonate");

        prepare_link_library(&library_directory);
        prepare_runtime_library(&library_directory);
    }
}
