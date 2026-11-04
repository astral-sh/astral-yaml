use astral_yaml_dev::{Value, benchmarks};
use benchmarks::{FIXTURES, ServiceConfig, documents, fixture, services_fixture};
use serde::Deserialize;

fn deser_documents<'a, T: deser::Deserialize<'a>>(source: &'a str) -> Vec<T> {
    let config = deser_yaml::DeserializerConfig::builder()
        .merge_keys(false)
        .build();
    let mut deserializer = deser_yaml::Deserializer::from_str_with_config(source, config);

    deserializer.iter::<T>().collect::<Result<_, _>>().unwrap()
}

fn normalize_deser_value(value: &deser_value::Value) -> Value {
    use deser::ImplicitValue;
    use deser_value::Kind;

    match value.kind() {
        Kind::Null => Value::Null,
        Kind::Bool(value) => Value::Bool(*value),
        Kind::U64(value) => Value::Unsigned(*value),
        Kind::I64(value) => Value::Integer(*value),
        Kind::F32(value) => Value::Float(f64::from(*value)),
        Kind::F64(value) => Value::Float(*value),
        Kind::Str(value) | Kind::Lexical(value) => Value::String(value.clone()),
        Kind::Implicit(value) => match value.value() {
            ImplicitValue::Null => Value::Null,
            ImplicitValue::Bool(value) => Value::Bool(value),
            ImplicitValue::U64(value) => Value::Unsigned(value),
            ImplicitValue::I64(value) => Value::Integer(value),
            ImplicitValue::F64(value) => Value::Float(value),
            other => panic!("unsupported implicit Deser value: {other:?}"),
        },
        Kind::Seq(values) => Value::Sequence(values.iter().map(normalize_deser_value).collect()),
        Kind::Map(entries) => Value::Mapping(
            entries
                .iter()
                .map(|(key, value)| (normalize_deser_value(key), normalize_deser_value(value)))
                .collect(),
        ),
        _ => panic!("unsupported Deser value: {value:?}"),
    }
}

fn yaml_serde_documents(source: &str) -> Vec<Value> {
    yaml_serde::Deserializer::from_str(source)
        .map(|document| Value::deserialize(document).unwrap())
        .collect()
}

fn saphyr_options() -> serde_saphyr::Options {
    serde_saphyr::options! {
        strict_booleans: true,
        merge_keys: serde_saphyr::MergeKeyPolicy::AsOrdinary,
        reject_unsupported_tags: true,
    }
}

fn main() {
    for &(name, source) in FIXTURES {
        parse::check_counts(name, source);

        let values = benchmarks::astral(source);
        assert_eq!(values, yaml_serde_documents(source));
        assert_eq!(
            values,
            serde_saphyr::from_multiple_with_options::<Value>(source, saphyr_options()).unwrap()
        );
        let deser_values = deser_documents::<deser_value::Value>(source);
        assert_eq!(
            values,
            deser_values
                .iter()
                .map(normalize_deser_value)
                .collect::<Vec<_>>()
        );
    }

    let source = services_fixture();
    assert_eq!(
        documents::<ServiceConfig>(&source),
        serde_saphyr::from_multiple_with_options::<ServiceConfig>(&source, saphyr_options())
            .unwrap()
    );
    assert_eq!(
        documents::<ServiceConfig>(&source),
        deser_documents::<ServiceConfig>(&source)
    );

    divan::main();
}

mod parse {
    use std::fmt;

    use astral_yaml_dev::benchmarks::fixture;
    use divan::{Bencher, black_box};
    use granit_parser::{Event, Parser};

    #[derive(Debug, Default, Eq, PartialEq)]
    struct Counts {
        documents: usize,
        nodes: usize,
        comments: usize,
    }

    pub(super) fn check_counts(name: &str, source: &str) {
        assert_eq!(astral_counts(source), granit_counts(source), "{name}");
    }

    fn astral_counts(source: &str) -> Counts {
        let stream = astral_yaml::parse(source).unwrap();

        Counts {
            documents: stream.documents().len(),
            nodes: stream.nodes().len(),
            comments: stream.comments().len(),
        }
    }

    fn granit_counts(source: &str) -> Counts {
        let options = granit_parser::options! { strict_indentation: true, emit_comments: true };
        let mut counts = Counts::default();

        for result in Parser::new_from_str_with_options(source, options) {
            let (event, _) = result.unwrap();
            match event {
                Event::DocumentStart(..) => counts.documents += 1,
                Event::Comment(..) => counts.comments += 1,
                _ if event.is_node() => counts.nodes += 1,
                _ => {}
            }
        }

        counts
    }

    #[derive(Clone, Copy, Debug)]
    enum Implementation {
        Astral,
        Granit,
    }

