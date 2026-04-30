//! ETNA benchmark harness.
//!
//! Defines the framework-neutral `PropertyResult` enum plus one `property_*`
//! function per mined bug. Every framework adapter in `src/bin/etna.rs` and
//! every witness test calls into these functions.

#![allow(missing_docs)]

use crate::accumulator::CobsAccumulator;
use crate::{from_bytes, to_stdvec, to_stdvec_cobs};

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// ---- i128_deserialization ----

/// Roundtrip an `i128` through serialize/deserialize. Pre-`70ea33a`,
/// `deserialize_i128` read 16 raw bytes instead of decoding the
/// zig-zag-encoded varint produced by `serialize_i128`, so any non-zero
/// value (especially negatives) failed to roundtrip.
pub fn property_i128_roundtrip(value: i128) -> PropertyResult {
    let mut buf = [0u8; 32];
    let used = match crate::to_slice(&value, &mut buf) {
        Ok(b) => b,
        Err(e) => return PropertyResult::Fail(format!("serialize failed: {e:?}")),
    };
    let owned = used.to_vec();
    match from_bytes::<i128>(&owned) {
        Ok(decoded) if decoded == value => PropertyResult::Pass,
        Ok(decoded) => PropertyResult::Fail(format!(
            "roundtrip mismatch: in={} out={} bytes={:?}",
            value, decoded, owned
        )),
        Err(e) => PropertyResult::Fail(format!("deserialize failed: {e:?} bytes={:?}", owned)),
    }
}

// ---- cobs_acc_oob ----

/// Feeding any COBS-encoded postcard frame into a `CobsAccumulator<N>` must
/// never panic, regardless of how `N` compares to the frame length. The
/// historical regression (pre-`41c2ddb`) was that when the encoded frame was
/// exactly `N + 1` bytes long, the in-bounds guard checked
/// `self.idx + n <= N` where `n` excluded the sentinel byte, so the check
/// passed by one byte and `extend_unchecked` then indexed out of bounds.
///
/// We pick fixed accumulator capacities at every byte from 1..=15 and feed a
/// payload sized to make the encoded frame exactly `cap + 1` bytes — the
/// exact panic-trigger from PR #90.
pub fn property_cobs_acc_no_oob(seed: u8) -> PropertyResult {
    // Pick a buffer capacity in 1..=15 deterministically from the seed.
    let cap = ((seed % 15) + 1) as usize;
    // For Vec<u8> with payload length `p`, postcard's COBS encoding is
    // `p + 3` bytes (1 byte varint length + payload + COBS overhead = offset
    // byte + sentinel). To make the frame exactly `cap + 1` bytes long we
    // want `p = cap - 2`. When cap < 2, the frame is at least 3 bytes >
    // cap, which still exercises the OverFull path safely (cannot panic
    // even on the buggy version because n is small).
    let payload_len = cap.saturating_sub(2);
    let payload = vec![0xCCu8; payload_len];
    let mut frame = match to_stdvec_cobs(&payload) {
        Ok(v) => v,
        Err(e) => return PropertyResult::Fail(format!("encode failed: {e:?}")),
    };
    match cap {
        1 => feed_typed::<1>(&mut frame),
        2 => feed_typed::<2>(&mut frame),
        3 => feed_typed::<3>(&mut frame),
        4 => feed_typed::<4>(&mut frame),
        5 => feed_typed::<5>(&mut frame),
        6 => feed_typed::<6>(&mut frame),
        7 => feed_typed::<7>(&mut frame),
        8 => feed_typed::<8>(&mut frame),
        9 => feed_typed::<9>(&mut frame),
        10 => feed_typed::<10>(&mut frame),
        11 => feed_typed::<11>(&mut frame),
        12 => feed_typed::<12>(&mut frame),
        13 => feed_typed::<13>(&mut frame),
        14 => feed_typed::<14>(&mut frame),
        _ => feed_typed::<15>(&mut frame),
    }
}

