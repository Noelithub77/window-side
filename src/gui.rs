mod configuration;
mod fixture;
mod model;
mod theme;
mod views;
mod wizard;

use crate::{cli, config::Config, setup};
use gpui_kit::component::{
    ActiveTheme, Theme, TitleBar,
    slider::{SliderEvent, SliderState},
};
use gpui_kit::*;
use model::{Detection, Metrics};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use theme::Appearance;

#[derive(Clone, Copy)]
enum Resource {
    Memory,
    Cpus,
}

struct WindowSide {
    config: Option<Config>,
    inventory: Option<setup::Inventory>,
    choices: Option<setup::Choices>,
    plan: Option<setup::Plan>,
    metrics: Metrics,
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
        let fixture_mode = if cfg!(debug_assertions) {
            std::env::var("WS_GUI_FIXTURE").ok()
        } else {
            None
        };
        let fixture = fixture_mode.is_some();
        let mut view = Self {
            config: None,
            inventory: None,
            choices: None,
            plan: None,
            metrics: Metrics::default(),
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
            if fixture_mode.as_deref() == Some("status") {
                view.setup_open = false;
                view.metrics = Metrics::fixture();
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
        if self.fixture || self.refreshing || self.config.is_none() {
            return;
        }
        self.refreshing = true;
        let task = cx.background_spawn(async { Metrics::read() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.refreshing = false;
                match result {
                    Ok(metrics) => {
                        this.metrics = metrics;
                        this.metrics_error = None;
                    }
                    Err(error) => {
                        this.metrics = Metrics::default();
                        this.metrics_error = Some(format!("{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn run_action(&mut self, label: &'static str, action: cli::Action, cx: &mut Context<Self>) {
        if self.action.is_some() || self.fixture {
            return;
        }
        self.action = Some(label);
        self.error = None;
        let task = cx.background_spawn(async move { cli::run(action) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.action = None;
                match result {
                    Ok(0) => {}
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

    fn set_appearance(&mut self, appearance: Appearance, cx: &mut Context<Self>) {
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

    fn next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
            .child(if self.setup_open {
                self.setup_view(cx)
            } else {
                self.status_view(cx)
            })
    }
}

pub fn run() -> bool {
    let failed = Arc::new(AtomicBool::new(false));
    let window_failed = failed.clone();
    application().with_assets(assets::Assets).run(move |cx| {
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
        // Window geometry is a platform boundary; content uses the rem scale.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(880.), px(620.)),
                cx,
            ))),
            window_min_size: Some(size(px(640.), px(500.))),
            app_id: Some("window-side".into()),
            titlebar: Some(TitlebarOptions {
                title: Some("window-side".into()),
                ..TitleBar::title_bar_options()
            }),
            ..TitleBar::window_options()
        };
        if let Err(error) = gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| WindowSide::new(window, appearance, cx))
        }) {
            eprintln!("window-side window: {error:#}");
            window_failed.store(true, Ordering::Relaxed);
            cx.quit();
        }
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
    !failed.load(Ordering::Relaxed)
}
