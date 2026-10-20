//! Short messages for CLI failures crossing the desktop boundary.

pub(super) fn unavailable_reason(error: &str) -> &'static str {
    let error = error.to_lowercase();
    if error.contains("permission denied")
        || error.contains("authentication failed")
        || error.contains("not authorized")
        || error.contains("access denied")
        || error.contains("authorization requires")
    {
        "No permission to manage VMs"
    } else if error.contains("domain not found") || error.contains("failed to get domain") {
        "VM not defined"
    } else if error.contains("no such file or directory")
        || error.contains("connection refused")
        || error.contains("failed to connect")
    {
        "libvirt is not running"
    } else {
        "VM status unavailable"
    }
}

pub(super) fn action_message(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    if let Some(message) = stderr.lines().filter_map(clean_line).next() {
        return message;
    }
    // Some CLI actions report a failure as structured stdout with a nonzero exit.
    if let Ok(report) = serde_json::from_slice::<serde_json::Value>(stdout) {
        if report["status"] == "not-signed-in" {
            return "Windows sign-in did not finish".into();
        }
        if report["failed"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
        {
            return "Windows Update could not install some updates".into();
        }
    }
    "Windows could not complete the action".into()
}

fn clean_line(line: &str) -> Option<String> {
    let line = line.trim().trim_start_matches("window-side:").trim();
    let line = line.strip_prefix("error:").unwrap_or(line).trim();
    let lower = line.to_lowercase();
    if line.is_empty()
        || lower.starts_with("stack backtrace")
        || lower.starts_with("caused by:")
        || lower.starts_with("at ")
        || line.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        return None;
    }
    if lower.contains("permission denied") || lower.contains("authentication failed") {
        return Some("No permission to complete the action".into());
    }
    // Do not expose credentials, command dumps or guest script bodies in UI errors.
    if [
        "password=",
        "password:",
        "token=",
        "token:",
        "secret=",
        "secret:",
        "credential=",
        "authorization:",
        "powershell",
        "-encodedcommand",
        "<domain",
        "{",
        "\u{1b}",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return Some("Windows could not complete the action".into());
    }
    let message: String = line.chars().filter(|c| !c.is_control()).take(240).collect();
    let mut chars = message.chars();
    let first = chars.next()?;
    Some(first.to_uppercase().collect::<String>() + chars.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_errors_keep_the_failure_without_prefix_or_trace() {
        assert_eq!(
            action_message(
                b"window-side: Start Windows first.\nStack backtrace:\n  0: internals",
                b""
            ),
            "Start Windows first."
        );
        assert_eq!(
            action_message(b"error: failed to connect: Permission denied", b""),
            "No permission to complete the action"
        );
        assert_eq!(
            action_message(b"window-side: token=private\nadditional-private-value\n", b""),
            "Windows could not complete the action"
        );
        assert_eq!(
            action_message(b"", br#"{"status":"not-signed-in"}"#),
            "Windows sign-in did not finish"
        );
        assert_eq!(
            action_message(b"", br#"{"failed":["update"]}"#),
            "Windows Update could not install some updates"
        );
        assert_eq!(
            action_message(b"window-side: The keyring did not store the credential", b""),
            "The keyring did not store the credential"
        );
        assert_eq!(
            action_message(b"error: boot entry is unavailable", b""),
            "Boot entry is unavailable"
        );
    }

    #[test]
    fn unavailable_errors_distinguish_access_service_and_missing_domain() {
        assert_eq!(
            unavailable_reason("Failed to connect: Permission denied"),
            "No permission to manage VMs"
        );
        assert_eq!(
            unavailable_reason("Failed to connect socket: No such file or directory"),
            "libvirt is not running"
        );
        assert_eq!(
            unavailable_reason("Domain not found: no domain with matching name"),
            "VM not defined"
        );
        assert_eq!(
            unavailable_reason("unexpected response"),
            "VM status unavailable"
        );
    }
}
