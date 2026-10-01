//! Run with `cargo run -p astral-yaml-serde --example edit-config`.
use astral_yaml::{CollectionStyle, comments::CommentPlacement};
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    timeout: u64,
    servers: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "timeout: 30 # seconds\nservers: [primary, retired]\nextra: 'retained'\n";
    let mut config = Editable::<Config>::parse(source)?;
    config.edit(|syntax| {
        let root = syntax.root();
        let entries = syntax.mapping_entries(&root)?;
        syntax.replace(&entries[0].1, "60")?;
        syntax.remove_item(&entries[1].1, 1)?;
        syntax.insert_item(&entries[1].1, 1, "backup")?;
        syntax.set_collection_style(&entries[1].1, CollectionStyle::Block)?;
        syntax.insert_comment(
            &entries[1].1,
            0,
            " failover order",
            CommentPlacement::Leading,
        )?;
        syntax.insert_entry(&root, 3, "enabled", "true")
    })?;
    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup"]);
    print!("{}", config.to_yaml());
    Ok(())
}
