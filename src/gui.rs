mod actions;
mod assets;
mod configuration;
mod first_boot;
mod fixture;
mod instance;
mod model;
mod overview;
mod review;
mod settings;
mod steps;
mod theme;
mod tray;
mod views;
mod welcome;
mod widgets;
mod wizard;

use crate::notify::{self, Urgency};
use crate::vm::virsh;
use crate::{agents::{self, Agent}, cli, config::Config, setup};
use gpui_kit::component::{
    ActiveTheme, Theme, TitleBar,
    input::InputState,
    slider::{SliderEvent, SliderState},
};
use gpui_kit::*;
use model::{Detection, History, Metrics};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
};
use std::time::Duration;
use theme::Appearance;
use tray::{Command, TrayHandle};

/// Synthetic hardware for the isolated GUI tests; debug builds only.
fn fixture_mode() -> Option<String> {
    if cfg!(debug_assertions) { std::env::var("WS_GUI_FIXTURE").ok() } else { None }
}

#[derive(Clone, Copy)]
enum Resource {
    Memory,
    Cpus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Overview,
    Settings,
}

/// Destructive actions wait for a second click within a few seconds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Armed {
    ForceStop,
    BootNormal,
    BootGpu,
}

struct WindowSide {
    config: Option<Config>,
    inventory: Option<setup::Inventory>,
    choices: Option<setup::Choices>,
    plan: Option<setup::Plan>,
    metrics: Metrics,
    history: History,
    samples: usize,
    first_boot: Option<crate::first_boot::Status>,
    page: Page,
    welcome: bool,
    armed: Option<Armed>,
    credential: Option<bool>,
    password: Entity<InputState>,
    agents: Vec<Agent>,
    copied: bool,
    setup_open: bool,
    show_files: bool,
    step: usize,
    detecting: bool,
    initialized: bool,
    planning: bool,
    applying: bool,
    refreshing: bool,
    action: Option<&'static str>,
    error: Option<String>,
    metrics_error: Option<String>,
    appearance: Appearance,
    memory: Entity<SliderState>,
    cpus: Entity<SliderState>,
    subscriptions: Vec<Subscription>,
    focus: FocusHandle,
    fixture: bool,
}

actions!(window_side, [Refresh, NextStep, PreviousStep, Quit]);

