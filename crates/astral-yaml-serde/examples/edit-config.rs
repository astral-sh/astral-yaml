//! Run with `cargo run -p astral-yaml-serde --features editable --example edit-config`.
use astral_yaml_serde::{
    Editable,
    edit::{CollectionStyle, CommentPlacement},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    timeout: u64,
    servers: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r"timeout: 30 # seconds
servers: [primary, retired]
extra: 'retained'
";
    let mut config = Editable::<Config>::parse(source)?;
    config.edit(|root| {
        let mut settings = root.into_mapping()?;
        settings
            .field("timeout")?
            .update(|seconds: u64| seconds + 30)?;
        settings.entry("enabled")?.or_insert(true)?;

        let mut servers = settings.field("servers")?.into_sequence()?;
        servers.item(1)?.set("backup")?;
        servers.set_collection_style(CollectionStyle::Block)?;
        servers.append_comment(" failover order", CommentPlacement::Leading)?;
        Ok(())
    })?;
    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup"]);
    print!("{}", config.source());
    Ok(())
}
