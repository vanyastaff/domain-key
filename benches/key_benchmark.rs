#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use domain_key::{Domain, Key, KeyDomain};
use std::collections::HashMap;

#[derive(Debug)]
struct BenchDomain;
impl Domain for BenchDomain {
    const DOMAIN_NAME: &'static str = "bench";
}
impl KeyDomain for BenchDomain {
    const MAX_LENGTH: usize = 128;
}
type BenchKey = Key<BenchDomain>;

fn bench_key_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_creation");

    group.bench_function("short_4", |b| b.iter(|| BenchKey::new(black_box("abcd"))));

    group.bench_function("medium_16", |b| {
        b.iter(|| BenchKey::new(black_box("user_profile_set")))
    });

    group.bench_function("long_48", |b| {
        let input = "a_b_c_d_e_f_g_h_i_j_k_l_m_n_o_p_q_r_s_t_u_v_w_x".to_string();
        b.iter(|| BenchKey::new(black_box(input.as_str())))
    });

    group.finish();
}

fn bench_hash_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_lookup");

    // Pre-build a HashMap with 1000 keys
    let mut map: HashMap<BenchKey, u64> = HashMap::new();
    for i in 0..1000 {
        let key = BenchKey::new(format!("key_{i:04}")).unwrap();
        map.insert(key, i);
    }

    // Lookup by Key (clone to create)
    group.bench_function("by_key_clone", |b| {
        b.iter(|| {
            let key = BenchKey::new(black_box("key_0500")).unwrap();
            map.get(&key)
        })
    });

    // Lookup by &str via Borrow<str>
    group.bench_function("by_str_borrow", |b| {
        b.iter(|| map.get(black_box("key_0500")))
    });

    group.finish();
}

fn bench_accessors(c: &mut Criterion) {
    let key = BenchKey::new("user_profile_settings").unwrap();

    let mut group = c.benchmark_group("accessors");

    group.bench_function("len", |b| b.iter(|| black_box(&key).len()));

    group.bench_function("as_str", |b| b.iter(|| black_box(&key).as_str()));

    group.bench_function("hash", |b| b.iter(|| black_box(&key).hash()));

    group.bench_function("starts_with", |b| {
        b.iter(|| black_box(&key).starts_with("user_"))
    });

    group.bench_function("contains", |b| {
        b.iter(|| black_box(&key).contains("profile"))
    });

    group.finish();
}

fn bench_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone");

    let short = BenchKey::new("abc").unwrap();
    let long = BenchKey::new("this_is_a_somewhat_longer_key_for_bench").unwrap();

    group.bench_function("short_inline", |b| b.iter(|| black_box(&short).clone()));

    group.bench_function("long_heap", |b| b.iter(|| black_box(&long).clone()));

    group.finish();
}

fn bench_from_parts(c: &mut Criterion) {
    c.bench_function("from_parts_3", |b| {
        b.iter(|| BenchKey::from_parts(black_box(&["user", "123", "profile"]), "_"))
    });
}

fn bench_collection_insert(c: &mut Criterion) {
    c.bench_function("hashmap_insert_1000", |b| {
        let keys: Vec<BenchKey> = (0..1000)
            .map(|i| BenchKey::new(format!("key_{i:04}")).unwrap())
            .collect();

        b.iter(|| {
            let mut map = HashMap::with_capacity(1000);
            for (i, key) in keys.iter().enumerate() {
                map.insert(key.clone(), i);
            }
            black_box(&map);
        })
    });
}

criterion_group!(
    benches,
    bench_key_creation,
    bench_hash_lookup,
    bench_accessors,
    bench_clone,
    bench_from_parts,
    bench_collection_insert,
);
criterion_main!(benches);
