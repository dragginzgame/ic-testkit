use super::{DecodeError, DecodeLimits, decode_args, decode_exactly_one, decode_one};
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
fn exact_result_policy_requires_one_wire_argument() {
    let empty = candid::encode_args(()).unwrap();
    assert!(matches!(
        decode_exactly_one::<Option<u64>>(&empty, limits(empty.len())),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        decode_exactly_one::<()>(&empty, limits(empty.len())),
        Err(DecodeError::Candid(_))
    ));
    assert_eq!(
        decode_one::<Option<u64>>(&empty, limits(empty.len())).unwrap(),
        None
    );
    decode_one::<()>(&empty, limits(empty.len())).unwrap();

    let optional = candid::encode_one(None::<u64>).unwrap();
    assert_eq!(
        decode_exactly_one::<Option<u64>>(&optional, limits(optional.len())).unwrap(),
        None
    );
    let null = candid::encode_one(()).unwrap();
    decode_exactly_one::<()>(&null, limits(null.len())).unwrap();
}

#[test]
fn argument_lists_preserve_wire_arity_and_wide_values() {
    let empty = candid::encode_args(()).unwrap();
    decode_args::<()>(&empty, limits(empty.len())).unwrap();
    let single = candid::encode_args((7_u64,)).unwrap();
    assert_eq!(
        decode_args::<(u64,)>(&single, limits(single.len())).unwrap(),
        (7,)
    );
    let values = (u128::MAX, u128::from(u64::MAX) + 1);
    let args = candid::encode_args(values).unwrap();
    assert_eq!(
        decode_args::<(u128, u128)>(&args, limits(args.len())).unwrap(),
        values
    );
    assert!(decode_one::<(u128, u128)>(&args, limits(args.len())).is_err());
    let tuple = candid::encode_one(values).unwrap();
    assert!(decode_args::<(u128, u128)>(&tuple, limits(tuple.len())).is_err());
    assert_eq!(
        decode_args::<((u128, u128),)>(&tuple, limits(tuple.len())).unwrap(),
        (values,)
    );
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
        let error = decode_args::<(u64,)>(&bytes, limits(bytes.len())).unwrap_err();
        assert!(matches!(error, DecodeError::Candid(_)));
        assert!(error.source().is_some());
        assert_eq!(bytes, original);
    }
    let bytes = candid::encode_one(42_u64).unwrap();
    let mut ceiling = limits(bytes.len());
    ceiling.reply_bytes = bytes.len() - 1;
    assert!(matches!(
        decode_one::<u64>(&bytes, ceiling),
        Err(DecodeError::ReplyTooLarge { actual, maximum })
            if actual == bytes.len() && maximum == bytes.len() - 1
    ));
    assert!(matches!(
        decode_exactly_one::<u64>(&bytes, ceiling),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
    assert!(matches!(
        decode_args::<(u64,)>(&bytes, ceiling),
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
    assert!(matches!(
        decode_exactly_one::<Vec<()>>(&bytes, limits(bytes.len())),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        decode_args::<(Vec<()>,)>(&bytes, limits(bytes.len())),
        Err(DecodeError::Candid(_))
    ));
    let bytes = candid::encode_args((42_u64, vec![(); 100_000])).unwrap();
    assert!(bytes.len() < 32);
    assert!(matches!(
        decode_one::<u64>(&bytes, limits(bytes.len())),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        decode_args::<(u64,)>(&bytes, limits(bytes.len())),
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
        assert!(matches!(
            decode_exactly_one::<u64>(&bytes, selected),
            Err(DecodeError::Candid(_))
        ));
        assert!(matches!(
            decode_args::<(u64,)>(&bytes, selected),
            Err(DecodeError::Candid(_))
        ));
    }
    let bytes = candid::encode_one(vec![42_u64]).unwrap();
    let selected = DecodeLimits {
        type_entries: 0,
        ..limits(bytes.len())
    };
    assert!(matches!(
        decode_one::<Vec<u64>>(&bytes, selected),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        decode_exactly_one::<Vec<u64>>(&bytes, selected),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        decode_args::<(Vec<u64>,)>(&bytes, selected),
        Err(DecodeError::Candid(_))
    ));
}
