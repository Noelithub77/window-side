//! Signs in to Windows by typing a Linux keyring credential on the VM's virtual keyboard.
use crate::guest::agent::signed_in_user;
use crate::process::output;
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const SHIFTED: &[(char, &str)] = &[
    ('~', "GRAVE"), ('!', "1"), ('@', "2"), ('#', "3"), ('$', "4"), ('%', "5"), ('^', "6"), ('&', "7"),
    ('*', "8"), ('(', "9"), (')', "0"), ('_', "MINUS"), ('+', "EQUAL"), ('{', "LEFTBRACE"), ('}', "RIGHTBRACE"),
    ('|', "BACKSLASH"), (':', "SEMICOLON"), ('"', "APOSTROPHE"), ('<', "COMMA"), ('>', "DOT"), ('?', "SLASH"),
];
const PLAIN: &[(char, &str)] = &[
    ('`', "GRAVE"), ('-', "MINUS"), ('=', "EQUAL"), ('[', "LEFTBRACE"), (']', "RIGHTBRACE"), ('\\', "BACKSLASH"),
    (';', "SEMICOLON"), ('\'', "APOSTROPHE"), (',', "COMMA"), ('.', "DOT"), ('/', "SLASH"), (' ', "SPACE"),
];

/// evdev keys for one character on a US layout, Shift first when needed.
fn keys_for(character: char) -> Result<Vec<String>> {
    let lookup = |table: &[(char, &str)]| table.iter().find(|(key, _)| *key == character).map(|(_, name)| format!("KEY_{name}"));
    let keys = if character.is_ascii_alphabetic() {
        let key = format!("KEY_{}", character.to_ascii_uppercase());
        if character.is_ascii_uppercase() { vec!["KEY_LEFTSHIFT".into(), key] } else { vec![key] }
    } else if character.is_ascii_digit() {
        vec![format!("KEY_{character}")]
    } else if let Some(key) = lookup(PLAIN) {
        vec![key]
    } else if let Some(key) = lookup(SHIFTED) {
        vec!["KEY_LEFTSHIFT".into(), key]
    } else {
        bail!("The stored credential has a character the virtual keyboard cannot type.")
    };
    Ok(keys)
}

fn virsh_script(domain: &str, credential: &str) -> Result<String> {
    let mut lines = credential
        .chars()
        .map(|character| Ok(format!("send-key {domain} --codeset linux {}", keys_for(character)?.join(" "))))
        .collect::<Result<Vec<_>>>()?;
    lines.push(format!("send-key {domain} --codeset linux KEY_ENTER"));
    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn attributes(user: &str) -> [&str; 6] {
    ["application", "window-side", "purpose", "console-login", "user", user]
}

fn stored_credential(user: &str) -> String {
    let mut args = vec!["lookup"];
    args.extend(attributes(user));
    output("secret-tool", &args)
        .ok()
        .filter(|result| result.status.success())
        .map(|result| String::from_utf8_lossy(&result.stdout).trim_end_matches('\n').to_owned())
        .unwrap_or_default()
}

/// Types the credential only when nobody is signed in and one is stored.
pub fn run(domain: &str, user: &str, wait: Duration) -> Result<Value> {
    let current = signed_in_user(domain)?;
    if !current.is_empty() {
        return Ok(json!({"status": "signed-in", "user": current, "typed": false}));
    }
    let credential = stored_credential(user);
    if credential.is_empty() {
        return Ok(json!({"status": "no-credential", "typed": false}));
    }
    let script = virsh_script(domain, &credential)?;
    // Ctrl lifts the lock screen without typing into the password field.
    virsh::run_script(&format!("send-key {domain} --codeset linux KEY_LEFTCTRL\n"))?;
    sleep(Duration::from_secs(2));
    virsh::run_script(&script).context("The VM did not accept the virtual keystrokes")?;
    let deadline = Instant::now() + wait;
    while Instant::now() < deadline {
        sleep(Duration::from_secs(2));
        let current = signed_in_user(domain)?;
        if !current.is_empty() {
            return Ok(json!({"status": "signed-in", "user": current, "typed": true}));
        }
    }
    Ok(json!({"status": "not-signed-in", "typed": true}))
}

/// Lets secret-tool prompt in the terminal so the credential never passes through ws.
pub fn store_interactively(user: &str) -> Result<()> {
    if !std::io::stdin().is_terminal() {
        bail!("Run this in a terminal; secret-tool prompts for the Windows password.");
    }
    println!("Enter the Windows sign-in password or PIN. It goes straight to your Linux keyring.");
    let status = Command::new("secret-tool")
        .arg("store")
        .arg("--label=window-side VM sign-in")
        .args(attributes(user))
        .status()
        .context("Could not run secret-tool")?;
    if !status.success() {
        bail!("secret-tool did not store the credential");
    }
    if let Err(error) = virsh_script("check", &stored_credential(user)) {
        forget(user)?;
        return Err(error);
    }
    Ok(())
}

pub fn is_stored(user: &str) -> bool {
    !stored_credential(user).is_empty()
}

/// Stores a credential typed into the window. It reaches secret-tool on stdin, never on argv.
pub fn store(user: &str, credential: &str) -> Result<()> {
    if credential.is_empty() {
        bail!("Enter the Windows password or PIN.");
    }
    virsh_script("check", credential)?;
    let mut child = Command::new("secret-tool")
        .arg("store")
        .arg("--label=window-side VM sign-in")
        .args(attributes(user))
        .stdin(Stdio::piped())
        .spawn()
        .context("Could not run secret-tool")?;
    child
        .stdin
        .take()
        .context("Could not open secret-tool input")?
        .write_all(credential.as_bytes())?;
    if !child.wait()?.success() {
        bail!("secret-tool did not store the credential");
    }
    Ok(())
}

pub fn forget(user: &str) -> Result<()> {
    let mut args = vec!["clear"];
    args.extend(attributes(user));
    crate::process::text("secret-tool", &args).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_follow_the_us_layout() {
        assert_eq!(keys_for('a').unwrap(), ["KEY_A"]);
        assert_eq!(keys_for('A').unwrap(), ["KEY_LEFTSHIFT", "KEY_A"]);
        assert_eq!(keys_for('7').unwrap(), ["KEY_7"]);
        assert_eq!(keys_for('@').unwrap(), ["KEY_LEFTSHIFT", "KEY_2"]);
        assert_eq!(keys_for('?').unwrap(), ["KEY_LEFTSHIFT", "KEY_SLASH"]);
        assert_eq!(keys_for(' ').unwrap(), ["KEY_SPACE"]);
    }

    #[test]
    fn untypeable_characters_are_rejected_before_sending() {
        assert!(virsh_script("vm", "pässword").is_err());
    }

    #[test]
    fn script_ends_with_enter() {
        let script = virsh_script("vm", "Ab1").unwrap();
        let lines: Vec<&str> = script.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "send-key vm --codeset linux KEY_LEFTSHIFT KEY_A");
        assert_eq!(lines[3], "send-key vm --codeset linux KEY_ENTER");
    }
}
