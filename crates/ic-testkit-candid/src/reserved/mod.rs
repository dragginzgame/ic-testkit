//! Strict Candid reserved-marker admission.

use candid::CandidType;
use serde::{Deserialize, Deserializer, de};
use std::fmt;

#[cfg(test)]
mod tests;

/// A Candid `reserved` value that refuses arbitrary payload skipping.
///
/// Candid's ordinary `Reserved` accepts any wire value. This visitor accepts only
/// the decoder's exact owned reserved marker (`[3]`), retaining zero-skipping
/// admission for strict reply schemas. Select a zero skipping quota through
/// [`crate::DecodeLimits`] to reject incompatible wire payloads before this marker.
/// This is a codec boundary tied to Candid's marker representation; callers still
/// own variant tags, schema evolution and decoder work limits.
pub struct StrictReserved;

impl CandidType for StrictReserved {
    fn _ty() -> candid::types::Type {
        candid::types::TypeInner::Reserved.into()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_null(())
    }
}

impl<'de> Deserialize<'de> for StrictReserved {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct ReservedVisitor;
        impl de::Visitor<'_> for ReservedVisitor {
            type Value = StrictReserved;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("the Candid reserved marker")
            }
            fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
                if bytes == [3] {
                    Ok(StrictReserved)
                } else {
                    Err(E::custom("invalid reserved marker"))
                }
            }
        }
        d.deserialize_any(ReservedVisitor)
    }
}
