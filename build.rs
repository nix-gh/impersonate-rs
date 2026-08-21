
use std::fs;

fn main() {
    #[cfg(not(feature = "mock"))]
    {
        println!("cargo:rerun-if-env-changed=LIBCURL_IMPERSONATE_DIR");

        let library_directory = build_support::find_or_install_library()
            .expect("failed to install libcurl-impersonate");

        println!(
            "cargo:rustc-link-search=native={}",
            library_directory.display()
        );

        let target_directory =
            build_support::target_directory().expect("failed to determine Cargo target directory");

        let runtime_library = target_directory.join("libcurl-impersonate.so.4");

        fs::copy(
            library_directory.join("libcurl-impersonate.so.4"),
            &runtime_library,
        )
        .expect("failed to copy libcurl-impersonate runtime library");

        println!("cargo:rustc-link-lib=curl-impersonate");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
