//! Bounded Candid sequences without allocating from untrusted size hints.

use serde::{Deserialize, Deserializer, de};
use std::{fmt, marker::PhantomData};

#[cfg(test)]
mod tests;

/// Deserialize at most `LIMIT` typed elements using caller-selected diagnostics.
///
/// Rejects an excessive size hint before reading elements, never reserves from a
/// hint, and decodes one excess element by type when a hint is absent or too small.
/// Unlike persisted-format visitors, this does not request `IgnoredAny`, which
/// Candid cannot deserialize through its ordinary typed sequence path. Callers
/// retain wire/work quotas and any aggregate or application-specific bounds.
///
/// # Errors
/// Returns the caller's overflow diagnostic or the original element/shape error.
pub fn deserialize<'de, D, T, const LIMIT: usize>(
    deserializer: D,
    expected: impl fmt::Display,
    overflow: impl fmt::Display,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T, X, E, const LIMIT: usize> {
        expected: X,
        overflow: E,
        item: PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>, X: fmt::Display, E: fmt::Display, const LIMIT: usize>
        de::Visitor<'de> for Visitor<T, X, E, LIMIT>
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}", self.expected)
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            if sequence.size_hint().is_some_and(|size| size > LIMIT) {
                return Err(de::Error::custom(self.overflow));
            }
            let mut entries = Vec::new();
            while entries.len() < LIMIT {
                match sequence.next_element()? {
                    Some(entry) => entries.push(entry),
                    None => return Ok(entries),
                }
            }
            if sequence.next_element::<T>()?.is_some() {
                return Err(de::Error::custom(self.overflow));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(Visitor::<T, _, _, LIMIT> {
        expected,
        overflow,
        item: PhantomData,
    })
}