fn feed_typed<const N: usize>(frame: &mut [u8]) -> PropertyResult {
    let mut acc: CobsAccumulator<N> = CobsAccumulator::new();
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        acc.feed::<Vec<u8>>(frame)
    }));
    match res {
        Ok(_) => PropertyResult::Pass,
        Err(_) => PropertyResult::Fail(format!(
            "CobsAccumulator<{}>::feed panicked on {}-byte frame",
            N,
            frame.len()
        )),
    }
}

// ---- tuple_struct_deserialize ----

/// Roundtrip a `(u8, u16, u32)` tuple-struct. Pre-`2a62f8c`,
/// `deserialize_tuple_struct` discarded its `len` argument and called
/// `deserialize_seq`, which prepends a varint length when serializing but
/// not when deserializing as a tuple — so the byte stream gets misaligned
/// and the decoded value differs.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct TupleS(pub u8, pub u16, pub u32);

pub fn property_tuple_struct_roundtrip(value: (u8, u16, u32)) -> PropertyResult {
    let v = TupleS(value.0, value.1, value.2);
    let bytes = match to_stdvec(&v) {
        Ok(b) => b,
        Err(e) => return PropertyResult::Fail(format!("serialize: {e:?}")),
    };
    match from_bytes::<TupleS>(&bytes) {
        Ok(d) if d == v => PropertyResult::Pass,
        Ok(d) => PropertyResult::Fail(format!(
            "roundtrip mismatch: in={:?} out={:?} bytes={:?}",
            v, d, bytes
        )),
        Err(e) => PropertyResult::Fail(format!("deserialize: {e:?} bytes={:?}", bytes)),
    }
}

// ---- serialize_bytes_size_prefix ----

/// `Serializer::serialize_bytes` must emit a varint length prefix before the
/// payload, so consumers reading the stream know how many bytes to take.
/// Pre-`58b3047`, `serialize_bytes` wrote only the raw payload, breaking
/// roundtrip for `&serde_bytes::Bytes`, `CStr`, and any other type whose
/// `Serialize` impl uses `serialize_bytes`.
pub fn property_bytes_roundtrip(payload: Vec<u8>) -> PropertyResult {
    use serde::Serialize;
    // Bound payload size so each test stays fast.
    let payload: Vec<u8> = payload.into_iter().take(64).collect();
    let mut buf = vec![0u8; 256];
    let written = {
        struct Wrap<'a>(&'a [u8]);
        impl Serialize for Wrap<'_> {
            fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                ser.serialize_bytes(self.0)
            }
        }
        match crate::to_slice(&Wrap(&payload), &mut buf) {
            Ok(used) => used.len(),
            Err(e) => return PropertyResult::Fail(format!("serialize: {e:?}")),
        }
    };
    let bytes = &buf[..written];
    match from_bytes::<Vec<u8>>(bytes) {
        Ok(d) if d == payload => PropertyResult::Pass,
        Ok(d) => PropertyResult::Fail(format!(
            "roundtrip mismatch: in_len={} out_len={} bytes={:?}",
            payload.len(),
            d.len(),
            bytes
        )),
        Err(e) => PropertyResult::Fail(format!("deserialize: {e:?} bytes={:?}", bytes)),
    }
}

// ---- max_size_varint_zero ----

/// `MaxSize::POSTCARD_MAX_SIZE` for a zero-capacity `heapless::Vec<u8, 0>`
/// must include at least the one-byte varint length prefix that postcard
/// emits for the empty payload. Pre-`f31d226`, the helper `varint_size(0)`
/// returned 0, so the const reported the empty vec needing 0 bytes — wrong
/// against the actual on-the-wire encoding (a single `0x00` byte).
#[cfg(all(feature = "experimental-derive", feature = "heapless"))]
pub fn property_max_size_heapless_vec_zero(_: ()) -> PropertyResult {
    use crate::experimental::max_size::MaxSize;
    type V = heapless::Vec<u8, 0>;
    let v = V::new();
    let bytes = match to_stdvec(&v) {
        Ok(b) => b,
        Err(e) => return PropertyResult::Fail(format!("serialize: {e:?}")),
    };
    if (V::POSTCARD_MAX_SIZE) < bytes.len() {
        return PropertyResult::Fail(format!(
            "POSTCARD_MAX_SIZE = {}, but encoding needs {} bytes",
            V::POSTCARD_MAX_SIZE,
            bytes.len()
        ));
    }
    PropertyResult::Pass
}

