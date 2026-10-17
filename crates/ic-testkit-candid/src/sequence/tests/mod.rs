use super::deserialize;
use serde::de::{
    DeserializeSeed, SeqAccess,
    value::{Error, SeqAccessDeserializer, U64Deserializer},
};
use std::cell::Cell;

struct Sequence<'a> {
    values: &'a [u64],
    hint: Option<usize>,
    reads: &'a Cell<usize>,
}

impl<'de> SeqAccess<'de> for Sequence<'_> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        let Some((first, rest)) = self.values.split_first() else {
            return Ok(None);
        };
        self.values = rest;
        self.reads.set(self.reads.get() + 1);
        seed.deserialize(U64Deserializer::new(*first)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        self.hint
    }
}

fn read<const LIMIT: usize>(
    values: &[u64],
    hint: Option<usize>,
    reads: &Cell<usize>,
) -> Result<Vec<u8>, Error> {
    deserialize::<_, u8, LIMIT>(
        SeqAccessDeserializer::new(Sequence {
            values,
            hint,
            reads,
        }),
        "bounded bytes",
        "sequence limit",
    )
}

#[test]
fn exact_and_zero_bounds_preserve_values() {
    let reads = Cell::new(0);
    assert_eq!(read::<2>(&[0, 255], Some(2), &reads).unwrap(), [0, 255]);
    assert_eq!(reads.get(), 2);
    assert_eq!(read::<0>(&[], None, &reads).unwrap(), [] as [u8; 0]);
    assert_eq!(
        read::<0>(&[1], None, &reads).unwrap_err().to_string(),
        "sequence limit"
    );
}

#[test]
fn excessive_hints_reject_before_reading_or_allocating_from_them() {
    let reads = Cell::new(0);
    assert_eq!(
        read::<2>(&[1], Some(usize::MAX), &reads)
            .unwrap_err()
            .to_string(),
        "sequence limit"
    );
    assert_eq!(reads.get(), 0);
}

#[test]
fn absent_or_deceptive_hints_read_only_one_typed_excess_element() {
    for hint in [None, Some(0), Some(2)] {
        let reads = Cell::new(0);
        assert_eq!(
            read::<2>(&[1, 2, 3, 4], hint, &reads)
                .unwrap_err()
                .to_string(),
            "sequence limit"
        );
        assert_eq!(reads.get(), 3);
        reads.set(0);
        let error = read::<2>(&[1, 2, u64::MAX, 4], hint, &reads).unwrap_err();
        assert!(error.to_string().contains("invalid value"));
        assert_eq!(reads.get(), 3);
    }
}

#[test]
fn candid_sequences_keep_explicit_work_and_zero_skip_limits() {
    use candid::CandidType;
    use serde::{Deserialize, Deserializer};
    fn bytes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        deserialize::<_, _, 2>(d, "bounded bytes", "sequence limit")
    }
    #[derive(CandidType, Deserialize)]
    struct Bounded {
        #[serde(deserialize_with = "bytes")]
        bytes: Vec<u8>,
    }
    let limits = crate::DecodeLimits {
        reply_bytes: 1024,
        decoding_quota: 10_000,
        skipping_quota: 0,
        type_entries: 4,
        header_bytes: 128,
    };
    let wire = candid::encode_one(Bounded {
        bytes: vec![0, 255],
    })
    .unwrap();
    assert_eq!(
        crate::decode_exactly_one::<Bounded>(&wire, limits)
            .unwrap()
            .bytes,
        [0, 255]
    );
    let wire = candid::encode_one(Bounded {
        bytes: vec![0, 1, 2],
    })
    .unwrap();
    assert!(crate::decode_exactly_one::<Bounded>(&wire, limits).is_err());
}
