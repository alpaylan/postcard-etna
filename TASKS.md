# postcard — ETNA Tasks

Total tasks: 28

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `cobs_acc_oob_41c2ddb_1` | proptest | `CobsAccNoOob` | `witness_cobs_acc_no_oob_case_overrun_by_one` |
| 002 | `cobs_acc_oob_41c2ddb_1` | quickcheck | `CobsAccNoOob` | `witness_cobs_acc_no_oob_case_overrun_by_one` |
| 003 | `cobs_acc_oob_41c2ddb_1` | crabcheck | `CobsAccNoOob` | `witness_cobs_acc_no_oob_case_overrun_by_one` |
| 004 | `cobs_acc_oob_41c2ddb_1` | hegel | `CobsAccNoOob` | `witness_cobs_acc_no_oob_case_overrun_by_one` |
| 005 | `collect_str_unreachable_77fd54b_1` | proptest | `CollectStrRoundtrip` | `witness_collect_str_roundtrip_case_short_a` |
| 006 | `collect_str_unreachable_77fd54b_1` | quickcheck | `CollectStrRoundtrip` | `witness_collect_str_roundtrip_case_short_a` |
| 007 | `collect_str_unreachable_77fd54b_1` | crabcheck | `CollectStrRoundtrip` | `witness_collect_str_roundtrip_case_short_a` |
| 008 | `collect_str_unreachable_77fd54b_1` | hegel | `CollectStrRoundtrip` | `witness_collect_str_roundtrip_case_short_a` |
| 009 | `i128_deserialization_70ea33a_1` | proptest | `I128Roundtrip` | `witness_i128_roundtrip_case_negative_large` |
| 010 | `i128_deserialization_70ea33a_1` | quickcheck | `I128Roundtrip` | `witness_i128_roundtrip_case_negative_large` |
| 011 | `i128_deserialization_70ea33a_1` | crabcheck | `I128Roundtrip` | `witness_i128_roundtrip_case_negative_large` |
| 012 | `i128_deserialization_70ea33a_1` | hegel | `I128Roundtrip` | `witness_i128_roundtrip_case_negative_large` |
| 013 | `max_size_varint_off_by_one_c160626_1` | proptest | `MaxSizeVecUpperBound` | `witness_max_size_vec_upper_bound_case_127` |
| 014 | `max_size_varint_off_by_one_c160626_1` | quickcheck | `MaxSizeVecUpperBound` | `witness_max_size_vec_upper_bound_case_127` |
| 015 | `max_size_varint_off_by_one_c160626_1` | crabcheck | `MaxSizeVecUpperBound` | `witness_max_size_vec_upper_bound_case_127` |
| 016 | `max_size_varint_off_by_one_c160626_1` | hegel | `MaxSizeVecUpperBound` | `witness_max_size_vec_upper_bound_case_127` |
| 017 | `max_size_varint_zero_f31d226_1` | proptest | `MaxSizeHeaplessVecZero` | `witness_max_size_heapless_vec_zero_case_default` |
| 018 | `max_size_varint_zero_f31d226_1` | quickcheck | `MaxSizeHeaplessVecZero` | `witness_max_size_heapless_vec_zero_case_default` |
| 019 | `max_size_varint_zero_f31d226_1` | crabcheck | `MaxSizeHeaplessVecZero` | `witness_max_size_heapless_vec_zero_case_default` |
| 020 | `max_size_varint_zero_f31d226_1` | hegel | `MaxSizeHeaplessVecZero` | `witness_max_size_heapless_vec_zero_case_default` |
| 021 | `serialize_bytes_size_prefix_58b3047_1` | proptest | `BytesRoundtrip` | `witness_bytes_roundtrip_case_short` |
| 022 | `serialize_bytes_size_prefix_58b3047_1` | quickcheck | `BytesRoundtrip` | `witness_bytes_roundtrip_case_short` |
| 023 | `serialize_bytes_size_prefix_58b3047_1` | crabcheck | `BytesRoundtrip` | `witness_bytes_roundtrip_case_short` |
| 024 | `serialize_bytes_size_prefix_58b3047_1` | hegel | `BytesRoundtrip` | `witness_bytes_roundtrip_case_short` |
| 025 | `tuple_struct_deserialize_2a62f8c_1` | proptest | `TupleStructRoundtrip` | `witness_tuple_struct_roundtrip_case_small` |
| 026 | `tuple_struct_deserialize_2a62f8c_1` | quickcheck | `TupleStructRoundtrip` | `witness_tuple_struct_roundtrip_case_small` |
| 027 | `tuple_struct_deserialize_2a62f8c_1` | crabcheck | `TupleStructRoundtrip` | `witness_tuple_struct_roundtrip_case_small` |
| 028 | `tuple_struct_deserialize_2a62f8c_1` | hegel | `TupleStructRoundtrip` | `witness_tuple_struct_roundtrip_case_small` |

## Witness Catalog

- `witness_cobs_acc_no_oob_case_overrun_by_one` — 11-byte buffer, 12-byte encoded frame — the exact regression scenario.
- `witness_cobs_acc_no_oob_case_overrun_by_one_b` — base passes, variant fails
- `witness_collect_str_roundtrip_case_short_a` — base passes, variant fails
- `witness_collect_str_roundtrip_case_short_b` — base passes, variant fails
- `witness_i128_roundtrip_case_negative_large` — Regression value lifted directly from the test added in the fix commit.
- `witness_i128_roundtrip_case_positive_one` — base passes, variant fails
- `witness_i128_roundtrip_case_negative_one` — base passes, variant fails
- `witness_max_size_vec_upper_bound_case_127` — bits=7; buggy varint_size returns 2 instead of 1.
- `witness_max_size_vec_upper_bound_case_16383` — bits=14; buggy varint_size returns 3 instead of 2.
- `witness_max_size_heapless_vec_zero_case_default` — base passes, variant fails
- `witness_bytes_roundtrip_case_short` — base passes, variant fails
- `witness_bytes_roundtrip_case_empty` — base passes, variant fails
- `witness_tuple_struct_roundtrip_case_small` — base passes, variant fails
- `witness_tuple_struct_roundtrip_case_large` — base passes, variant fails
