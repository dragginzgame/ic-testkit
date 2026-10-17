use super::*;
use ic_host_artifacts::wasm::{InspectionLimits, inspect};

const LIMITS: RawReplyLimits = RawReplyLimits {
    methods: 2,
    reply_bytes: 65_537,
    method_name_bytes: 100,
};

#[test]
fn exports_both_modes_preserving_raw_binary_data_and_page_boundaries() {
    let reply = vec![0xff; 65_537];
    let methods = [
        RawReplyMethod {
            name: "read\"\\é",
            mode: ReplyMode::Query,
            reply: &reply,
        },
        RawReplyMethod {
            name: "read",
            mode: ReplyMode::Update,
            reply: b"not candid\0",
        },
    ];
    let wasm = raw_reply_canister(
        &methods,
        RawReplyLimits {
            reply_bytes: reply.len() + 11,
            ..LIMITS
        },
    )
    .unwrap();
    wasmparser::Validator::new().validate_all(&wasm).unwrap();
    let facts = inspect(
        &wasm,
        InspectionLimits {
            module_bytes: 100_000,
            sections: 8,
            exports: 3,
            custom_sections: 0,
        },
    )
    .unwrap();
    assert!(facts.exports.contains_key("canister_query read\"\\é"));
    assert!(facts.exports.contains_key("canister_update read"));
    assert_eq!(facts.defined_functions, 2);
    assert_eq!(facts.imported_functions, 2);
    assert_eq!(facts.data_segments, 2);
    assert!(wasm.windows(reply.len()).any(|bytes| bytes == reply));
    assert!(wasm.windows(11).any(|bytes| bytes == b"not candid\0"));
}

#[test]
fn permits_same_name_across_modes_empty_replies_and_an_empty_catalog() {
    let methods = [
        RawReplyMethod {
            name: "same",
            mode: ReplyMode::Query,
            reply: &[],
        },
        RawReplyMethod {
            name: "same",
            mode: ReplyMode::Update,
            reply: &[],
        },
    ];
    for selected in [&methods[..], &[][..]] {
        let wasm = raw_reply_canister(selected, LIMITS).unwrap();
        wasmparser::Validator::new().validate_all(&wasm).unwrap();
        inspect(
            &wasm,
            InspectionLimits {
                module_bytes: 100_000,
                sections: 8,
                exports: 3,
                custom_sections: 0,
            },
        )
        .unwrap();
    }
}

#[test]
fn rejects_names_duplicates_and_each_aggregate_limit() {
    let method = RawReplyMethod {
        name: "method",
        mode: ReplyMode::Update,
        reply: b"raw",
    };
    assert_eq!(
        raw_reply_canister(&[method, method], LIMITS),
        Err(RawReplyError::DuplicateExport { index: 1 })
    );
    for name in ["", "bad\0name", "bad\nname"] {
        assert!(matches!(
            raw_reply_canister(&[RawReplyMethod { name, ..method }], LIMITS),
            Err(RawReplyError::InvalidName { index: 0 })
        ));
    }
    for limits in [
        RawReplyLimits {
            methods: 0,
            ..LIMITS
        },
        RawReplyLimits {
            reply_bytes: 2,
            ..LIMITS
        },
        RawReplyLimits {
            method_name_bytes: 5,
            ..LIMITS
        },
    ] {
        assert!(matches!(
            raw_reply_canister(&[method], limits),
            Err(RawReplyError::Budget { .. })
        ));
    }
    let pair = [
        method,
        RawReplyMethod {
            mode: ReplyMode::Query,
            ..method
        },
    ];
    for limits in [
        RawReplyLimits {
            reply_bytes: 5,
            ..LIMITS
        },
        RawReplyLimits {
            method_name_bytes: 11,
            ..LIMITS
        },
    ] {
        assert!(matches!(
            raw_reply_canister(&pair, limits),
            Err(RawReplyError::Budget { .. })
        ));
    }
    assert!(matches!(
        charge(usize::MAX, 1, usize::MAX, "test"),
        Err(RawReplyError::Budget { .. })
    ));
}