impl WindowSide {
    fn new(window: &mut Window, appearance: Appearance, cx: &mut Context<Self>) -> Self {
        let fixture_mode = fixture_mode();
        let fixture = fixture_mode.is_some();
        let mut view = Self {
            config: None,
            inventory: None,
            choices: None,
            plan: None,
            metrics: Metrics::default(),
            history: History::default(),
            samples: 0,
            first_boot: None,
            page: Page::Overview,
            welcome: false,
            armed: None,
            credential: None,
            password: cx.new(|cx| InputState::new(window, cx).masked(true).placeholder("Windows password or PIN")),
            agents: if fixture { vec![Agent::Claude, Agent::Codex] } else { agents::detect() },
            copied: false,
            setup_open: true,
            show_files: false,
            step: 0,
            detecting: true,
            initialized: false,
            planning: false,
            applying: false,
            refreshing: false,
            action: None,
            error: None,
            metrics_error: None,
            appearance,
            memory: cx.new(|_| SliderState::new()),
            cpus: cx.new(|_| SliderState::new()),
            subscriptions: Vec::new(),
            focus: cx.focus_handle(),
            fixture,
        };
        view.focus.focus(window, cx);
        if fixture {
            view.accept_detection(fixture::detection(), cx);
            if let Some(page) = fixture_mode.as_deref().and_then(|mode| mode.strip_prefix("status")) {
                view.setup_open = false;
                view.welcome = false;
                view.accept_metrics(Metrics::fixture());
                view.history = History::fixture();
                view.config = Some(fixture::config());
                view.credential = Some(false);
                if page == "-settings" {
                    view.page = Page::Settings;
                }
            } else if let Some(mode) = fixture_mode.as_deref().and_then(|mode| mode.strip_prefix("first-boot-")) {
                view.setup_open = false;
                view.welcome = false;
                view.first_boot = Some(fixture::first_boot(mode));
            }
        } else {
            view.detect(cx);
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(Duration::from_secs(5)).await;
                    if this
                        .update(cx, |view, cx| {
                            if !view.setup_open {
                                view.refresh(cx);
                            }
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .detach();
        }
        view
    }

    fn accept_detection(&mut self, detection: Detection, cx: &mut Context<Self>) {
        self.detecting = false;
        self.config = detection.config;
        if !self.initialized {
            self.setup_open = self.config.is_none();
            self.welcome = self.setup_open;
        }
        self.initialized = true;
        self.choices = detection.inventory.default_choices(self.config.as_ref());
        if let Some(choices) = &self.choices {
            let memory_limit = (detection.inventory.memory_limit_mib() / 1024) as f32;
            let cpu_limit = detection.inventory.vcpu_limit() as f32;
            self.memory = cx.new(|_| {
                SliderState::new()
                    .min(2.)
                    .max(memory_limit)
                    .step(1.)
                    .default_value((choices.memory_mib / 1024) as f32)
            });
            self.cpus = cx.new(|_| {
                SliderState::new()
                    .min(2.)
                    .max(cpu_limit)
                    .step(1.)
                    .default_value(choices.vcpus as f32)
            });
            self.subscriptions = [&self.memory, &self.cpus]
                .into_iter()
                .map(|slider| {
                    cx.subscribe(slider, |this, _, event: &SliderEvent, cx| {
                        if let SliderEvent::Change(_) = event {
                            this.plan = None;
                            cx.notify();
                        }
                    })
                })
                .collect();
        }
        self.inventory = Some(detection.inventory);
        self.check_credential(cx);
        if !self.setup_open {
            self.refresh(cx);
        }
        cx.notify();
    }

    fn detect(&mut self, cx: &mut Context<Self>) {
        if self.fixture {
            return;
        }
        self.detecting = true;
        self.error = None;
        let task = cx.background_spawn(async { model::detect() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(detection) => this.accept_detection(detection, cx),
                    Err(error) => {
                        this.detecting = false;
                        this.error = Some(format!("{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.fixture || self.refreshing || self.action.is_some() || self.config.is_none() {
            return;
        }
        self.refreshing = true;
        let config = self.config.clone();
        let task = cx.background_spawn(async move {
            let boot = config.filter(|config| config.needs_first_boot())
                .map(|config| crate::first_boot::status(&config)).transpose();
            (Metrics::read(), boot)
        });
        cx.spawn(async move |this, cx| {
            let (result, boot) = task.await;
            let _ = this.update(cx, |this, cx| {
                this.refreshing = false;
                match boot {
                    Ok(status) => this.first_boot = status,
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                match result {
                    Ok(metrics) => {
                        this.accept_metrics(metrics);
                        this.metrics_error = None;
                    }
                    Err(error) => {
                        this.accept_metrics(Metrics::default());
                        this.metrics_error = Some(format!("{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn accept_metrics(&mut self, metrics: Metrics) {
        self.history.record(&metrics);
        self.samples += 1;
        self.metrics = metrics;
    }

    fn next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.welcome {
            self.welcome = false;
            cx.notify();
            return;
        }
        if !self.setup_open
            || self.detecting
            || self.planning
            || self.applying
            || self.choices.is_none()
        {
            return;
        }
        self.error = None;
        self.focus.focus(window, cx);
        if self.step < 2 {
            self.step += 1;
        } else if self.step == 2 {
            self.prepare_plan(cx);
        } else {
            self.apply_plan(window, cx);
        }
        cx.notify();
    }

    fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.applying || self.planning {
            return;
        }
        self.focus.focus(window, cx);
        if self.step == 0 && self.config.is_none() {
            self.welcome = true;
        }
        self.step = self.step.saturating_sub(1);
        self.plan = None;
        self.error = None;
        cx.notify();
    }
}

impl Render for WindowSide {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("WindowSide")
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &Quit, _, cx| cx.quit()))
            .on_action(cx.listener(|this, _: &Refresh, _, cx| this.refresh(cx)))
            .on_action(cx.listener(|this, _: &NextStep, window, cx| this.next(window, cx)))
            .on_action(cx.listener(|this, _: &PreviousStep, window, cx| this.back(window, cx)))
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.header(cx))
            .child(self.page_view(cx))
    }
}

fn open_main(appearance: Appearance, cx: &mut App) -> Result<()> {
    // Window geometry is a platform boundary; content uses the rem scale.
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(920.), px(680.)), cx))),
        window_min_size: Some(size(px(720.), px(540.))),
        app_id: Some("window-side".into()),
        titlebar: Some(TitlebarOptions { title: Some("window-side".into()), ..TitleBar::title_bar_options() }),
        ..TitleBar::window_options()
    };
    gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| WindowSide::new(window, appearance, cx)))?;
    cx.activate(true);
    Ok(())
}

/// Brings the window forward, reopening it when it was closed to the tray.
fn show_main(cx: &mut App) {
    if let Some(window) = cx.windows().first() {
        let _ = window.update(cx, |_, window, _| window.activate_window());
    } else if let Err(error) = open_main(Appearance::load(), cx) {
        notify::send(Urgency::Normal, &format!("Could not open window-side: {error:#}"));
    }
}

/// Handles tray and second-launch requests and keeps the tray's Windows state current.
fn serve(commands: Receiver<Command>, tray: Option<TrayHandle>, cx: &mut App) {
    cx.spawn(async move |cx| {
        let mut tick = 0_u32;
        loop {
            while let Ok(command) = commands.try_recv() {
                match command {
                    Command::Window => cx.update(show_main),
                    Command::Quit => cx.update(|cx| cx.quit()),
                    Command::Run(label, action) => cx
                        .background_executor()
                        .spawn(async move {
                            match cli::run(action) {
                                Ok(0) => {}
                                Ok(code) => notify::send(Urgency::Normal, &format!("{label} exited with code {code}")),
                                Err(error) => notify::send(Urgency::Normal, &format!("{label} failed: {error:#}")),
                            }
                        })
                        .detach(),
                }
            }
            if let Some(tray) = tray.as_ref().filter(|_| tick.is_multiple_of(25)) {
                let (status, configured) = cx
                    .background_executor()
                    .spawn(async {
                        Config::load().map_or_else(|_| (String::new(), false), |config| (virsh::state(&config.vm.name), true))
                    })
                    .await;
                tray.update(status, configured);
            }
            tick = tick.wrapping_add(1);
            cx.background_executor().timer(Duration::from_millis(200)).await;
        }
    })
    .detach();
}

pub fn run() -> bool {
    let fixture = fixture_mode().is_some();
    let (sender, commands) = mpsc::channel();
    let instance = if fixture {
        None
    } else {
        match instance::claim(sender.clone()) {
            Ok(Some(connection)) => Some(connection),
            Ok(None) => return true,
            Err(error) => {
                eprintln!("window-side: running without single-instance support: {error:#}");
                None
            }
        }
    };
    let failed = Arc::new(AtomicBool::new(false));
    let window_failed = failed.clone();
    // The window-closed handler decides when to quit, so the app can stay in the tray.
    application().with_assets(assets::AppAssets).with_quit_mode(QuitMode::Explicit).run(move |cx| {
        gpui_kit::init(cx);
        let mut appearance = Appearance::load();
        if let Err(error) = appearance.apply(cx) {
            eprintln!("window-side theme: {error:#}; using teal");
            appearance = Appearance::Teal;
            let _ = appearance.apply(cx);
        }
        Theme::update(cx, |theme| theme.font_size = px(15.));
        cx.bind_keys([
            KeyBinding::new("ctrl-r", Refresh, Some("WindowSide")),
            KeyBinding::new("ctrl-q", Quit, Some("WindowSide")),
            KeyBinding::new("alt-right", NextStep, Some("WindowSide")),
            KeyBinding::new("alt-left", PreviousStep, Some("WindowSide")),
        ]);
        if let Err(error) = open_main(appearance, cx) {
            eprintln!("window-side window: {error:#}");
            window_failed.store(true, Ordering::Relaxed);
            cx.quit();
            return;
        }
        let tray = if fixture { None } else { tray::start(sender) };
        let in_tray = tray.as_ref().map(|tray| tray.online.clone());
        // Closing the window keeps the app in the tray only while a bar shows it.
        cx.on_window_closed(move |cx, _| {
            // Holding the connection keeps the session name for the app's lifetime.
            let _ = &instance;
            if cx.windows().is_empty() && !in_tray.as_ref().is_some_and(|online| online.load(Ordering::Relaxed)) {
                cx.quit();
            }
        })
        .detach();
        serve(commands, tray, cx);
    });
    !failed.load(Ordering::Relaxed)
}
