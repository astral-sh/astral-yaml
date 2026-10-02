//! Run with `cargo run -p astral-yaml-serde --example edit-config`.
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
    let source = "timeout: 30 # seconds\nservers: [primary, retired]\nextra: 'retained'\n";
    let mut config = Editable::<Config>::parse(source)?;
    config.edit(|editor| {
        editor.root_mut().field("timeout")?.set(60)?;

        let mut servers = editor.root_mut().field("servers")?;
        servers.reborrow().item(1)?.set("backup")?;
        servers.set_collection_style(CollectionStyle::Block)?;
        servers.append_comment(" failover order", CommentPlacement::Leading)?;

        editor.root_mut().set_entry("enabled", true)?;
        Ok(())
    })?;
    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup"]);
    print!("{}", config.source());
    Ok(())
}
