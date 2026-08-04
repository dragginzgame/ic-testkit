//! Reusable PocketIC-oriented test utilities for IC canister tests.
//!
//! This crate is intended for host-side test environments (for example via
//! PocketIC) and provides generic helpers such as stable dummy principals,
//! PocketIC re-exports, standalone canister fixtures, generic prebuilt wasm
//! install helpers, retry helpers for PocketIC install throttling, and cached
//! baseline primitives.

pub mod benchmark;

#[cfg(not(target_arch = "wasm32"))]
pub mod artifacts;

#[cfg(not(target_arch = "wasm32"))]
pub mod pic;

pub mod performance;
use candid::Principal;

///
/// Deterministic dummy-value generator for tests.
///
/// Produces stable principals derived from a numeric seed, which makes tests
/// reproducible without hardcoding raw byte arrays.
///

pub struct Fake;

impl Fake {
    ///
    /// Deterministically derive a [`Principal`] from `seed`.
    ///
    #[must_use]
    pub fn principal(seed: u32) -> Principal {
        let mut buf = [0u8; 29];
        buf[..4].copy_from_slice(&seed.to_be_bytes());

        Principal::from_slice(&buf)
    }
}

///
/// TESTS
///

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_principal_is_deterministic_and_unique() {
        let p1 = Fake::principal(7);
        let p2 = Fake::principal(7);
        let q = Fake::principal(8);

        assert_eq!(p1, p2, "Fake::principal should be deterministic");
        assert_ne!(p1, q, "Fake::principal should differ for different seeds");

        let bytes = p1.as_slice();
        assert_eq!(bytes.len(), 29, "Principal must be 29 bytes");
    }
}
