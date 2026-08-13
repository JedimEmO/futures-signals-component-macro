// Compile-fail suite. Gated off wasm32 so the wasm-bindgen test runner configured in
// .cargo/config.toml never tries to execute it; under wasm this file compiles to an
// empty test crate.
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
