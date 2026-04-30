// ETNA workload runner for postcard.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: <PascalCase> | All
//
// Each run emits a single JSON line on stdout; exit status is always 0 on
// completion (non-zero exit is reserved for adapter-level panics that escape
// the catch_unwind in main()).

use crabcheck::quickcheck as crabcheck_qc;
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use postcard::etna::{
    property_bytes_roundtrip, property_cobs_acc_no_oob, property_collect_str_roundtrip,
    property_i128_roundtrip, property_max_size_heapless_vec_zero,
    property_max_size_vec_upper_bound, property_tuple_struct_roundtrip, PropertyResult,
};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestRunner};
use quickcheck::{QuickCheck, ResultStatus, TestResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "I128Roundtrip",
    "CobsAccNoOob",
    "TupleStructRoundtrip",
    "BytesRoundtrip",
    "MaxSizeHeaplessVecZero",
    "MaxSizeVecUpperBound",
    "CollectStrRoundtrip",
];

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    let mut final_status: Result<(), String> = Ok(());
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if r.is_err() && final_status.is_ok() {
            final_status = r;
        }
    }
    (final_status, total)
}

// ---- etna (deterministic witness-shaped inputs) ----

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "I128Roundtrip" => to_err(property_i128_roundtrip(
            -19490127978232325886905073712831_i128,
        )),
        "CobsAccNoOob" => to_err(property_cobs_acc_no_oob(10)),
        "TupleStructRoundtrip" => to_err(property_tuple_struct_roundtrip((1, 2, 3))),
        "BytesRoundtrip" => to_err(property_bytes_roundtrip(vec![1, 2, 3, 4, 5])),
        "MaxSizeHeaplessVecZero" => to_err(property_max_size_heapless_vec_zero(())),
        "MaxSizeVecUpperBound" => to_err(property_max_size_vec_upper_bound(0)),
        "CollectStrRoundtrip" => to_err(property_collect_str_roundtrip(7)),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    (result, Metrics { inputs: 1, elapsed_us })
}

// ---- proptest ----

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let mut runner = TestRunner::new(ProptestConfig::default());
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "I128Roundtrip" => runner
            .run(&any::<i128>(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_i128_roundtrip(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "CobsAccNoOob" => runner
            .run(&any::<u8>(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_cobs_acc_no_oob(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "TupleStructRoundtrip" => runner
            .run(&(any::<u8>(), any::<u16>(), any::<u32>()), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_tuple_struct_roundtrip(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({:?})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "BytesRoundtrip" => runner
            .run(
                &proptest::collection::vec(any::<u8>(), 0..64usize),
                move |args| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        property_bytes_roundtrip(args.clone())
                    }));
                    match res {
                        Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                        Ok(PropertyResult::Fail(_)) | Err(_) => {
                            Err(TestCaseError::fail(format!("({:?})", args)))
                        }
                    }
                },
            )
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "MaxSizeHeaplessVecZero" => runner
            .run(&Just(()), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_max_size_heapless_vec_zero(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail("(())".to_string()))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "MaxSizeVecUpperBound" => runner
            .run(&any::<u16>(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_max_size_vec_upper_bound(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        "CollectStrRoundtrip" => runner
            .run(&any::<u8>(), move |args| {
                c.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_collect_str_roundtrip(args)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({})", args)))
                    }
                }
            })
            .map_err(|e| match e {
                proptest::test_runner::TestError::Fail(r, _) => r.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ---- quickcheck ----
//
// Forked quickcheck shrinks Gen.size() so Vec<T> arbitrary panics on
// `random_range(0..0)`. We use Vec<u8> which returns empty on size=0.

#[derive(Debug, Clone)]
struct QcBytes(Vec<u8>);

impl std::fmt::Display for QcBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl quickcheck::Arbitrary for QcBytes {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        // Cap the length to avoid the empty-range panic when `g.size() == 0`.
        let s = g.size().max(1);
        let len = g.random_range(0..s);
        let v = (0..len)
            .map(|_| <u8 as quickcheck::Arbitrary>::arbitrary(g))
            .collect();
        QcBytes(v)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(QcBytes))
    }
}

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_i128_roundtrip(a: i64, b: i64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    // Combine two i64s into one i128 covering large magnitudes incl negatives.
    let v: i128 = ((a as i128) << 64) | (b as i128 & 0xFFFF_FFFF_FFFF_FFFFi128);
    match property_i128_roundtrip(v) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_cobs_acc_no_oob(n: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_cobs_acc_no_oob(n) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_tuple_struct_roundtrip(a: u8, b: u16, c: u32) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_tuple_struct_roundtrip((a, b, c)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_bytes_roundtrip(payload: QcBytes) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_bytes_roundtrip(payload.0) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_max_size_heapless_vec_zero() -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_max_size_heapless_vec_zero(()) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_max_size_vec_upper_bound(payload_len: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_max_size_vec_upper_bound(payload_len) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_collect_str_roundtrip(seed: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_collect_str_roundtrip(seed) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let result = match property {
        "I128Roundtrip" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_i128_roundtrip as fn(i64, i64) -> TestResult),
        "CobsAccNoOob" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_cobs_acc_no_oob as fn(u8) -> TestResult),
        "TupleStructRoundtrip" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_tuple_struct_roundtrip as fn(u8, u16, u32) -> TestResult),
        "BytesRoundtrip" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_bytes_roundtrip as fn(QcBytes) -> TestResult),
        "MaxSizeHeaplessVecZero" => QuickCheck::new()
            .tests(20)
            .max_tests(40)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_max_size_heapless_vec_zero as fn() -> TestResult),
        "MaxSizeVecUpperBound" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_max_size_vec_upper_bound as fn(u16) -> TestResult),
        "CollectStrRoundtrip" => QuickCheck::new()
            .tests(200)
            .max_tests(2000)
            .max_time(Duration::from_secs(86_400))
            .quicktest(qc_collect_str_roundtrip as fn(u8) -> TestResult),
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("aborted: {err:?}")),
        ResultStatus::TimedOut => Err("timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "gave up: passed={}, discarded={}",
            result.n_tests_passed, result.n_tests_discarded
        )),
    };
    (status, metrics)
}

// ---- crabcheck ----

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_i128_roundtrip(args: (i64, i64)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let (a, b) = args;
    let v: i128 = ((a as i128) << 64) | (b as i128 & 0xFFFF_FFFF_FFFF_FFFFi128);
    match property_i128_roundtrip(v) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_cobs_acc_no_oob(n: u8) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_cobs_acc_no_oob(n) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_tuple_struct_roundtrip(args: (u8, u16, u32)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_tuple_struct_roundtrip(args) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_bytes_roundtrip(payload: Vec<u8>) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_bytes_roundtrip(payload) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_max_size_heapless_vec_zero(_: u8) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_max_size_heapless_vec_zero(()) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_max_size_vec_upper_bound(n: u16) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_max_size_vec_upper_bound(n) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_collect_str_roundtrip(seed: u8) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_collect_str_roundtrip(seed) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cfg = crabcheck_qc::Config { tests: 2000 };
    let small_cfg = crabcheck_qc::Config { tests: 40 };
    let result = match property {
        "I128Roundtrip" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_i128_roundtrip as fn((i64, i64)) -> Option<bool>,
        ),
        "CobsAccNoOob" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_cobs_acc_no_oob as fn(u8) -> Option<bool>,
        ),
        "TupleStructRoundtrip" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_tuple_struct_roundtrip as fn((u8, u16, u32)) -> Option<bool>,
        ),
        "BytesRoundtrip" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_bytes_roundtrip as fn(Vec<u8>) -> Option<bool>,
        ),
        "MaxSizeHeaplessVecZero" => crabcheck_qc::quickcheck_with_config(
            small_cfg,
            cc_max_size_heapless_vec_zero as fn(u8) -> Option<bool>,
        ),
        "MaxSizeVecUpperBound" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_max_size_vec_upper_bound as fn(u16) -> Option<bool>,
        ),
        "CollectStrRoundtrip" => crabcheck_qc::quickcheck_with_config(
            cfg,
            cc_collect_str_roundtrip as fn(u8) -> Option<bool>,
        ),
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck_qc::ResultStatus::TimedOut => Err("timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => Err(format!("aborted: {error}")),
    };
    (status, metrics)
}

