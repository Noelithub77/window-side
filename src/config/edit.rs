//! `ws config`: read, check and change `config.json` as JSON for people and agents.
use super::{Config, SCHEMA_FILE, config_dir, schema};
use crate::setup;
use crate::vm::{lifecycle, virsh};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the config and schema paths as JSON
    Path,
    /// Print the whole config as JSON, defaults included
    Show,
    /// Print one value as JSON, e.g. `ws config get vm.memory_mib`
    Get { key: String },
    /// Change one value; VM values redefine stopped Windows, e.g. `ws config set vm.vcpus 8`
    Set {
        key: String,
        /// JSON value; text fields take plain text
        value: String,
    },
    /// Check the config file; exits 1 with the reason when it is invalid
    Validate,
    /// Print the JSON Schema of config.json
    Schema,
}

/// What a key change needs beyond the file.
#[derive(Debug, PartialEq, Eq)]
enum Effect {
    File,
    Domain,
}

fn effect(key: &str) -> Result<Effect> {
    match key {
        "vm.memory_mib" | "vm.vcpus" => Ok(Effect::Domain),
        "windows.user" | "windows.refresh_hz" | "boot.normal_entry" | "boot.gpu_entry" | "hooks.before_start"
        | "hooks.storage_status" => Ok(Effect::File),
        "vm.virtio" | "windows.provisioned" => bail!("{key} is managed by `ws setup first-boot`"),
        _ if ["vm.", "disk.", "gpu"].iter().any(|prefix| key.starts_with(prefix)) => {
            bail!("{key} changes system files; use `ws setup plan` and `ws setup apply`")
        }
        _ => bail!("Unknown key {key}; see `ws config schema`"),
    }
}

fn pointer(key: &str) -> String {
    format!("/{}", key.replace('.', "/"))
}

fn print(value: &Value) -> Result<i32> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(0)
}

pub fn run(action: ConfigAction) -> Result<i32> {
    match action {
        ConfigAction::Path => print(&json!({
            "config": Config::path(),
            "schema": config_dir().join(SCHEMA_FILE),
            "domain": lifecycle::domain_file(),
        })),
        ConfigAction::Schema => print(&schema()),
        ConfigAction::Show => print(&serde_json::to_value(Config::load()?)?),
        ConfigAction::Get { key } => {
            let config = serde_json::to_value(Config::load()?)?;
            print(config.pointer(&pointer(&key)).with_context(|| format!("Unknown key {key}; see `ws config schema`"))?)
        }
        ConfigAction::Validate => {
            let config = Config::load()?;
            config.validate()?;
            print(&json!({ "valid": true, "config": Config::path() }))
        }
        ConfigAction::Set { key, value } => set(&key, &value),
    }
}

fn set(key: &str, raw: &str) -> Result<i32> {
    let effect = effect(key)?;
    let config = Config::load()?;
    let mut value = serde_json::to_value(&config)?;
    let slot = value.pointer_mut(&pointer(key)).with_context(|| format!("Unknown key {key}; see `ws config schema`"))?;
    let before = slot.clone();
    // String fields take the raw text, so `set windows.refresh_hz 75` needs no JSON quotes.
    *slot = match serde_json::from_str(raw) {
        Ok(Value::String(text)) => Value::String(text),
        Ok(parsed) if !before.is_string() => parsed,
        _ => Value::String(raw.to_owned()),
    };
    let after = slot.clone();
    let updated: Config = serde_json::from_value(value).with_context(|| format!("{key} cannot be {after}"))?;
    updated.validate()?;
    let redefined = effect == Effect::Domain && before != after;
    if redefined {
        if virsh::is_running(&config.vm.name) {
            bail!("Shut down Windows before changing {key}.");
        }
        let inventory = setup::detect()?;
        let choices = inventory
            .default_choices(Some(&updated))
            .context("The configured Windows install was not detected")?;
        // The plan checks host limits and renders the VM exactly as setup would.
        let plan = setup::plan(&inventory, &choices, Some(&updated))?;
        lifecycle::redefine(&updated, &plan.domain_xml)?;
    } else {
        updated.save()?;
    }
    print(&json!({ "key": key, "before": before, "after": after, "redefined": redefined, "config": Config::path() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_map_to_what_they_change() {
        assert_eq!(effect("vm.memory_mib").ok(), Some(Effect::Domain));
        assert_eq!(effect("boot.gpu_entry").ok(), Some(Effect::File));
        assert!(effect("disk.id").is_err());
        assert!(effect("gpu.ids").is_err());
        assert!(effect("vm.virtio").is_err());
        assert!(effect("nope").is_err());
        assert_eq!(pointer("vm.memory_mib"), "/vm/memory_mib");
    }
}