// ---- max_size_varint_off_by_one ----

/// The `varint_size(n)` helper used by `MaxSize` impls must return at least
/// the number of bytes the runtime varint encoder writes for `n`. Pre-
/// `c160626`, the helper rounded up bits using `BITS_PER_BYTE - 1` instead of
/// `BITS_PER_VARINT_BYTE - 1`, so for many `n` it under-reported by one byte
/// and `<heapless::Vec<u8, N>>::POSTCARD_MAX_SIZE` was too small to hold the
/// real serialized output.
#[cfg(all(feature = "experimental-derive", feature = "heapless"))]
pub fn property_max_size_vec_upper_bound(payload_len: u16) -> PropertyResult {
    // Pick a representative N where the off-by-one bug bites.
    let n = (payload_len % 4) as usize;
    match n {
        0 => check_vec_max::<127>(),
        1 => check_vec_max::<128>(),
        2 => check_vec_max::<16383>(),
        _ => check_vec_max::<16384>(),
    }
}

#[cfg(all(feature = "experimental-derive", feature = "heapless"))]
fn check_vec_max<const N: usize>() -> PropertyResult {
    use crate::experimental::max_size::MaxSize;
    type Out = heapless::Vec<u8, 16400>;
    let mut v: heapless::Vec<u8, N> = heapless::Vec::new();
    for _ in 0..N {
        v.push(0u8).unwrap();
    }
    let bytes: Out = match crate::to_vec::<heapless::Vec<u8, N>, 16400>(&v) {
        Ok(b) => b,
        Err(e) => return PropertyResult::Fail(format!("serialize: {e:?}")),
    };
    let max = <heapless::Vec<u8, N> as MaxSize>::POSTCARD_MAX_SIZE;
    if max != bytes.len() {
        return PropertyResult::Fail(format!(
            "POSTCARD_MAX_SIZE for heapless::Vec<u8, {N}> = {max}, but encoded len = {}",
            bytes.len()
        ));
    }
    PropertyResult::Pass
}

// ---- collect_str_unreachable ----

/// Serializing a value via `Serializer::collect_str` must succeed and produce
/// a length-prefixed UTF-8 byte string, just like `serialize_str`. Pre-
/// `77fd54b`, `collect_str` was `unreachable!()`, panicking on any type whose
/// `Serialize` impl funnelled through it (the canonical case is types from
/// the `chrono` crate).
pub fn property_collect_str_roundtrip(seed: u8) -> PropertyResult {
    use serde::Serialize;

    // Build a Display-only wrapper that calls serialize_str via collect_str.
    // The text is constant-ish; vary tail with `seed` to keep input mass.
    let len = ((seed % 16) as usize) + 1;
    let s: String = (0..len).map(|i| (b'a' + ((i as u8) % 26)) as char).collect();

    struct ViaCollectStr<'a>(&'a str);
    impl Serialize for ViaCollectStr<'_> {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.collect_str(&self.0)
        }
    }

    let mut buf = vec![0u8; 256];
    let used_len = match crate::to_slice(&ViaCollectStr(&s), &mut buf) {
        Ok(used) => used.len(),
        Err(e) => return PropertyResult::Fail(format!("serialize: {e:?}")),
    };
    let bytes = &buf[..used_len];
    match from_bytes::<String>(bytes) {
        Ok(d) if d == s => PropertyResult::Pass,
        Ok(d) => PropertyResult::Fail(format!(
            "roundtrip mismatch: in={:?} out={:?} bytes={:?}",
            s, d, bytes
        )),
        Err(e) => PropertyResult::Fail(format!("deserialize: {e:?} bytes={:?}", bytes)),
    }
}
