//! Compares [`BString`] methods with their [`String`] counterparts.

use std::hint::black_box;

use boundstr::BString;
use boundstr::encoding::{Utf8, Utf16};
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};

/// The number of characters pushed per iteration.
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

criterion_group!(benches, push);
criterion_main!(benches);
