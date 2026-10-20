//! Local coding agents that can run setup from a prompt, opened in the user's terminal.
use crate::config::{config_dir, home};
use anyhow::{Context, Result};
use std::env;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const SETUP_PROMPT: &str = include_str!("../assets/agent-setup.md");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Codex,
    OpenCode,
}

impl Agent {
    pub const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::OpenCode];

    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::OpenCode => "OpenCode",
        }
    }

    fn program(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
        }
    }

    /// Interactive session that starts from `prompt`.
    fn args(self, prompt: &str) -> Vec<&str> {
        match self {
            Self::Claude | Self::Codex => vec![prompt],
            Self::OpenCode => vec!["--prompt", prompt],
        }
    }
}

fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// PATH first, then the per-user install folders a desktop session's PATH often lacks.
fn find(program: &str) -> Option<PathBuf> {
    let home = home();
    let fallbacks = [".local/bin", ".bun/bin", ".npm-global/bin", ".opencode/bin", ".claude/local", ".cargo/bin"];
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .chain(fallbacks.iter().map(|folder| home.join(folder)))
        .map(|folder| folder.join(program))
        .find(|path| executable(path))
}

pub fn detect() -> Vec<Agent> {
    Agent::ALL.into_iter().filter(|agent| find(agent.program()).is_some()).collect()
}

/// Terminals and the arguments that come before the command they run.
const TERMINALS: [(&str, &[&str]); 11] = [
    ("xdg-terminal-exec", &[]),
    ("konsole", &["-e"]),
    ("ptyxis", &["--"]),
    ("gnome-terminal", &["--"]),
    ("kgx", &["--"]),
    ("ghostty", &["-e"]),
    ("alacritty", &["-e"]),
    ("kitty", &[]),
    ("foot", &[]),
    ("wezterm", &["start", "--"]),
    ("xterm", &["-e"]),
];

/// The desktop's own terminal first, after the standard `xdg-terminal-exec`.
fn terminal() -> Option<(PathBuf, &'static [&'static str])> {
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_ascii_lowercase();
    let native: &[&str] = if desktop.contains("kde") {
        &["konsole"]
    } else if desktop.contains("gnome") {
        &["ptyxis", "gnome-terminal", "kgx"]
    } else {
        &[]
    };
    let ordered = TERMINALS.iter().filter(|(name, _)| *name == "xdg-terminal-exec").chain(
        native.iter().filter_map(|name| TERMINALS.iter().find(|(known, _)| known == name)),
    );
    ordered
        .chain(TERMINALS.iter())
        .find_map(|(name, prefix)| find(name).map(|path| (path, *prefix)))
}

fn command(agent: Agent, program: &Path, terminal: &Path, prefix: &[&str]) -> Command {
    let mut command = Command::new(terminal);
    command.args(prefix).arg(program).args(agent.args(SETUP_PROMPT));
    command
}

/// Opens `agent` in a terminal with the setup prompt; the session outlives the window.
pub fn launch(agent: Agent) -> Result<()> {
    let program = find(agent.program()).with_context(|| format!("{} is not installed", agent.name()))?;
    let (terminal, prefix) = terminal().context("No terminal was found to open the agent in")?;
    let directory = config_dir();
    std::fs::create_dir_all(&directory)?;
    let mut child = command(agent, &program, &terminal, prefix)
        .current_dir(directory)
        .process_group(0)
        .spawn()
        .with_context(|| format!("Could not open {}", agent.name()))?;
    std::thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_agent_starts_interactively_from_the_prompt() {
        assert_eq!(Agent::Claude.args("go"), ["go"]);
        assert_eq!(Agent::OpenCode.args("go"), ["--prompt", "go"]);
        assert!(SETUP_PROMPT.contains("ws setup detect") && SETUP_PROMPT.contains("Ask me before running `ws setup apply`"));
        let command = command(Agent::OpenCode, Path::new("/bin/opencode"), Path::new("/bin/ptyxis"), &["--"]);
        let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert_eq!(command.get_program(), "/bin/ptyxis");
        assert_eq!(args, ["--", "/bin/opencode", "--prompt", SETUP_PROMPT]);
    }
}
