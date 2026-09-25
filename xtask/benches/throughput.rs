use astral_yaml_serde::{Spanned, from_document, from_str, to_string};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    env,
    hint::black_box,
    time::{Duration, Instant},
};
use yaml_serde::Value;

#[path = "../tests/support/mod.rs"]
mod support;

type Fields<'a> = Spanned<'a, BTreeMap<String, Spanned<'a, Value>>>;

fn documents<'a, T: Deserialize<'a>>(source: &'a str) -> Vec<T> {
    let stream = astral_yaml::parse(source).unwrap();
    (0..stream.documents().len())
        .map(|index| from_document(&stream, index).unwrap())
        .collect()
}

fn sample(mut operation: impl FnMut(), duration: Duration) -> f64 {
    let start = Instant::now();
    let mut count = 0u32;

    while start.elapsed() < duration {
        for _ in 0..16 {
            operation();
        }

        count += 16;
    }

    start.elapsed().as_secs_f64() / f64::from(count)
}

fn measure(fixture: &str, mode: &str, mut operation: impl FnMut()) {
    if env::var("ASTRAL_BENCH_MODE").is_ok_and(|filter| filter != mode) {
        return;
    }

    for _ in 0..64 {
        operation();
    }

    if env::var_os("ASTRAL_BENCH_PROFILE").is_some() {
        sample(operation, Duration::from_secs(20));
        return;
    }

    let mut samples: Vec<_> = (0..11)
        .map(|_| sample(&mut operation, Duration::from_millis(150)))
        .collect();
    samples.sort_by(f64::total_cmp);
    println!("{fixture},{mode},{:.3}", samples[5] * 1e6);
}

fn main() {
    println!("fixture,mode,median_us");

    for &(name, source) in support::FIXTURES {
        if env::var("ASTRAL_BENCH_FIXTURE").is_ok_and(|filter| filter != name) {
            continue;
        }

        let values = support::astral(source);
        assert_eq!(values, support::reference(source));
        let spanned = documents::<Spanned<'_, Value>>(source);
        let fields = documents::<Fields<'_>>(source);
        assert_eq!(spanned.len(), values.len());
        assert_eq!(fields.len(), values.len());

        for ((value, spanned), fields) in values.iter().zip(&spanned).zip(&fields) {
            assert_eq!(value, &spanned.value);
            assert_eq!(spanned.comments, fields.comments);
            assert_eq!(
                *value,
                from_str::<Value>(&to_string(fields).unwrap()).unwrap()
            );
            assert_eq!(
                *value,
                from_str::<Value>(&to_string(value).unwrap()).unwrap()
            );
        }

        measure(name, "parse", || {
            black_box(astral_yaml::parse(black_box(source)).unwrap());
        });
        measure(name, "unspanned", || {
            black_box(documents::<Value>(black_box(source)));
        });
        measure(name, "spanned", || {
            black_box(documents::<Spanned<'_, Value>>(black_box(source)));
        });
        measure(name, "spanned-fields", || {
            black_box(documents::<Fields<'_>>(black_box(source)));
        });
        measure(name, "serialize", || {
            for value in black_box(&values) {
                black_box(to_string(black_box(value)).unwrap());
            }
        });
    }
}