    impl fmt::Display for Implementation {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(match self {
                Self::Astral => "astral-yaml",
                Self::Granit => "granit-parser",
            })
        }
    }

    const IMPLEMENTATIONS: &[Implementation] = &[Implementation::Astral, Implementation::Granit];

    fn run(bencher: Bencher, name: &str, implementation: Implementation) {
        let source = fixture(name);

        match implementation {
            Implementation::Astral => bencher.bench_local(|| {
                black_box(astral_counts(black_box(source)));
            }),
            Implementation::Granit => bencher.bench_local(|| {
                black_box(granit_counts(black_box(source)));
            }),
        }
    }

    #[divan::bench(name = "uv-ci", args = IMPLEMENTATIONS)]
    fn uv_ci(bencher: Bencher, implementation: Implementation) {
        run(bencher, "uv-ci", implementation);
    }

    #[divan::bench(name = "kubernetes-guestbook", args = IMPLEMENTATIONS)]
    fn kubernetes_guestbook(bencher: Bencher, implementation: Implementation) {
        run(bencher, "kubernetes-guestbook", implementation);
    }

    #[divan::bench(name = "openapi-petstore", args = IMPLEMENTATIONS)]
    fn openapi_petstore(bencher: Bencher, implementation: Implementation) {
        run(bencher, "openapi-petstore", implementation);
    }
}

mod deserialize {
    use std::fmt;

    use astral_yaml_dev::Value;
    use divan::{Bencher, black_box};

    use super::{deser_documents, documents, fixture, saphyr_options, yaml_serde_documents};

    #[derive(Clone, Copy, Debug)]
    enum Implementation {
        Astral,
        YamlSerde,
        Saphyr,
        DeserYaml,
    }

    impl fmt::Display for Implementation {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(match self {
                Self::Astral => "astral-yaml",
                Self::YamlSerde => "yaml_serde",
                Self::Saphyr => "serde-saphyr",
                Self::DeserYaml => "deser-yaml",
            })
        }
    }

    const IMPLEMENTATIONS: &[Implementation] = &[
        Implementation::Astral,
        Implementation::YamlSerde,
        Implementation::Saphyr,
        Implementation::DeserYaml,
    ];

    fn run(bencher: Bencher, name: &str, implementation: Implementation) {
        let source = fixture(name);

        // Select the implementation before measuring, and include output destruction.
        match implementation {
            Implementation::Astral => bencher.bench_local(|| {
                black_box(documents::<Value>(black_box(source)));
            }),
            Implementation::YamlSerde => bencher.bench_local(|| {
                black_box(yaml_serde_documents(black_box(source)));
            }),
            Implementation::Saphyr => bencher.bench_local(|| {
                black_box(
                    serde_saphyr::from_multiple_with_options::<Value>(
                        black_box(source),
                        saphyr_options(),
                    )
                    .unwrap(),
                );
            }),
            Implementation::DeserYaml => bencher.bench_local(|| {
                black_box(deser_documents::<deser_value::Value>(black_box(source)));
            }),
        }
    }

    #[divan::bench(name = "uv-ci", args = IMPLEMENTATIONS)]
    fn uv_ci(bencher: Bencher, implementation: Implementation) {
        run(bencher, "uv-ci", implementation);
    }

    #[divan::bench(name = "kubernetes-guestbook", args = IMPLEMENTATIONS)]
    fn kubernetes_guestbook(bencher: Bencher, implementation: Implementation) {
        run(bencher, "kubernetes-guestbook", implementation);
    }

    #[divan::bench(name = "openapi-petstore", args = IMPLEMENTATIONS)]
    fn openapi_petstore(bencher: Bencher, implementation: Implementation) {
        run(bencher, "openapi-petstore", implementation);
    }
}

mod deserialize_services_128 {
    use astral_yaml_dev::benchmarks::{ServiceConfig, documents, services_fixture};
    use divan::{Bencher, black_box};

    use super::{deser_documents, saphyr_options};

    #[divan::bench(name = "astral-yaml")]
    fn astral(bencher: Bencher) {
        let source = services_fixture();

        bencher.bench_local(|| {
            black_box(documents::<ServiceConfig>(black_box(&source)));
        });
    }

    #[divan::bench(name = "serde-saphyr")]
    fn saphyr(bencher: Bencher) {
        let source = services_fixture();

        bencher.bench_local(|| {
            black_box(
                serde_saphyr::from_multiple_with_options::<ServiceConfig>(
                    black_box(&source),
                    saphyr_options(),
                )
                .unwrap(),
            );
        });
    }

    #[divan::bench(name = "deser-yaml")]
    fn deser(bencher: Bencher) {
        let source = services_fixture();

        bencher.bench_local(|| {
            black_box(deser_documents::<ServiceConfig>(black_box(&source)));
        });
    }
}
