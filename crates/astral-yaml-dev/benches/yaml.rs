use std::collections::BTreeMap;

use astral_yaml_dev::{Value, benchmarks};
use astral_yaml_serde::{Spanned, from_document, from_str, to_string};
use benchmarks::FIXTURES;
use serde::Deserialize;

type Fields<'a> = Spanned<'a, BTreeMap<String, Spanned<'a, Value>>>;

fn documents<'a, T: Deserialize<'a>>(source: &'a str) -> Vec<T> {
    let stream = astral_yaml::parse(source).unwrap();

    (0..stream.documents().len())
        .map(|index| from_document(&stream, index).unwrap())
        .collect()
}

fn fixture(name: &str) -> &'static str {
    FIXTURES
        .iter()
        .find(|&&(candidate, _)| candidate == name)
        .unwrap()
        .1
}

fn main() {
    for &(_, source) in FIXTURES {
        let values = benchmarks::astral(source);
        assert_eq!(values, benchmarks::reference(source));

        #[cfg(feature = "saphyr")]
        assert_eq!(
            values,
            serde_saphyr::from_multiple_with_options::<Value>(
                source,
                astral_yaml_dev::saphyr::options(),
            )
            .unwrap()
        );

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

mod deserialize {
    use std::fmt;

    use astral_yaml_dev::{Value, benchmarks};
    use divan::{Bencher, black_box};

    use super::{documents, fixture};

    #[derive(Clone, Copy, Debug)]
    enum Implementation {
        Astral,
        YamlSerde,
        #[cfg(feature = "saphyr")]
        Saphyr,
    }

    impl fmt::Display for Implementation {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(match self {
                Self::Astral => "astral-yaml",
                Self::YamlSerde => "yaml_serde",
                #[cfg(feature = "saphyr")]
                Self::Saphyr => "serde-saphyr",
            })
        }
    }

    const IMPLEMENTATIONS: &[Implementation] = &[
        Implementation::Astral,
        Implementation::YamlSerde,
        #[cfg(feature = "saphyr")]
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
                black_box(benchmarks::reference(black_box(source)));
            }),
            #[cfg(feature = "saphyr")]
            Implementation::Saphyr => bencher.bench_local(|| {
                black_box(
                    serde_saphyr::from_multiple_with_options::<Value>(
                        black_box(source),
                        astral_yaml_dev::saphyr::options(),
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

mod astral {
    use astral_yaml_dev::{Value, benchmarks};
    use astral_yaml_serde::{Spanned, to_string};
    use benchmarks::FIXTURES;
    use divan::{Bencher, black_box};

    use super::{Fields, documents, fixture};

    // Drop outputs inside each measured closure; Divan defers dropping returned values.
    #[divan::bench(args = FIXTURES.iter().map(|&(name, _)| name))]
    fn parse(bencher: Bencher, name: &str) {
        let source = fixture(name);

        bencher.bench_local(|| {
            black_box(astral_yaml::parse(black_box(source)).unwrap());
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

#[cfg(feature = "saphyr")]
mod deserialize_services_128 {
    use std::{collections::BTreeMap, fmt::Write};

    use astral_yaml_dev::saphyr::options;
    use divan::{Bencher, black_box};
    use serde::Deserialize;

    use super::documents;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Config {
        services: Vec<Service>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Service {
        name: String,
        image: String,
        enabled: bool,
        replicas: u32,
        ports: Vec<u16>,
        environment: BTreeMap<String, String>,
        command: Option<Vec<String>>,
    }

    fn checked_fixture() -> String {
        let mut source = String::from("# Generated service configuration\nservices:\n");

        for index in 0..128 {
            writeln!(source, "  - name: service-{index}").unwrap();
            source.push_str("    image: example/app:v1\n    enabled: true\n    replicas: 3\n");
            source.push_str(
                "    ports: [8080, 8443]\n    environment: {LOG_LEVEL: info, REGION: west}\n",
            );
            source.push_str("    command: [serve, '--verbose'] # process arguments\n");
        }

        assert_eq!(
            documents::<Config>(&source),
            serde_saphyr::from_multiple_with_options::<Config>(&source, options()).unwrap()
        );

        source
    }

    #[divan::bench(name = "astral-yaml")]
    fn astral(bencher: Bencher) {
        let source = checked_fixture();

        bencher.bench_local(|| {
            black_box(documents::<Config>(black_box(&source)));
        });
    }

    #[divan::bench(name = "serde-saphyr")]
    fn saphyr(bencher: Bencher) {
        let source = checked_fixture();

        bencher.bench_local(|| {
            black_box(
                serde_saphyr::from_multiple_with_options::<Config>(black_box(&source), options())
                    .unwrap(),
            );
        });
    }
}
