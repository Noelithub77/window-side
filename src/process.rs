use anyhow::{Context, Result, bail};
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

pub fn output(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("Could not run {program}"))
}

/// Stdout of a command that must succeed; its stderr becomes the error.
pub fn text(program: &str, args: &[&str]) -> Result<String> {
    checked(program, output(program, args)?)
}

/// Like `text`, but feeds `input` on stdin so secrets never reach the process list.
pub fn text_with_input(program: &str, args: &[&str], input: &str) -> Result<String> {
    checked(program, output_with_input(program, args, input)?)
}

pub fn output_with_input(program: &str, args: &[&str], input: &str) -> Result<Output> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Could not run {program}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).with_context(|| format!("Could not write to {program}"))?;
    }
    child.wait_with_output().with_context(|| format!("{program} did not finish"))
}

fn checked(program: &str, output: Output) -> Result<String> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if error.is_empty() {
        bail!("{program} exited with {}", output.status);
    }
    bail!("{error}")
}

pub fn has_command(program: &str) -> bool {
    env::var_os("PATH").is_some_and(|paths| {
        env::split_paths(&paths).any(|dir| is_executable(&dir.join(program)))
    })
}

pub fn require(program: &str) -> Result<()> {
    if has_command(program) { Ok(()) } else { bail!("Required command is missing: {program}") }
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// Runs a configured hook given as an argument vector; `None` when no hook is set.
pub fn hook(argv: &[String]) -> Option<Result<Output>> {
    let (program, args) = argv.split_first()?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    Some(output(program, &args))
}
