# build-support

Internal utilities shared by the workspace build tooling.

This crate contains logic used by the root `build.rs`, which verifies that a compatible `libcurl-impersonate` installation is available;

Keeping this logic in a dedicated crate ensures there is a single implementation for:

- the supported `libcurl-impersonate` version;
- platform detection;
- installation discovery;
- version verification.

This crate is an implementation detail and is not intended for external use.