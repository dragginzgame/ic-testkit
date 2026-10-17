use super::StrictReserved;
use crate::{DecodeLimits, decode_exactly_one};
use candid::CandidType;
use serde::Deserialize;

#[test]
fn reserved_encoding_and_zero_skip_admission_match_candid() {
    let wire = candid::encode_one(StrictReserved).unwrap();
    assert_eq!(wire, candid::encode_one(candid::Reserved).unwrap());
    let limits = DecodeLimits {
        reply_bytes: 1024,
        decoding_quota: 10_000,
        skipping_quota: 0,
        type_entries: 4,
        header_bytes: 128,
    };
    assert!(decode_exactly_one::<StrictReserved>(&wire, limits).is_ok());
    for wire in [
        candid::encode_one(7_u64).unwrap(),
        candid::encode_one(vec![3_u8]).unwrap(),
    ] {
        assert!(decode_exactly_one::<StrictReserved>(&wire, limits).is_err());
    }
}

#[test]
fn recognized_variant_tags_do_not_hide_arbitrary_payloads() {
    #[derive(CandidType, Deserialize)]
    enum Strict {
        #[serde(rename = "source")]
        Source(StrictReserved),
    }
    #[derive(CandidType, Deserialize)]
    enum Untrusted {
        #[serde(rename = "source")]
        Source(u64),
    }
    let limits = DecodeLimits {
        reply_bytes: 1024,
        decoding_quota: 10_000,
        skipping_quota: 0,
        type_entries: 4,
        header_bytes: 128,
    };
    let valid = candid::encode_one(Strict::Source(StrictReserved)).unwrap();
    assert!(decode_exactly_one::<Strict>(&valid, limits).is_ok());
    let invalid = candid::encode_one(Untrusted::Source(7)).unwrap();
    assert!(decode_exactly_one::<Strict>(&invalid, limits).is_err());
}
