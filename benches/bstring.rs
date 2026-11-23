//! Compares [`BString`] methods with their [`String`] counterparts.

use std::hint::black_box;

use boundstr::BString;
use boundstr::encoding::{Utf8, Utf16};
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};

/// The number of characters pushed or popped per iteration.
const CHARS: usize = 4096;
/// Large enough that no push is ever rejected, in either encoding.
const MAX: usize = CHARS * 4;

fn inputs() -> [(&'static str, Vec<char>); 2] {
    [
        ("ascii", "a".chars().cycle().take(CHARS).collect()),
        ("mixed", "aé你😀".chars().cycle().take(CHARS).collect()),
    ]
}

fn push(c: &mut Criterion) {
    for (name, chars) in inputs() {
        let mut group = c.benchmark_group(format!("push/{name}"));
        group.throughput(Throughput::Elements(CHARS as u64));

        group.bench_function("String::push", |b| {
            b.iter_batched_ref(
                || String::with_capacity(MAX),
                |s| {
                    for &ch in &chars {
                        s.push(black_box(ch));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf8>::try_push", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf8>::with_capacity(MAX),
                |s| {
                    for &ch in &chars {
                        s.try_push(black_box(ch)).unwrap();
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf16>::try_push", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf16>::with_capacity(MAX),
                |s| {
                    for &ch in &chars {
                        s.try_push(black_box(ch)).unwrap();
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.finish();
    }
}

fn pop(c: &mut Criterion) {
    for (name, chars) in inputs() {
        let full: String = chars.iter().collect();
        let mut group = c.benchmark_group(format!("pop/{name}"));
        group.throughput(Throughput::Elements(CHARS as u64));

        group.bench_function("String::pop", |b| {
            b.iter_batched_ref(
                || full.clone(),
                |s| {
                    while let Some(ch) = s.pop() {
                        black_box(ch);
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf8>::pop", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf8>::try_from(full.as_str()).unwrap(),
                |s| {
                    while let Some(ch) = s.pop() {
                        black_box(ch);
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf16>::pop", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf16>::try_from(full.as_str()).unwrap(),
                |s| {
                    while let Some(ch) = s.pop() {
                        black_box(ch);
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.finish();
    }
}

/// The number of characters removed per iteration.
///
/// Kept small because removing from the front shifts the rest of the string,
/// which would otherwise dominate the measurement.
const REMOVED: usize = 64;

fn remove(c: &mut Criterion) {
    for (name, chars) in inputs() {
        let full: String = chars.iter().take(REMOVED).collect();
        let mut group = c.benchmark_group(format!("remove/{name}"));
        group.throughput(Throughput::Elements(REMOVED as u64));

        group.bench_function("String::remove", |b| {
            b.iter_batched_ref(
                || full.clone(),
                |s| {
                    while !s.is_empty() {
                        black_box(s.remove(0));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf8>::remove", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf8>::try_from(full.as_str()).unwrap(),
                |s| {
                    while !s.is_empty() {
                        black_box(s.remove(0));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf16>::remove", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf16>::try_from(full.as_str()).unwrap(),
                |s| {
                    while !s.is_empty() {
                        black_box(s.remove(0));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.finish();
    }
}

/// The number of characters inserted per iteration.
///
/// Kept small because inserting at the front shifts the rest of the string,
/// which would otherwise dominate the measurement.
const INSERTED: usize = 64;

fn insert(c: &mut Criterion) {
    for (name, chars) in inputs() {
        let chars = &chars[..INSERTED];
        let mut group = c.benchmark_group(format!("insert/{name}"));
        group.throughput(Throughput::Elements(INSERTED as u64));

        group.bench_function("String::insert", |b| {
            b.iter_batched_ref(
                || String::with_capacity(MAX),
                |s| {
                    for &ch in chars {
                        s.insert(0, black_box(ch));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf8>::try_insert", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf8>::with_capacity(MAX),
                |s| {
                    for &ch in chars {
                        s.try_insert(0, black_box(ch)).unwrap();
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("BString<Utf16>::try_insert", |b| {
            b.iter_batched_ref(
                || BString::<MAX, Utf16>::with_capacity(MAX),
                |s| {
                    for &ch in chars {
                        s.try_insert(0, black_box(ch)).unwrap();
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.finish();
    }
}

criterion_group!(benches, push, pop, remove, insert);
criterion_main!(benches);