// ---- hegel ----

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(200)
        .suppress_health_check(hegel::HealthCheck::all())
}

fn hegel_settings_small() -> HegelSettings {
    HegelSettings::new()
        .test_cases(20)
        .suppress_health_check(hegel::HealthCheck::all())
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let small = hegel_settings_small();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "I128Roundtrip" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let lo: i64 = tc.draw(hgen::integers::<i64>());
                let hi: i64 = tc.draw(hgen::integers::<i64>());
                let v: i128 = ((hi as i128) << 64) | (lo as i128 & 0xFFFF_FFFF_FFFF_FFFFi128);
                let cex = format!("({})", v);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_i128_roundtrip(v)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "CobsAccNoOob" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n: u8 = tc.draw(hgen::integers::<u8>());
                let cex = format!("({})", n);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_cobs_acc_no_oob(n)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "TupleStructRoundtrip" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a: u8 = tc.draw(hgen::integers::<u8>());
                let b: u16 = tc.draw(hgen::integers::<u16>());
                let c: u32 = tc.draw(hgen::integers::<u32>());
                let cex = format!("(({}, {}, {}))", a, b, c);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_tuple_struct_roundtrip((a, b, c))
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "BytesRoundtrip" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let payload: Vec<u8> = tc.draw(
                    hgen::vecs(hgen::integers::<u8>())
                        .min_size(0)
                        .max_size(64),
                );
                let cex = format!("({:?})", payload);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_bytes_roundtrip(payload.clone())
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "MaxSizeHeaplessVecZero" => {
            Hegel::new(|_tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_max_size_heapless_vec_zero(())
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("(())"),
                }
            })
            .settings(small.clone())
            .run();
        }
        "MaxSizeVecUpperBound" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n: u16 = tc.draw(hgen::integers::<u16>());
                let cex = format!("({})", n);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_max_size_vec_upper_bound(n)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "CollectStrRoundtrip" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let seed: u8 = tc.draw(hgen::integers::<u8>());
                let cex = format!("({})", seed);
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_collect_str_roundtrip(seed)
                }));
                match res {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{cex}"),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{property}"),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: I128Roundtrip | CobsAccNoOob | \
             TupleStructRoundtrip | BytesRoundtrip | MaxSizeHeaplessVecZero | \
             MaxSizeVecUpperBound | CollectStrRoundtrip | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "adapter panic (non-string payload)".to_string());
            emit_json(tool, property, "aborted", Metrics::default(), None, Some(&msg));
            return;
        }
    };
    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(e) => emit_json(tool, property, "failed", metrics, Some(&e), None),
    }
}
