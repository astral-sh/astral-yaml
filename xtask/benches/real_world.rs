use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use astral_yaml_dev::Value;
use astral_yaml_dev::benchmarks::{FIXTURES, astral, reference};

fn sample(source: &str, parse: fn(&str) -> Vec<Value>) -> f64 {
    let start = Instant::now();
    let mut count = 0u32;

    // Each measurement includes parsing, value construction, and destruction.
    while start.elapsed() < Duration::from_millis(200) {
        for _ in 0..16 {
            black_box(parse(black_box(source)));
        }
        count += 16;
    }

    start.elapsed().as_secs_f64() / f64::from(count)
}

fn main() {
    println!("Equivalent workload: YAML stream -> Vec<astral_yaml_dev::Value>, including drop");
    println!("11 samples per implementation, >=200 ms each, alternating order");
    println!("fixture,bytes,astral_us,yaml_serde_us,speedup");

    for &(name, source) in FIXTURES {
        assert_eq!(
            astral(source),
            reference(source),
            "semantic mismatch: {name}"
        );

        for _ in 0..32 {
            black_box(astral(source));
            black_box(reference(source));
        }

        let mut ours = Vec::new();
        let mut theirs = Vec::new();

        for i in 0..11 {
            if i % 2 == 0 {
                ours.push(sample(source, astral));
                theirs.push(sample(source, reference));
            } else {
                theirs.push(sample(source, reference));
                ours.push(sample(source, astral));
            }
        }

        ours.sort_by(f64::total_cmp);
        theirs.sort_by(f64::total_cmp);
        println!(
            "{name},{},{:.2},{:.2},{:.2}x",
            source.len(),
            ours[5] * 1e6,
            theirs[5] * 1e6,
            theirs[5] / ours[5]
        );
    }
}
