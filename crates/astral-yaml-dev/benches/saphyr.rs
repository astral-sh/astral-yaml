use std::{
    collections::BTreeMap,
    fmt::{Debug, Write},
    hint::black_box,
    time::{Duration, Instant},
};

use astral_yaml_dev::Value;
use astral_yaml_dev::{
    benchmarks::FIXTURES,
    saphyr::{astral_documents, options},
};
use serde::Deserialize;

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

fn typed_fixture() -> String {
    let mut source = String::from("# Generated service configuration\nservices:\n");

    for index in 0..128 {
        writeln!(source, "  - name: service-{index}").unwrap();
        source.push_str("    image: example/app:v1\n    enabled: true\n    replicas: 3\n");
        source.push_str(
            "    ports: [8080, 8443]\n    environment: {LOG_LEVEL: info, REGION: west}\n",
        );
        source.push_str("    command: [serve, '--verbose'] # process arguments\n");
    }

    source
}

fn sample<T>(mut operation: impl FnMut() -> T) -> (f64, u32) {
    let start = Instant::now();
    let mut iterations = 0;

    while start.elapsed() < Duration::from_millis(200) {
        for _ in 0..16 {
            black_box(operation());
        }

        iterations += 16;
    }

    (
        start.elapsed().as_secs_f64() * 1e9 / f64::from(iterations),
        iterations,
    )
}

fn compare<T: serde::de::DeserializeOwned + Debug + PartialEq>(
    run: u32,
    fixture: &str,
    mode: &str,
    source: &str,
) {
    let astral = || astral_documents::<T>(black_box(source)).unwrap();
    let saphyr =
        || serde_saphyr::from_multiple_with_options::<T>(black_box(source), options()).unwrap();
    assert_eq!(astral(), saphyr(), "{fixture}: {mode}");

    for _ in 0..32 {
        black_box(astral());
        black_box(saphyr());
    }

    for sample_index in 0..11 {
        let order = if (run + sample_index) % 2 == 0 {
            ["astral", "saphyr"]
        } else {
            ["saphyr", "astral"]
        };

        for implementation in order {
            let (elapsed, iterations) = if implementation == "astral" {
                sample(astral)
            } else {
                sample(saphyr)
            };

            println!(
                "{run},{fixture},{},{mode},{implementation},{},{elapsed:.3},{iterations}",
                source.len(),
                sample_index + 1
            );
        }
    }
}

fn main() {
    println!("run,fixture,bytes,mode,implementation,sample,ns_per_iteration,iterations");
    let typed = typed_fixture();

    for run in 1..=3 {
        for &(fixture, source) in FIXTURES {
            compare::<Value>(run, fixture, "value", source);
        }

        compare::<Config>(run, "services-128", "typed", &typed);
    }
}
