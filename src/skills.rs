//! Agent skills bundled with ws, installed where Claude Code, Codex and OpenCode look for them.
use crate::agents::{self, Agent};
use crate::config::{home, write_atomic};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const SKILLS: [(&str, &str); 3] = [
    ("window-side", include_str!("../skills/window-side/SKILL.md")),
    ("windows-app-testing", include_str!("../skills/windows-app-testing/SKILL.md")),
    ("window-side-setup", include_str!("../skills/window-side-setup/SKILL.md")),
];

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum SkillsAction {
    /// Show where each skill is installed and whether it matches this ws
    Status,
    /// Install or update the skills; linked copies are left alone
    Install,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum State {
    Current,
    Outdated,
    Missing,
    /// A symlink, usually to a dotfiles or repo copy, managed outside ws.
    Linked,
    Installed,
    Updated,
}

#[derive(Debug, Serialize)]
struct Entry {
    skill: &'static str,
    path: PathBuf,
    state: State,
}

/// `~/.agents/skills` serves Codex and OpenCode; Claude Code reads only its own folder.
fn roots(home: &Path, claude: bool) -> Vec<PathBuf> {
    let mut roots = vec![home.join(".agents/skills")];
    if claude {
        roots.push(home.join(".claude/skills"));
    }
    roots
}

fn is_link(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|meta| meta.file_type().is_symlink())
}

fn state(path: &Path, content: &str) -> State {
    if path.parent().is_some_and(is_link) || is_link(path) {
        return State::Linked;
    }
    match fs::read_to_string(path) {
        Ok(existing) if existing == content => State::Current,
        Ok(_) => State::Outdated,
        Err(_) => State::Missing,
    }
}

fn entries(roots: &[PathBuf]) -> Vec<(Entry, &'static str)> {
    roots
        .iter()
        .flat_map(|root| SKILLS.iter().map(move |&(skill, content)| (root.join(skill).join("SKILL.md"), skill, content)))
        .map(|(path, skill, content)| (Entry { skill, state: state(&path, content), path }, content))
        .collect()
}

fn install(roots: &[PathBuf]) -> Result<Vec<Entry>> {
    entries(roots)
        .into_iter()
        .map(|(mut entry, content)| {
            let next = match entry.state {
                State::Missing => State::Installed,
                State::Outdated => State::Updated,
                _ => return Ok(entry),
            };
            let folder = entry.path.parent().context("Skill path has no folder")?;
            fs::create_dir_all(folder).with_context(|| format!("Could not create {}", folder.display()))?;
            write_atomic(&entry.path, content)?;
            entry.state = next;
            Ok(entry)
        })
        .collect()
}

pub fn run(action: SkillsAction) -> Result<i32> {
    let roots = roots(&home(), agents::detect().contains(&Agent::Claude));
    let skills = match action {
        SkillsAction::Status => entries(&roots).into_iter().map(|(entry, _)| entry).collect(),
        SkillsAction::Install => install(&roots)?,
    };
    println!("{}", serde_json::json!({ "environment": "local", "skills": skills }));
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn states(roots: &[PathBuf]) -> Vec<State> {
        entries(roots).into_iter().map(|(entry, _)| entry.state).collect()
    }

    #[test]
    fn install_writes_missing_skills_updates_stale_ones_and_keeps_links() {
        let home = std::env::temp_dir().join(format!("ws-skills-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let roots = roots(&home, true);
        assert_eq!(roots.len(), 2);
        assert!(states(&roots).iter().all(|state| *state == State::Missing));

        let installed = install(&roots).unwrap_or_default();
        assert_eq!(installed.len(), SKILLS.len() * 2);
        assert!(installed.iter().all(|entry| entry.state == State::Installed));
        assert!(states(&roots).iter().all(|state| *state == State::Current));

        let stale = roots[0].join("window-side/SKILL.md");
        assert!(fs::write(&stale, "old").is_ok());
        let linked = roots[1].join("windows-app-testing");
        assert!(fs::remove_dir_all(&linked).is_ok());
        assert!(std::os::unix::fs::symlink(&home, &linked).is_ok());

        let again = install(&roots).unwrap_or_default();
        let find = |path: &Path| again.iter().find(|entry| entry.path == path).map(|entry| entry.state);
        assert_eq!(find(&stale), Some(State::Updated));
        assert_eq!(find(&linked.join("SKILL.md")), Some(State::Linked));
        assert_eq!(fs::read_to_string(&stale).ok().as_deref(), Some(SKILLS[0].1));
        assert!(!home.join("SKILL.md").exists());
        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn every_bundled_skill_names_its_own_folder() {
        for (skill, content) in SKILLS {
            assert!(content.starts_with(&format!("---\nname: {skill}\ndescription: ")), "{skill}");
            assert!(skill.split('-').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())));
        }
        assert!(roots(Path::new("/h"), false) == [PathBuf::from("/h/.agents/skills")]);
    }
}
