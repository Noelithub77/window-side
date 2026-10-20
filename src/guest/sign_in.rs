//! Signs in to Windows by typing a Linux keyring credential on the VM's virtual keyboard.
use crate::guest::agent::signed_in_user;
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use oo7::Keyring;
use serde_json::{Value, json};
use std::io::{BufRead, IsTerminal, Write};
use std::process::Command;
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

const LABEL: &str = "window-side VM sign-in";

fn attributes(user: &str) -> [(&str, &str); 3] {
    [("application", "window-side"), ("purpose", "console-login"), ("user", user)]
}

/// The default Secret Service collection, unlocked; the desktop may ask for the login password.
async fn keyring() -> Result<Keyring> {
    let keyring = Keyring::new().await.context("No keyring is available; install GNOME Keyring or KWallet")?;
    keyring.unlock().await.context("The keyring stayed locked")?;
    Ok(keyring)
}

fn stored_credential(user: &str) -> Result<String> {
    zbus::block_on(async {
        let Some(item) = keyring().await?.search_items(&attributes(user)).await?.into_iter().next() else {
            return Ok(String::new());
        };
        String::from_utf8(item.secret().await?.to_vec()).context("The stored credential is not text")
    })
}

/// Types the credential only when nobody is signed in and one is stored.
pub fn run(domain: &str, user: &str, wait: Duration) -> Result<Value> {
    let current = signed_in_user(domain)?;
    if !current.is_empty() {
        return Ok(json!({"status": "signed-in", "user": current, "typed": false}));
    }
    let credential = stored_credential(user)?;
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

/// Reads the credential in the terminal without echoing it.
pub fn store_interactively(user: &str) -> Result<()> {
    if !std::io::stdin().is_terminal() {
        bail!("Run this in a terminal; it prompts for the Windows password.");
    }
    print!("Windows password or PIN (kept in your Linux keyring): ");
    std::io::stdout().flush()?;
    let echo = |flag: &str| Command::new("stty").arg(flag).status().is_ok_and(|status| status.success());
    let hidden = echo("-echo");
    let mut line = String::new();
    let read = std::io::stdin().lock().read_line(&mut line);
    if hidden {
        echo("echo");
    }
    println!();
    read?;
    store(user, line.trim_end_matches(['\n', '\r']))
}

pub fn is_stored(user: &str) -> bool {
    stored_credential(user).is_ok_and(|credential| !credential.is_empty())
}

/// Stores a credential once the virtual keyboard can type every character of it.
pub fn store(user: &str, credential: &str) -> Result<()> {
    if credential.is_empty() {
        bail!("Enter the Windows password or PIN.");
    }
    virsh_script("check", credential)?;
    zbus::block_on(async {
        let keyring = keyring().await?;
        keyring.create_item(LABEL, &attributes(user), credential, true).await.context("The keyring did not store the credential")
    })
}

pub fn forget(user: &str) -> Result<()> {
    zbus::block_on(async { keyring().await?.delete(&attributes(user)).await.context("Could not remove the credential") })
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
