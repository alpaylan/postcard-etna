//! Witness tests for the postcard ETNA workload.
//!
//! Each `witness_*` test calls one of the `property_*` functions in
//! `postcard::etna` with frozen inputs. Tests pass on the base commit and
//! fail when the corresponding mutation is active.

use postcard::etna::{
    property_bytes_roundtrip, property_cobs_acc_no_oob, property_collect_str_roundtrip,
    property_i128_roundtrip, property_max_size_heapless_vec_zero,
    property_max_size_vec_upper_bound, property_tuple_struct_roundtrip, PropertyResult,
};

fn assert_pass(r: PropertyResult) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Fail(m) => panic!("property failed: {m}"),
        PropertyResult::Discard => panic!("property unexpectedly discarded"),
    }
}

// ---- i128_deserialization_70ea33a_1 ----

#[test]
fn witness_i128_roundtrip_case_negative_large() {
    // Regression case from PR (the same value the upstream test uses):
    // before the fix, deserialize_i128 read 16 raw bytes; with the
    // varint+zigzag encoding the serializer writes, this returns garbage.
    assert_pass(property_i128_roundtrip(-19490127978232325886905073712831_i128));
}

#[test]
fn witness_i128_roundtrip_case_positive_one() {
    assert_pass(property_i128_roundtrip(1_i128));
}

#[test]
fn witness_i128_roundtrip_case_negative_one() {
    assert_pass(property_i128_roundtrip(-1_i128));
}

// ---- cobs_acc_oob_41c2ddb_1 ----

#[test]
fn witness_cobs_acc_no_oob_case_overrun_by_one() {
    // Use an accumulator one byte smaller than the encoded frame (the
    // exact regression scenario described in the PR).
    assert_pass(property_cobs_acc_no_oob(10));
}

#[test]
fn witness_cobs_acc_no_oob_case_overrun_by_one_b() {
    assert_pass(property_cobs_acc_no_oob(7));
}

// ---- tuple_struct_deserialize_2a62f8c_1 ----

#[test]
fn witness_tuple_struct_roundtrip_case_small() {
    assert_pass(property_tuple_struct_roundtrip((1u8, 2u16, 3u32)));
}

#[test]
fn witness_tuple_struct_roundtrip_case_large() {
    assert_pass(property_tuple_struct_roundtrip((u8::MAX, u16::MAX, u32::MAX)));
}

// ---- serialize_bytes_size_prefix_58b3047_1 ----

#[test]
fn witness_bytes_roundtrip_case_short() {
    assert_pass(property_bytes_roundtrip(vec![0xCA, 0xFE, 0xBA, 0xBE]));
}

#[test]
fn witness_bytes_roundtrip_case_empty() {
    assert_pass(property_bytes_roundtrip(vec![]));
}

// ---- max_size_varint_zero_f31d226_1 ----

#[test]
fn witness_max_size_heapless_vec_zero_case_default() {
    assert_pass(property_max_size_heapless_vec_zero(()));
}

// ---- max_size_varint_off_by_one_c160626_1 ----

#[test]
fn witness_max_size_vec_upper_bound_case_127() {
    assert_pass(property_max_size_vec_upper_bound(0));
}

#[test]
fn witness_max_size_vec_upper_bound_case_128() {
    assert_pass(property_max_size_vec_upper_bound(1));
}

#[test]
fn witness_max_size_vec_upper_bound_case_16383() {
    assert_pass(property_max_size_vec_upper_bound(2));
}

#[test]
fn witness_max_size_vec_upper_bound_case_16384() {
    assert_pass(property_max_size_vec_upper_bound(3));
}

// ---- collect_str_unreachable_77fd54b_1 ----

#[test]
fn witness_collect_str_roundtrip_case_short_a() {
    assert_pass(property_collect_str_roundtrip(0));
}

#[test]
fn witness_collect_str_roundtrip_case_short_b() {
    assert_pass(property_collect_str_roundtrip(7));
}
