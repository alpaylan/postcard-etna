# postcard — Injected Bugs

A no_std + serde compatible message library for Rust — ETNA workload.

Total mutations: 7

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `cobs_acc_oob_41c2ddb_1` | `cobs_acc_oob` | `source/postcard/src/accumulator.rs` | `patch` | `41c2ddbdfc42e08e8eb2aa539a2a46806e65f4df` |
| 2 | `collect_str_unreachable_77fd54b_1` | `collect_str_unreachable` | `source/postcard/src/ser/serializer.rs` | `patch` | `77fd54b18977e8491ebe1b67cc0d104e32ab02d6` |
| 3 | `i128_deserialization_70ea33a_1` | `i128_deserialization` | `source/postcard/src/de/deserializer.rs` | `patch` | `70ea33a1ac7f82632697f4578002267eaf9095f5` |
| 4 | `max_size_varint_off_by_one_c160626_1` | `max_size_varint_off_by_one` | `source/postcard/src/max_size.rs` | `patch` | `c160626b5e7dd8e9a95c140aed8972e8e2a14a39` |
| 5 | `max_size_varint_zero_f31d226_1` | `max_size_varint_zero` | `source/postcard/src/max_size.rs` | `patch` | `f31d2263ce37f9aae7413ab2a78b787f8214b89b` |
| 6 | `serialize_bytes_size_prefix_58b3047_1` | `serialize_bytes_size_prefix` | `source/postcard/src/ser/serializer.rs` | `patch` | `58b30476955e9fc9a76c3bc85930c407b7ae86c4` |
| 7 | `tuple_struct_deserialize_2a62f8c_1` | `tuple_struct_deserialize` | `source/postcard/src/de/deserializer.rs` | `patch` | `2a62f8c3cd643ccabea9034b71f9c9e97529f4e3` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `cobs_acc_oob_41c2ddb_1` | `CobsAccNoOob` | `witness_cobs_acc_no_oob_case_overrun_by_one`, `witness_cobs_acc_no_oob_case_overrun_by_one_b` |
| `collect_str_unreachable_77fd54b_1` | `CollectStrRoundtrip` | `witness_collect_str_roundtrip_case_short_a`, `witness_collect_str_roundtrip_case_short_b` |
| `i128_deserialization_70ea33a_1` | `I128Roundtrip` | `witness_i128_roundtrip_case_negative_large`, `witness_i128_roundtrip_case_positive_one`, `witness_i128_roundtrip_case_negative_one` |
| `max_size_varint_off_by_one_c160626_1` | `MaxSizeVecUpperBound` | `witness_max_size_vec_upper_bound_case_127`, `witness_max_size_vec_upper_bound_case_16383` |
| `max_size_varint_zero_f31d226_1` | `MaxSizeHeaplessVecZero` | `witness_max_size_heapless_vec_zero_case_default` |
| `serialize_bytes_size_prefix_58b3047_1` | `BytesRoundtrip` | `witness_bytes_roundtrip_case_short`, `witness_bytes_roundtrip_case_empty` |
| `tuple_struct_deserialize_2a62f8c_1` | `TupleStructRoundtrip` | `witness_tuple_struct_roundtrip_case_small`, `witness_tuple_struct_roundtrip_case_large` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `CobsAccNoOob` | ✓ | ✓ | ✓ | ✓ |
| `CollectStrRoundtrip` | ✓ | ✓ | ✓ | ✓ |
| `I128Roundtrip` | ✓ | ✓ | ✓ | ✓ |
| `MaxSizeVecUpperBound` | ✓ | ✓ | ✓ | ✓ |
| `MaxSizeHeaplessVecZero` | ✓ | ✓ | ✓ | ✓ |
| `BytesRoundtrip` | ✓ | ✓ | ✓ | ✓ |
| `TupleStructRoundtrip` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. cobs_acc_oob

- **Variant**: `cobs_acc_oob_41c2ddb_1`
- **Location**: `source/postcard/src/accumulator.rs` (inside `CobsAccumulator::feed`)
- **Property**: `CobsAccNoOob`
- **Witness(es)**:
  - `witness_cobs_acc_no_oob_case_overrun_by_one` — 11-byte buffer, 12-byte encoded frame — the exact regression scenario.
  - `witness_cobs_acc_no_oob_case_overrun_by_one_b`
