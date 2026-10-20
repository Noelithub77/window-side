//! Window-side actions: CLI commands off the UI thread, confirmations and settings writes.
use super::{Armed, WindowSide, fixture, model::{History, Metrics}, theme::Appearance};
use crate::{agents::{self, Agent}, cli, config::Config, guest::sign_in};
use anyhow::{Context as _, Result};
use gpui_kit::*;
use std::process::Command;
use std::time::Duration;

impl WindowSide {
    pub(super) fn run_action(&mut self, label: &'static str, action: cli::Action, cx: &mut Context<Self>) {
        if self.action.is_some() {
            return;
        }
        self.armed = None;
        if self.fixture {
            if let cli::Action::Setup { step: cli::SetupStep::FirstBoot { action } } = action {
                match action {
                    cli::FirstBootAction::Start => self.first_boot = Some(fixture::first_boot("drivers")),
                    cli::FirstBootAction::Finish => {
                        self.first_boot = None;
                        self.accept_metrics(Metrics::fixture());
                        self.history = History::fixture();
                    }
                    _ => {}
                }
            }
            cx.notify();
            return;
        }
        self.action = Some(label);
        self.error = None;
        let task = cx.background_spawn(async move {
            match action {
                // `ws reboot` replaces its process with pkexec, so it must not run inside the window.
                cli::Action::Reboot { profile } => reboot(profile),
                action => cli::run(action),
            }
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.action = None;
                match result {
                    Ok(0) => {
                        if let Ok(config) = Config::load() {
                            this.config = Some(config);
                        }
                    }
                    Ok(code) => this.error = Some(format!("{label} exited with code {code}")),
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                this.refresh(cx);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// First click arms `armed`; it disarms itself after six seconds.
    pub(super) fn arm(&mut self, armed: Armed, cx: &mut Context<Self>) {
        self.armed = Some(armed);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(6)).await;
            let _ = this.update(cx, |this, cx| {
                if this.armed == Some(armed) {
                    this.armed = None;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    pub(super) fn set_appearance(&mut self, appearance: Appearance, cx: &mut Context<Self>) {
        match appearance.apply(cx).and_then(|()| {
            if self.fixture {
                Ok(())
            } else {
                appearance.save()
            }
        }) {
            Ok(()) => {
                self.appearance = appearance;
                self.error = None;
            }
            Err(error) => self.error = Some(format!("{error:#}")),
        }
        cx.notify();
    }

    pub(super) fn open_agent(&mut self, agent: Agent, cx: &mut Context<Self>) {
        if self.fixture {
            return;
        }
        let task = cx.background_spawn(async move { agents::launch(agent) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.error = result.err().map(|error| format!("{error:#}"));
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn copy_prompt(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(agents::SETUP_PROMPT.to_owned()));
        self.copied = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(3)).await;
            let _ = this.update(cx, |this, cx| {
                this.copied = false;
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn windows_user(&self) -> Option<String> {
        self.config
            .as_ref()
            .map(|config| config.windows.user.clone())
            .filter(|user| !user.is_empty())
    }

    pub(super) fn check_credential(&mut self, cx: &mut Context<Self>) {
        if self.fixture {
            return;
        }
        let Some(user) = self.windows_user() else {
            return;
        };
        let task = cx.background_spawn(async move { sign_in::is_stored(&user) });
        cx.spawn(async move |this, cx| {
            let stored = task.await;
            let _ = this.update(cx, |this, cx| {
                this.credential = Some(stored);
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn save_credential(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let secret = self.password.read(cx).value().to_string();
        self.password.update(cx, |input, cx| input.set_value("", window, cx));
        if self.fixture {
            self.credential = Some(true);
            cx.notify();
            return;
        }
        let Some(user) = self.windows_user() else {
            return;
        };
        self.credential_task(cx.background_spawn(async move { sign_in::store(&user, &secret).map(|()| true) }), cx);
    }

    pub(super) fn forget_credential(&mut self, cx: &mut Context<Self>) {
        if self.fixture {
            self.credential = Some(false);
            cx.notify();
            return;
        }
        let Some(user) = self.windows_user() else {
            return;
        };
        self.credential_task(cx.background_spawn(async move { sign_in::forget(&user).map(|()| false) }), cx);
    }

    fn credential_task(&mut self, task: Task<Result<bool>>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(stored) => {
                        this.credential = Some(stored);
                        this.error = None;
                    }
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

fn reboot(profile: cli::BootChoice) -> Result<i32> {
    let name = match profile {
        cli::BootChoice::Normal => "normal",
        cli::BootChoice::Gpu => "gpu",
    };
    let status = Command::new(std::env::current_exe()?)
        .args(["reboot", name])
        .status()
        .context("Could not start the reboot")?;
    Ok(status.code().unwrap_or(1))
}
