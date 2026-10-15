//! Caller-bounded Candid reply decoding, independent of PocketIC or transport.
//!
//! Callers select finite work limits, authenticate sources and retain original
//! bytes. Ordinary single-result decoding permits Candid's bounded skipping of
//! additional values; exact-single-result decoding rejects them.

use candid::{
    CandidType,
    de::{DecoderConfig, IDLDeserialize},
};
use serde::Deserialize;
use std::{error::Error, fmt};

#[cfg(test)]
mod tests;

/// Caller-selected wire, decoding, skipping, type-table and header ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum original reply bytes admitted before decoder construction.
    pub reply_bytes: usize,
    /// Maximum Candid decoding work; zero permits no charged work.
    pub decoding_quota: usize,
    /// Maximum work spent skipping values not requested by the caller.
    pub skipping_quota: usize,
    /// Maximum number of type-table entries in the wire header.
    pub type_entries: usize,
    /// Maximum wire bytes occupied by the Candid header.
    pub header_bytes: usize,
}

/// Wire admission or Candid decoding failure, retaining the original decoder cause.
#[derive(Debug)]
pub enum DecodeError {
    /// Original wire reply exceeds the selected byte ceiling.
    ReplyTooLarge {
        /// Observed original reply length.
        actual: usize,
        /// Caller-selected maximum reply length.
        maximum: usize,
    },
    /// Candid shape, work quota, truncation or exact-result admission failed.
    Candid(candid::Error),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReplyTooLarge { actual, maximum } => {
                write!(f, "Candid reply is {actual} bytes, maximum is {maximum}")
            }
            Self::Candid(error) => write!(f, "Candid reply decoding failed: {error}"),
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReplyTooLarge { .. } => None,
            Self::Candid(error) => Some(error),
        }
    }
}

fn config(bytes: &[u8], limits: DecodeLimits) -> Result<DecoderConfig, DecodeError> {
    if bytes.len() > limits.reply_bytes {
        return Err(DecodeError::ReplyTooLarge {
            actual: bytes.len(),
            maximum: limits.reply_bytes,
        });
    }
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(limits.decoding_quota)
        .set_skipping_quota(limits.skipping_quota)
        .set_max_type_len(limits.type_entries)
        .set_max_header_len(limits.header_bytes);
    Ok(config)
}

/// Decode one typed result with caller-bounded skipping of other Candid values.
///
/// # Errors
/// Refuses oversized replies and preserves Candid shape/quota failures.
pub fn decode_one<'a, T: Deserialize<'a> + CandidType>(
    bytes: &'a [u8],
    limits: DecodeLimits,
) -> Result<T, DecodeError> {
    candid::decode_one_with_config(bytes, &config(bytes, limits)?).map_err(DecodeError::Candid)
}

/// Decode exactly one typed Candid result, rejecting missing or additional values or bytes.
///
/// # Errors
/// Refuses wire/work ceilings, wrong result shape/arity and trailing bytes.
pub fn decode_exactly_one<'a, T: Deserialize<'a> + CandidType>(
    bytes: &'a [u8],
    limits: DecodeLimits,
) -> Result<T, DecodeError> {
    let config = config(bytes, limits)?;
    let mut decoder =
        IDLDeserialize::new_with_config(bytes, &config).map_err(DecodeError::Candid)?;
    if decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "expected exactly one Candid result",
        )));
    }
    let value = decoder.get_value::<T>().map_err(DecodeError::Candid)?;
    if !decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "expected exactly one Candid result",
        )));
    }
    decoder.done().map_err(DecodeError::Candid)?;
    Ok(value)
}
