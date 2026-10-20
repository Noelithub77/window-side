//! Read-only boot detection. An unreadable or ambiguous running entry disables GPU setup.
use crate::process::{has_command, text};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Loader { Limine, Grub, SystemdBoot }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Initramfs { Mkinitcpio, Tools, Dracut }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootProfile {
    pub loader: Loader,
    pub initramfs: Initramfs,
    pub kernel: String,
    pub source: String,
    pub entry: String,
    pub contents: String,
}

pub fn safe_name(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric) && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

pub fn bls_path(path: &str) -> bool {
    ["/boot", "/boot/efi", "/efi"].iter().any(|root| {
        path.strip_prefix(&format!("{root}/loader/entries/")).is_some_and(|name| {
            name.strip_suffix(".conf").is_some_and(safe_name)
        })
    })
}

pub fn bls_value<'a>(entry: &'a str, key: &str) -> Option<&'a str> {
    entry.lines().find_map(|line| {
        let (name, value) = line.trim().split_once(char::is_whitespace)?;
        (name == key).then_some(value.trim())
    })
}

pub fn running_bls(entry: &str, release: &str) -> bool {
    let linux = bls_value(entry, "linux").unwrap_or_default();
    // Type 2 UKIs can have a signed embedded command line: do not pretend it is editable.
    !entry.lines().any(|line| line.trim_start().starts_with("efi "))
        && (linux.ends_with(&format!("/vmlinuz-{release}"))
            || (bls_value(entry, "version") == Some(release) && linux.starts_with('/')))
        && bls_value(entry, "initrd").is_some()
        && bls_value(entry, "options").is_some_and(|options| !options.contains('$'))
}

/// Extract the running kernel's generated menu body, preserving search, modules and initrds.
pub fn grub_entry(config: &str, release: &str) -> Option<String> {
    let mut body = Vec::new();
    let mut active = false;
    for line in config.lines() {
        if line.trim_start().starts_with("menuentry ") {
            active = true;
            body.clear();
        } else if active && line.trim() == "}" {
            let found = body.iter().any(|line: &&str| {
                let words: Vec<_> = line.split_whitespace().collect();
                matches!(words.first(), Some(&"linux" | &"linuxefi"))
                    && words.get(1).is_some_and(|path| path.ends_with(&format!("/vmlinuz-{release}")))
            });
            if found { return Some(body.join("\n")); }
            active = false;
        } else if active {
            // Nested shell blocks need a real parser; refuse instead of producing a partial entry.
            if line.trim_end().ends_with('{') { active = false; }
            body.push(line);
        }
    }
    None
}

fn initramfs() -> Option<Initramfs> {
    if has_command("update-initramfs") { Some(Initramfs::Tools) }
    else if has_command("dracut") { Some(Initramfs::Dracut) }
    else if has_command("mkinitcpio") { Some(Initramfs::Mkinitcpio) }
    else { None }
}

fn selected_systemd() -> Option<String> {
    let variable = fs::read_dir("/sys/firmware/efi/efivars").ok()?.flatten()
        .find(|entry| entry.file_name().to_string_lossy().starts_with("LoaderEntrySelected-"))?;
    let bytes = fs::read(variable.path()).ok()?;
    let units: Vec<u16> = bytes.get(4..)?.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0).collect();
    String::from_utf16(&units).ok().filter(|name| name.ends_with(".conf") && safe_name(name))
}

pub fn detect() -> Option<BootProfile> {
    let release = fs::read_to_string("/proc/sys/kernel/osrelease").ok()?.trim().to_owned();
    if !safe_name(&release) { return None; }
    if has_command("limine-entry-tool") && has_command("limine-mkinitcpio") && Path::new("/etc/default/limine").exists() {
        let kernel = text("pacman", &["-Qoq", &format!("/usr/lib/modules/{release}/vmlinuz")]).ok()?.trim().to_owned();
        if !safe_name(&kernel) { return None; }
        return Some(BootProfile { loader: Loader::Limine, initramfs: Initramfs::Mkinitcpio, kernel,
            source: "/etc/default/limine".into(), entry: String::new(), contents: String::new() });
    }
    let initramfs = initramfs()?;
    if let Some(selected) = selected_systemd() {
        if !has_command("bootctl") { return None; }
        for root in ["/boot", "/efi", "/boot/efi"] {
            let mut source = format!("{root}/loader/entries/{selected}");
            let selected_contents = fs::read_to_string(&source).ok();
            let mut entry = selected.clone();
            if selected == "window-side-gpu.conf" {
                let original = selected_contents.as_ref()?.lines().find_map(|line| line.strip_prefix("# window-side-source: "))?;
                if !safe_name(original) || original == selected { return None; }
                source = format!("{root}/loader/entries/{original}");
                entry = original.into();
            }
            if let Ok(contents) = fs::read_to_string(&source)
                && running_bls(&contents, &release) {
                    return Some(BootProfile { loader: Loader::SystemdBoot, initramfs, kernel: release, source, entry, contents });
            }
        }
        return None;
    }
    if initramfs == Initramfs::Dracut && has_command("grubby") && has_command("grub2-reboot") {
        let info = text("grubby", &[&format!("--info=/boot/vmlinuz-{release}")]).ok()?;
        let info = info.split("index=").find(|section| section.lines().any(|line| line.starts_with("id=") && line.trim() != "id=\"window-side-gpu\"" && line.trim() != "id=window-side-gpu"))?;
        let value = |key: &str| info.lines().find_map(|line| line.strip_prefix(key)).map(|v| v.trim_matches('"').to_owned());
        let id = value("id=")?;
        if !safe_name(&id) { return None; }
        let source = format!("/boot/loader/entries/{id}.conf");
        let mut contents = fs::read_to_string(&source).ok()?;
        if bls_value(&contents, "options").is_some_and(|v| v.contains('$')) {
            let args = value("args=")?;
            contents = contents.lines().map(|line| if line.starts_with("options ") { format!("options {args}") } else { line.to_owned() }).collect::<Vec<_>>().join("\n");
        }
        if !running_bls(&contents, &release) { return None; }
        return Some(BootProfile { loader: Loader::Grub, initramfs, kernel: release, source, entry: id, contents });
    }
    if initramfs == Initramfs::Tools && has_command("grub-reboot") && has_command("update-grub") {
        let source = "/boot/grub/grub.cfg";
        let config = fs::read_to_string(source).ok()?;
        let contents = grub_entry(&config, &release)?;
        return Some(BootProfile { loader: Loader::Grub, initramfs, kernel: release, source: source.into(), entry: "window-side-normal".into(), contents });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bls_options_and_paths_are_strict() {
        assert!(bls_path("/boot/loader/entries/linux.conf"));
        assert!(bls_path("/efi/loader/entries/linux.conf"));
        assert!(!bls_path("/boot/loader/entries/../linux.conf"));
        assert_eq!(bls_value("options\troot=UUID=x quiet\n", "options"), Some("root=UUID=x quiet"));
        assert!(!safe_name("../help"));
    }
}
