//! Standalone dependency setup; no hardware detection, VM changes or boot writes.
use crate::detect::distro::{self, Family};
use crate::process::{has_command, output};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::process::{Command, Stdio};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    environment: &'static str,
    distro: Option<Family>,
    desktop: String,
    omarchy: bool,
    missing: Vec<String>,
    installed: Vec<String>,
    status: &'static str,
}

fn installed(family: Family, package: &str) -> Result<bool> {
    let (program, args) = match family {
        Family::Arch => ("pacman", vec!["-Q", "--", package]),
        Family::Debian => ("dpkg-query", vec!["-W", "-f=${db:Status-Status}", package]),
        Family::Fedora => ("rpm", vec!["-q", "--quiet", "--", package]),
    };
    let result = output(program, &args).with_context(|| format!("Cannot query packages: {program} is unavailable"))?;
    Ok(result.status.success() && (family != Family::Debian || result.stdout == b"installed"))
}

fn missing(family: Family) -> Result<Vec<String>> {
    let mut missing = Vec::new();
    for package in family.required_packages() {
        if !installed(family, package)? { missing.push(package.to_owned()); }
    }
    Ok(missing)
}

fn install_args(family: Family, packages: &[String]) -> Result<(&'static str, Vec<String>)> {
    if packages.iter().any(|package| !family.known(package)) { bail!("Refusing unknown packages"); }
    let (program, flags) = family.installer();
    Ok((program, flags.iter().map(|flag| (*flag).to_owned()).chain(packages.iter().cloned()).collect()))
}

fn run_installer(family: Family, packages: &[String]) -> Result<()> {
    let (program, args) = install_args(family, packages)?;
    if family == Family::Debian {
        run_privileged("apt-get", &["update".into()])?;
    }
    run_privileged(program, &args)
}

fn run_privileged(program: &str, args: &[String]) -> Result<()> {
    let mut command = if has_command("pkexec") {
        let mut command = Command::new("pkexec");
        command.arg(program);
        command
    } else if has_command("sudo") {
        let mut command = Command::new("sudo");
        command.arg("--").arg(program);
        command
    } else { bail!("Install pkexec or sudo, then rerun ws setup packages --install"); };
    let status = command.args(args).stdin(Stdio::inherit()).stdout(Stdio::from(std::io::stderr())).stderr(Stdio::inherit())
        .status().context("Could not start package installation")?;
    if !status.success() { bail!("Package installation failed ({status}); rerun ws setup packages --install to retry"); }
    Ok(())
}

pub fn run(install: bool) -> Result<Report> {
    let family = distro::detect();
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into());
    let omarchy = has_command("omarchy");
    eprintln!("Environment: distro family={}, desktop={}, Omarchy={}",
        family.map_or("unsupported", Family::name), desktop, if omarchy { "yes" } else { "no" });
    let mut report = Report { environment: "local-packages", distro: family, desktop, omarchy,
        missing: Vec::new(), installed: Vec::new(), status: "unsupported" };
    let Some(family) = family else {
        if install { bail!("Unsupported distro family; install the VM tools listed in ws setup detect with your package manager"); }
        return Ok(report);
    };
    report.missing = missing(family)?;
    if install && !report.missing.is_empty() {
        run_installer(family, &report.missing)?;
        let remaining = missing(family)?;
        if !remaining.is_empty() { bail!("Package installation did not provide: {}", remaining.join(", ")); }
        report.installed = std::mem::take(&mut report.missing);
    }
    report.status = if report.missing.is_empty() { "ready" } else { "missing" };
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_installers_only_receive_known_family_packages() -> Result<()> {
        for family in [Family::Arch, Family::Debian, Family::Fedora] {
            let packages = family.required_packages().iter().map(|package| (*package).to_owned()).collect::<Vec<_>>();
            let (program, args) = install_args(family, &packages)?;
            assert_eq!(program, family.installer().0);
            assert!(packages.iter().all(|package| args.contains(package)));
            assert!(install_args(family, &["--bad-option".into()]).is_err());
        }
        Ok(())
    }
}
