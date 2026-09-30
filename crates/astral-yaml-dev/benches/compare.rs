use astral_yaml_dev::{Value, benchmarks};
use benchmarks::{FIXTURES, ServiceConfig, documents, fixture, services_fixture};
use serde::Deserialize;

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
    for &(_, source) in FIXTURES {
        let values = benchmarks::astral(source);
        assert_eq!(values, yaml_serde_documents(source));
        assert_eq!(
            values,
            serde_saphyr::from_multiple_with_options::<Value>(source, saphyr_options()).unwrap()
        );
    }

    let source = services_fixture();
    assert_eq!(
        documents::<ServiceConfig>(&source),
        serde_saphyr::from_multiple_with_options::<ServiceConfig>(&source, saphyr_options())
            .unwrap()
    );

    divan::main();
}

mod deserialize {
    use std::fmt;

    use astral_yaml_dev::Value;
    use divan::{Bencher, black_box};

    use super::{documents, fixture, saphyr_options, yaml_serde_documents};

    #[derive(Clone, Copy, Debug)]
    enum Implementation {
        Astral,
        YamlSerde,
        Saphyr,
    }

    impl fmt::Display for Implementation {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(match self {
                Self::Astral => "astral-yaml",
                Self::YamlSerde => "yaml_serde",
                Self::Saphyr => "serde-saphyr",
            })
        }
    }

    const IMPLEMENTATIONS: &[Implementation] = &[
        Implementation::Astral,
        Implementation::YamlSerde,
        Implementation::Saphyr,
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

    use super::saphyr_options;

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
}
