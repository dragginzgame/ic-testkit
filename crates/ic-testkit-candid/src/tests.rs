use super::{DecodeError, DecodeLimits, decode_exactly_one, decode_one};
use std::error::Error;

fn limits(bytes: usize) -> DecodeLimits {
    DecodeLimits {
        reply_bytes: 2 * 1024 * 1024,
        decoding_quota: bytes
            .saturating_mul(32)
            .saturating_add(1_000_000)
            .min(256_000_000),
        skipping_quota: 100_000,
        type_entries: 4_096,
        header_bytes: 64 * 1_024,
    }
}

#[test]
fn ordinary_and_exact_result_policies_are_explicit() {
    let text = "x".repeat(1024 * 1024);
    let bytes = candid::encode_one(&text).unwrap();
    assert_eq!(
        decode_exactly_one::<String>(&bytes, limits(bytes.len())).unwrap(),
        text
    );
    let bytes = candid::encode_args((42_u64, 7_u64)).unwrap();
    assert_eq!(decode_one::<u64>(&bytes, limits(bytes.len())).unwrap(), 42);
    assert!(matches!(
        decode_exactly_one::<u64>(&bytes, limits(bytes.len())),
        Err(DecodeError::Candid(_))
    ));
}

#[test]
fn refusals_preserve_original_bytes_and_decoder_causes() {
    let mut trailing = candid::encode_one(42_u64).unwrap();
    trailing.push(0);
    let mut truncated = candid::encode_one(42_u64).unwrap();
    truncated.pop();
    for bytes in [trailing, truncated, b"DIDL\x81\x20".to_vec()] {
        let original = bytes.clone();
        let error = decode_exactly_one::<u64>(&bytes, limits(bytes.len())).unwrap_err();
        assert!(matches!(error, DecodeError::Candid(_)));
        assert!(error.source().is_some());
        assert_eq!(bytes, original);
    }
    let bytes = candid::encode_one(42_u64).unwrap();
    let mut ceiling = limits(bytes.len());
    ceiling.reply_bytes = bytes.len() - 1;
    assert!(matches!(
        decode_one::<u64>(&bytes, ceiling),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
}

#[test]
fn caller_work_and_header_limits_cannot_be_bypassed() {
    let bytes = candid::encode_one(vec![(); 1_000_000]).unwrap();
    assert!(bytes.len() < 32);
    assert!(matches!(
        decode_one::<Vec<()>>(&bytes, limits(bytes.len())),
        Err(DecodeError::Candid(_))
    ));
    let bytes = candid::encode_args((42_u64, vec![(); 100_000])).unwrap();
    assert!(bytes.len() < 32);
    assert!(matches!(
        decode_one::<u64>(&bytes, limits(bytes.len())),
        Err(DecodeError::Candid(_))
    ));
    let bytes = candid::encode_one(42_u64).unwrap();
    for selected in [
        DecodeLimits {
            header_bytes: 0,
            ..limits(bytes.len())
        },
        DecodeLimits {
            decoding_quota: 0,
            ..limits(bytes.len())
        },
    ] {
        assert!(matches!(
            decode_one::<u64>(&bytes, selected),
            Err(DecodeError::Candid(_))
        ));
    }
}
