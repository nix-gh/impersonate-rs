fn main() {
    // Only link if not mocking
    #[cfg(not(feature = "mock"))]
    {
        println!("cargo:rerun-if-env-changed=LIBCURL_IMPERSONATE_DIR");

        let library_directory = build_support::find_library()
            .expect("libcurl-impersonate-chrome was not found");

        println!(
            "cargo:rustc-link-search=native={}",
            library_directory.display()
        );
        println!("cargo:rustc-link-lib=curl-impersonate-chrome");

        println!(
            "cargo:warning=Expecting libcurl-impersonate v{}",
            build_support::LIBCURL_IMPERSONATE_VERSION
        );
    }
}
