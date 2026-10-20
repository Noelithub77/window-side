//! Installed coding agents and the read-only setup prompt copied by the UI.
use crate::config::home;
use std::env;
use std::path::{Path, PathBuf};

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