- **Source**: [#90](https://github.com/jamesmunns/postcard/pull/90) — fix: cobs accumulator out-of-bounds index when data is 1 byte too long (#90)
  > `CobsAccumulator::feed` split each input chunk into `(take, release)` at the first sentinel and copied `take` into the internal buffer. The in-bounds guard checked `self.idx + n <= N`, where `n` excluded the sentinel byte that `take` actually contains; when the encoded frame was exactly `N + 1` bytes long the check passed by one byte and `extend_unchecked` indexed out of bounds and panicked. The fix uses `take.len()` instead of `n` so the bound matches the actual write.
- **Fix commit**: `41c2ddbdfc42e08e8eb2aa539a2a46806e65f4df` — fix: cobs accumulator out-of-bounds index when data is 1 byte too long (#90)
- **Invariant violated**: For every `N: usize` and every byte slice `frame`, `CobsAccumulator::<N>::new().feed::<T>(frame)` must complete without panicking — frames that exceed the buffer are reported as `FeedResult::OverFull`, never as a panic from out-of-bounds indexing.
- **How the mutation triggers**: The mutation restores the off-by-one bound `self.idx + n <= N`, where `n` is the chunk length excluding the sentinel byte. When the encoded frame is exactly `N + 1` bytes long, the bound passes but `extend_unchecked` then writes one byte past the buffer, panicking under debug bounds checks (and corrupting memory in release).

### 2. collect_str_unreachable

- **Variant**: `collect_str_unreachable_77fd54b_1`
- **Location**: `source/postcard/src/ser/serializer.rs` (inside `Serializer::collect_str`)
- **Property**: `CollectStrRoundtrip`
- **Witness(es)**:
  - `witness_collect_str_roundtrip_case_short_a`
  - `witness_collect_str_roundtrip_case_short_b`
- **Source**: internal report — Potential fix for collect_str bug
  > `Serializer::collect_str` was `unreachable!()`, so any type whose `Serialize` impl funnels through it (the canonical case is `chrono`'s timestamp types) panicked at runtime. The fix runs the formatter twice: once into a counting writer to learn the byte length, then once into the real flavor after writing a varint length prefix — matching the wire format of a regular `serialize_str`.
- **Fix commit**: `77fd54b18977e8491ebe1b67cc0d104e32ab02d6` — Potential fix for collect_str bug
- **Invariant violated**: Calling `Serializer::collect_str` for any `Display` value `v` must succeed and produce the same byte sequence as `serialize_str(&format!("{v}"))`: a varint length prefix followed by the UTF-8 bytes.
- **How the mutation triggers**: The mutation replaces the two-pass implementation with `unreachable!()`. Any serialize call that funnels through `collect_str` (e.g. a custom `Serialize` impl that calls `ser.collect_str(&self.0)`) panics at runtime instead of producing a length-prefixed string.

### 3. i128_deserialization

- **Variant**: `i128_deserialization_70ea33a_1`
- **Location**: `source/postcard/src/de/deserializer.rs` (inside `deserialize_i128`)
- **Property**: `I128Roundtrip`
- **Witness(es)**:
  - `witness_i128_roundtrip_case_negative_large` — Regression value lifted directly from the test added in the fix commit.
  - `witness_i128_roundtrip_case_positive_one`
  - `witness_i128_roundtrip_case_negative_one`
- **Source**: Fix deserialization of i128
  > `Deserializer::deserialize_i128` reached for 16 raw little-endian bytes, but the matching `serialize_i128` writes a zig-zag-encoded varint. Any non-zero (and especially negative) `i128` therefore failed to roundtrip — `from_bytes::<i128>(&to_stdvec(&x).unwrap())` produced a different value or returned an error. The fix replaces the raw read with `try_take_varint_u128` + a new `de_zig_zag_i128` helper to mirror the encoder.
- **Fix commit**: `70ea33a1ac7f82632697f4578002267eaf9095f5` — Fix deserialization of i128
- **Invariant violated**: For every `x: i128`, `from_bytes::<i128>(&to_stdvec(&x).unwrap()).unwrap() == x`. Postcard's wire format for signed integers is a zig-zag-encoded varint, and the decoder must mirror the encoder.
- **How the mutation triggers**: The mutation reverts `deserialize_i128` from `try_take_varint_u128` + `de_zig_zag_i128` back to a raw `try_take_n(16)` + `i128::from_le_bytes`. The serializer still emits varint+zig-zag, so the byte stream and decoder format disagree and roundtrip fails on every non-trivial value.

### 4. max_size_varint_off_by_one

- **Variant**: `max_size_varint_off_by_one_c160626_1`
- **Location**: `source/postcard/src/max_size.rs` (inside `varint_size`)
- **Property**: `MaxSizeVecUpperBound`
- **Witness(es)**:
  - `witness_max_size_vec_upper_bound_case_127` — bits=7; buggy varint_size returns 2 instead of 1.
  - `witness_max_size_vec_upper_bound_case_16383` — bits=14; buggy varint_size returns 3 instead of 2.
- **Source**: internal report — Add max size edge case tests and fix bug
  > The same `varint_size(max_n)` helper rounded up bits using `BITS_PER_BYTE - 1 = 7` instead of `BITS_PER_VARINT_BYTE - 1 = 6`. When `bits` is exactly a multiple of 7 (i.e. `max_n` lies in `[2^7k, 2^7k+1)` for `k >= 1`), this nudges the result one byte too high, so `<heapless::Vec<u8, N>>::POSTCARD_MAX_SIZE` over-estimates and breaks `assert_eq!(POSTCARD_MAX_SIZE, serialized.len())` for capacities like 127 and 16383.
- **Fix commit**: `c160626b5e7dd8e9a95c140aed8972e8e2a14a39` — Add max size edge case tests and fix bug
- **Invariant violated**: For a fully-filled `heapless::Vec<u8, N>`, `<heapless::Vec<u8, N>>::POSTCARD_MAX_SIZE == serialized_length(v)` — `MaxSize` is a tight upper bound, not just any upper bound. Specifically `POSTCARD_MAX_SIZE = N + varint_size(N)`, where `varint_size(N) = ceil(bits_needed(N) / 7)`.
- **How the mutation triggers**: The mutation flips the rounding constant in `varint_size` from `BITS_PER_VARINT_BYTE - 1` (= 6) back to `BITS_PER_BYTE - 1` (= 7). For `bits = 7` (`max_n` in 64..=127) the divisor turns 13/7 into 14/7, returning 2 instead of 1; the same happens at `bits = 14`. The const is now one byte larger than the runtime encoding, breaking the `MaxSize` invariant for capacities like 127 and 16383.

### 5. max_size_varint_zero

- **Variant**: `max_size_varint_zero_f31d226_1`
- **Location**: `source/postcard/src/max_size.rs` (inside `varint_size`)
- **Property**: `MaxSizeHeaplessVecZero`
- **Witness(es)**:
  - `witness_max_size_heapless_vec_zero_case_default`
- **Source**: internal report — Fix another bug when the capacity of a heapless::Vec or String is zero
  > The `varint_size(max_n)` helper that backs `<heapless::Vec<u8, N>>::POSTCARD_MAX_SIZE` returned `0` when `max_n == 0`, because `bits = 64 - leading_zeros(0) = 0` and the rounded-up division still produced 0. Postcard's runtime varint encoder writes a single `0x00` byte for a length of 0, so the const under-estimated by one for any zero-capacity collection. The fix adds an `if max_n == 0 { return 1; }` guard.
- **Fix commit**: `f31d2263ce37f9aae7413ab2a78b787f8214b89b` — Fix another bug when the capacity of a heapless::Vec or String is zero
- **Invariant violated**: For every type `T` whose serialized form starts with a varint length prefix, `<T as MaxSize>::POSTCARD_MAX_SIZE` must be at least the length of the actual encoding. In particular `<heapless::Vec<u8, 0>>::POSTCARD_MAX_SIZE >= 1`, because the empty vec encodes as the single varint byte `0x00`.
- **How the mutation triggers**: The mutation removes the `if max_n == 0 { return 1; }` early return. With `max_n == 0`, the formula computes `bits = 0`, `(0 + 6) / 7 = 0`, returning `varint_size(0) = 0`. `<heapless::Vec<u8, 0>>::POSTCARD_MAX_SIZE` then becomes 0, even though the runtime encoder emits a 1-byte `[0x00]`.

### 6. serialize_bytes_size_prefix

- **Variant**: `serialize_bytes_size_prefix_58b3047_1`
- **Location**: `source/postcard/src/ser/serializer.rs` (inside `Serializer::serialize_bytes`)
- **Property**: `BytesRoundtrip`
- **Witness(es)**:
  - `witness_bytes_roundtrip_case_short`
  - `witness_bytes_roundtrip_case_empty`
- **Source**: [#22](https://github.com/jamesmunns/postcard/pull/22) — Add missing size prefix for serialize_bytes (#22)
  > `Serializer::serialize_bytes` emitted only the raw payload bytes, but the deserializer for `&[u8]` / `Vec<u8>` reads a varint length prefix first. Any type whose `Serialize` impl funnels through `serialize_bytes` (`serde_bytes::Bytes`, `CStr`, `bytes::Bytes`) therefore failed to roundtrip — the decoder consumed a payload byte as the length. The fix prepends `try_push_varint_usize(v.len())` before extending the output.
- **Fix commit**: `58b30476955e9fc9a76c3bc85930c407b7ae86c4` — Add missing size prefix for serialize_bytes (#22)
- **Invariant violated**: For every byte payload `bs`, the bytes that `serialize_bytes(&bs)` writes to the output must be a varint-length-prefixed encoding of `bs`. Specifically, deserializing the resulting buffer as `Vec<u8>` (whose `Deserialize` impl reads `varint_usize` then that many bytes) must yield the original payload.
- **How the mutation triggers**: The mutation removes the `try_push_varint_usize(v.len())` call so `serialize_bytes` writes only the raw payload. The decoder still reads a varint length first, so it parses the first payload byte as the length and either truncates / over-reads the remainder, returning `DeserializeUnexpectedEnd` or a wrong-length result.

### 7. tuple_struct_deserialize

- **Variant**: `tuple_struct_deserialize_2a62f8c_1`
- **Location**: `source/postcard/src/de/deserializer.rs` (inside `deserialize_tuple_struct`)
- **Property**: `TupleStructRoundtrip`
- **Witness(es)**:
  - `witness_tuple_struct_roundtrip_case_small`
  - `witness_tuple_struct_roundtrip_case_large`
- **Source**: [#9](https://github.com/jamesmunns/postcard/pull/9) — Fix tuple struct deserialization bug
  > `Deserializer::deserialize_tuple_struct` discarded its `len` argument and forwarded to `deserialize_seq`. The serializer for a tuple struct calls `serialize_tuple` (no varint length prefix), but `deserialize_seq` expects a leading varint length, so the decoder's length cursor consumed a payload byte and every subsequent field was misaligned. The fix forwards to `deserialize_tuple(len, visitor)` so the wire shapes line up.
- **Fix commit**: `2a62f8c3cd643ccabea9034b71f9c9e97529f4e3` — Fix tuple struct deserialization bug
- **Invariant violated**: For every newtype-style tuple struct `S(T1, T2, ...)`, `from_bytes::<S>(&to_stdvec(&v).unwrap()).unwrap() == v`. The deserializer for a tuple struct must read the same number of fixed positional fields the serializer wrote, with no varint length prefix.
- **How the mutation triggers**: The mutation re-introduces `_len` (discarded) and forwards `deserialize_tuple_struct` to `deserialize_seq`. `deserialize_seq` reads a varint length first, so the decoder consumes a payload byte as the length and then under- or over-reads the actual fields, producing the wrong value or `SerdeDeCustom`.

## Dropped Candidates

- `67f9195` (Fix EOF not being handled correctly) — Bug surfaces only with an `embedded_io::Read` whose `read` returns 0 (EOF) on a non-empty buffer; reproducing without writing a custom Read impl that simulates partial reads would essentially be a different test, not the same bug.
- `6338601` (Fix Vec pre-alloc panic (#136)) — Bug is a panic from `Vec::with_capacity(huge)` triggered by a malformed varint-length claim; whether the panic fires depends on system memory and allocator behavior, not on a deterministic invariant of the postcard wire format.
- `7c172cc` (Fix a few bugs (postcard-schema map / enum disc max_size)) — Lives in the sister `postcard-schema` crate, not in `postcard`; building a property+witness here would require pulling postcard-schema into the runner's dep graph, which is out of scope for this workload's first slice.
- `b96e6f4` (Fix schema implementation for slice references (#142)) — Fix is a `?Sized` bound relaxation; the bug is a compile-error, not a runtime invariant violation, so there's no input that causes a witness assertion to fail at runtime.
- `488123` (Bump heapless, fixing unsoundness issue (#31)) — Fix is a Cargo dependency-version bump; the bug lives in the upstream heapless crate, not in postcard's source. There is no postcard-side function whose behavior changes.
