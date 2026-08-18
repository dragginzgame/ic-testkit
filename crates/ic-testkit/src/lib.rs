//! Focused PocketIC test-harness utilities for Internet Computer canisters.
//!
//! `ic-testkit` keeps PocketIC itself visible: [`pic`] re-exports the upstream
//! `PocketIc` and `PocketIcBuilder` types and adds extension traits for typed
//! Candid calls, generic installation, diagnostics, snapshots, startup errors,
//! caller-owned managed-server startup, and a small time conversion. It does
//! not provide a simulator wrapper or a host-wide runtime lock.
//!
//! The crate also provides:
//!
//! - host-only transactional artifacts, Wasm builds, and freshness helpers in
//!   [`artifacts`];
//! - marker parsing, aggregation, comparison, and reports in [`benchmark`];
//! - canister-side marker emission in [`performance`];
//! - deterministic test principals through [`Fake`].
//!
//! The [`pic`] and [`artifacts`] modules are unavailable when compiling for
//! `wasm32`; benchmark data types and marker emission remain available to
//! canister code.

pub mod benchmark;

#[cfg(not(target_arch = "wasm32"))]
mod timing;

#[cfg(not(target_arch = "wasm32"))]
pub mod artifacts;

#[cfg(not(target_arch = "wasm32"))]
pub mod pic;

pub mod performance;
use candid::Principal;

/// Deterministic principal generator for tests.
///
/// Values are derived directly from a numeric seed, making fixtures stable
/// without embedding textual principal literals.
pub struct Fake;

impl Fake {
    /// Deterministically derive a [`Principal`] from `seed`.
    #[must_use]
    pub fn principal(seed: u32) -> Principal {
        let mut buf = [0u8; 29];
        buf[..4].copy_from_slice(&seed.to_be_bytes());

        Principal::from_slice(&buf)
    }
}

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
