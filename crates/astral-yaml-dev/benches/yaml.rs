use std::collections::BTreeMap;

use astral_yaml_dev::{Value, benchmarks};
use astral_yaml_serde::{Spanned, from_str, to_string};
use benchmarks::{FIXTURES, documents};

type Fields = Spanned<BTreeMap<String, Spanned<Value>>>;

fn main() {
    for &(_, source) in FIXTURES {
        let values = benchmarks::astral(source);
        let spanned = documents::<Spanned<Value>>(source);
        let fields = documents::<Fields>(source);
        assert_eq!(spanned.len(), values.len());
        assert_eq!(fields.len(), values.len());

        for ((value, spanned), fields) in values.iter().zip(&spanned).zip(&fields) {
            assert_eq!(value, &spanned.value);
            assert_eq!(spanned.span, fields.span);
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

    // Drop outputs inside each measured closure. Divan defers dropping returned values.
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
            black_box(documents::<Spanned<Value>>(black_box(source)));
        });
    }

    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn deserialize_spanned_fields(bencher: Bencher, name: &str) {
        let source = fixture(name);

        bencher.bench_local(|| {
            black_box(documents::<Fields>(black_box(source)));
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

#[cfg(feature = "editable")]
mod editable {
    use astral_yaml_dev::{Value, indoc};
    use astral_yaml_serde::Editable;
    use divan::{Bencher, black_box};

    const SOURCE: &str = indoc! {r"
        timeout: 30 # seconds
        servers: [primary, backup]
        extra: 'retained'
    "};

    #[divan::bench]
    fn parse(bencher: Bencher) {
        bencher.bench_local(|| {
            black_box(Editable::<Value>::parse(black_box(SOURCE)).unwrap());
        });
    }

    #[divan::bench]
    fn unchanged_output(bencher: Bencher) {
        let document = Editable::<Value>::parse(SOURCE).unwrap();
        bencher.bench_local(|| {
            black_box(document.to_yaml());
        });
    }

    // Include construction: each transaction starts from the same source.
    #[divan::bench]
    fn replace_scalar(bencher: Bencher) {
        bencher.bench_local(|| {
            let mut document = Editable::<Value>::parse(SOURCE).unwrap();
            document
                .edit(|editor| editor.root_mut().field("timeout")?.set(60))
                .unwrap();
            black_box(document);
        });
    }

    #[divan::bench]
    fn insert_entry(bencher: Bencher) {
        bencher.bench_local(|| {
            let mut document = Editable::<Value>::parse(SOURCE).unwrap();
            document
                .edit(|editor| {
                    editor.root_mut().into_mapping()?.set("enabled", true)?;
                    Ok(())
                })
                .unwrap();
            black_box(document);
        });
    }
}
