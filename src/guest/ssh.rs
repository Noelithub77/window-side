use crate::config::{Config, home, state_dir};
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use std::os::unix::process::CommandExt;
use std::process::Command;

pub const HELP: &str = "\
Usage: ws ssh [PowerShell command]
       ws ssh --put LOCAL 'C:/path/file'
       ws ssh --get 'C:/path/file' LOCAL
Connect as the configured Windows user through key-only SSH to the running guest.
Requires Windows OpenSSH and a host key pinned through the guest agent.";

/// The guest's single IPv4 lease on libvirt's default network.
fn guest_address(domain: &str) -> Result<String> {
    let leases = virsh::run(&["domifaddr", domain, "--source", "lease"])?;
    let addresses: Vec<&str> = leases
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            (fields.get(2) == Some(&"ipv4")).then(|| fields.get(3).and_then(|cidr| cidr.split('/').next())).flatten()
        })
        .collect();
    match addresses.as_slice() {
        [address] if address.starts_with("192.168.122.") => Ok((*address).to_owned()),
        _ => bail!("Expected exactly one libvirt guest address on the default VM bridge."),
    }
}

/// Replaces this process with ssh or scp.
pub fn run(config: &Config, args: &[String]) -> Result<i32> {
    if args.first().map(String::as_str) == Some("--help") {
        println!("{HELP}");
        return Ok(0);
    }
    let known_hosts = state_dir().join("ssh_known_hosts");
    if !known_hosts.exists() {
        bail!("Pin the Windows SSH host key through the guest agent first.");
    }
    let address = guest_address(&config.vm.name)?;
    let options = [
        "-F".to_owned(), "/dev/null".to_owned(),
        "-i".to_owned(), home().join(".ssh/id_ed25519").display().to_string(),
        "-o".to_owned(), "BatchMode=yes".to_owned(),
        "-o".to_owned(), "IdentitiesOnly=yes".to_owned(),
        "-o".to_owned(), "ConnectTimeout=8".to_owned(),
        "-o".to_owned(), "StrictHostKeyChecking=yes".to_owned(),
        "-o".to_owned(), "HostKeyAlias=window-side".to_owned(),
        "-o".to_owned(), format!("UserKnownHostsFile={}", known_hosts.display()),
        "-o".to_owned(), format!("User={}", config.windows.user),
    ];
    let mut command = match args {
        [flag, source, target] if flag == "--put" => {
            let mut scp = Command::new("scp");
            scp.args(&options).arg("--").arg(source).arg(format!("{address}:{target}"));
            scp
        }
        [flag, source, target] if flag == "--get" => {
            let mut scp = Command::new("scp");
            scp.args(&options).arg("--").arg(format!("{address}:{source}")).arg(target);
            scp
        }
        [flag, ..] if flag == "--put" || flag == "--get" => bail!("Give a source and destination; use --help."),
        _ => {
            let mut ssh = Command::new("ssh");
            ssh.args(&options).arg(&address).args(args);
            ssh
        }
    };
    Err(command.exec()).context("Could not start ssh")
}
