
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
        println!("cargo:rustc-link-lib=curl-impersonate");
    }
}
