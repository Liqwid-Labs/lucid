use cbor_event::de::Deserializer;
use cbor_event::Len;
use std::io::{BufRead, Seek};

use super::*;

// Note: adapted from the upstream definition to use our version of cbor_events:
// https://github.com/dcSpark/cardano-multiplatform-lib/blob/45286430657145b916ec21bd0b83efb48b9aa4ff/chain/rust/src/utils.rs#L675-L684
// Used only for deserializing tagged sets after Plomin.
#[derive(Debug, Clone)]
pub struct NonemptySet<T> {
    pub(crate) elems: Vec<T>,
    pub(crate) len_encoding: Len,
}

impl <T> From<NonemptySet<T>> for Vec<T> {
    fn from(set: NonemptySet<T>) -> Self {
        set.elems
    }
}

impl<T: Deserialize> Deserialize for NonemptySet<T> {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let mut elems = Vec::new();
            if raw.cbor_type()? == cbor_event::Type::Tag {
                let tag = raw.tag()?;
                if tag != 258 {
                    return Err(DeserializeFailure::TagMismatch {
                        found: tag,
                        expected: 258
                    }
                    .into());
                }
            }
            let arr_len = raw.array()?;
            while match arr_len {
                cbor_event::Len::Len(n) => (elems.len() as u64) < n,
                cbor_event::Len::Indefinite => true,
            } {
                if raw.cbor_type()? == cbor_event::Type::Special {
                    assert_eq!(raw.special()?, cbor_event::Special::Break);
                    break;
                }
                let elem = T::deserialize(raw)?;
                elems.push(elem);
            }
            Ok(Self {
                elems,
                len_encoding: arr_len,
            })
        })()
        .map_err(|e| e.annotate("NonemptySet"))
    }
}


