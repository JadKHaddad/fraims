//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run owned
//! ```

#![no_main]

use libfuzzer_sys::fuzz_target;

// TODO
fuzz_target!(|data: &[u8]| {});
