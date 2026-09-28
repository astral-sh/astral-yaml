use std::collections::BTreeMap;

use astral_yaml_dev::{Value, benchmarks};
use astral_yaml_serde::{Spanned, from_str, to_string};
use benchmarks::{FIXTURES, documents};

type Fields<'a> = Spanned<'a, BTreeMap<String, Spanned<'a, Value>>>;

fn main() {
    for &(_, source) in FIXTURES {
        let values = benchmarks::astral(source);
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
    }

    divan::main();
}

mod astral {
    use astral_yaml_dev::{Value, benchmarks};
    use astral_yaml_serde::{Spanned, to_string};
    use benchmarks::{FIXTURES, ServiceConfig, documents, fixture, services_fixture};
    use divan::{Bencher, black_box};

    use super::Fields;

    // Drop outputs inside each measured closure; Divan defers dropping returned values.
    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn parse(bencher: Bencher, name: &str) {
        let source = fixture(name);

        bencher.bench_local(|| {
            black_box(astral_yaml::parse(black_box(source)).unwrap());
        });
    }

    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn deserialize(bencher: Bencher, name: &str) {
        let source = fixture(name);

        bencher.bench_local(|| {
            black_box(documents::<Value>(black_box(source)));
        });
    }

    #[divan::bench]
    fn deserialize_services_128(bencher: Bencher) {
        let source = services_fixture();

        bencher.bench_local(|| {
            black_box(documents::<ServiceConfig>(black_box(&source)));
        });
    }

    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn deserialize_spanned(bencher: Bencher, name: &str) {
        let source = fixture(name);

        bencher.bench_local(|| {
            black_box(documents::<Spanned<'_, Value>>(black_box(source)));
        });
    }

    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn deserialize_spanned_fields(bencher: Bencher, name: &str) {
        let source = fixture(name);

        bencher.bench_local(|| {
            black_box(documents::<Fields<'_>>(black_box(source)));
        });
    }

    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn serialize(bencher: Bencher, name: &str) {
        let values = benchmarks::astral(fixture(name));

        bencher.bench_local(|| {
            for value in black_box(&values) {
                black_box(to_string(black_box(value)).unwrap());
            }
        });
    }
}
