//! The running dashboard: lifecycle actions, live waves and the boot profile.
use super::{
    Armed, WindowSide,
    widgets::{aurora, card, chip, choice_card, icon_tile, live_dot, pill, pill_button, reveal, wave, windows_mark},
};
use crate::cli::{Action, BootChoice};
use crate::guest::metrics::byte_label;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Sizable,
    alert::Alert,
    button::ButtonVariants,
    spinner::Spinner,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::VecDeque;

fn rate(series: &VecDeque<f64>) -> String {
    series.back().map_or_else(|| "—".into(), |bytes| format!("{}/s", byte_label(*bytes)))
}

fn percent(value: Option<f64>) -> String {
    value.map_or_else(|| "—".into(), |value| format!("{value:.0}%"))
}

impl WindowSide {
    fn state_chip(&self, cx: &App) -> Div {
        let theme = cx.theme();
        if let Some(action) = self.action {
            return pill(theme.primary)
                .child(Spinner::new().color(theme.primary).with_size(px(12.)))
                .child(action);
        }
        let state = self.metrics.state();
        let color = match state {
            "Running" => theme.primary,
            "Paused" | "Stopping" => theme.warning,
            _ => theme.muted_foreground,
        };
        pill(color)
            .child(live_dot(self.samples, color, self.metrics.running()))
            .child(state.to_owned())
    }

    fn hero(&self, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let running = self.metrics.running();
        let busy = self.action.is_some();
        let known = matches!(self.metrics.state(), "Running" | "Stopped");
        let force = self.armed == Some(Armed::ForceStop);
        let subtitle = if running {
            format!("Up {} · {} processes · {}", self.metrics.uptime(), self.metrics.processes(), self.metrics.profile())
        } else {
            self.metrics.profile().to_owned()
        };
        card(cx)
            .gap_5()
            .p_6()
            .bg(linear_gradient(
                135.,
                linear_color_stop(theme.primary.opacity(if running { 0.16 } else { 0.07 }), 0.),
                linear_color_stop(theme.group_box.opacity(0.78), 0.65),
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .size_16()
                            .rounded(theme.radius_lg)
                            .bg(theme.background.opacity(0.6))
                            .border_1()
                            .border_color(theme.primary.opacity(0.25))
                            .child(windows_mark(px(14.), theme.primary)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(div().text_2xl().font_weight(FontWeight::SEMIBOLD).child("Windows"))
                            .child(div().text_sm().text_color(theme.muted_foreground).truncate().child(subtitle)),
                    )
                    .child(self.state_chip(cx)),
            )
            .when_some(self.metrics.storage_error(), |view, error| {
                view.child(Alert::error("storage", SharedString::from(error.to_owned())).title("Windows disk unavailable"))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        pill_button("open-windows")
                            .primary()
                            .icon(Icon::new(if running { IconName::MonitorPlay } else { IconName::Play }))
                            .label(if running { "Open" } else { "Start" })
                            .disabled(busy || !known || self.metrics.storage_error().is_some() && !running)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.run_action(
                                    if running { "Opening" } else { "Starting" },
                                    if running { Action::Console } else { Action::Launch },
                                    cx,
                                );
                            })),
                    )
                    .when(running, |view| {
                        view.child(
                            pill_button("stop-windows")
                                .outline()
                                .icon(Icon::new(IconName::Power))
                                .label("Shut down")
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| this.run_action("Stopping", Action::Stop, cx))),
                        )
                        .child(
                            pill_button("force-stop")
                                .when(force, |button| button.danger())
                                .when(!force, |button| button.ghost())
                                .label(if force { "Confirm force stop" } else { "Force stop" })
                                .disabled(busy)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if force {
                                        this.run_action("Forcing off", Action::ForceStop, cx);
                                    } else {
                                        this.arm(Armed::ForceStop, cx);
                                    }
                                })),
                        )
                    })
                    .child(div().flex_1())
                    .when_some(self.metrics.pending_updates(), |view, count| {
                        view.child(chip(
                            Some(IconName::HardDriveDownload),
                            format!("{count} update{}", if count == 1 { "" } else { "s" }),
                            theme.info,
                        ))
                    }),
            )
    }

    fn wave_card(&self, icon: IconName, color: Hsla, label: &str, value: String, chart: Div, cx: &App) -> Div {
        card(cx)
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(icon_tile(icon, color, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child(label.to_owned()))
                            .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child(value)),
                    ),
            )
            .child(chart)
    }

    fn waves(&self, cx: &App) -> Div {
        let theme = cx.theme();
        let history = &self.history;
        let (cpu, memory, gpu, down, up) = (theme.chart_1, theme.chart_2, theme.chart_3, theme.chart_4, theme.chart_5);
        let network = div()
            .flex()
            .gap_3()
            .child(format!("↓ {}", rate(&history.down)))
            .child(div().text_color(theme.muted_foreground).child(format!("↑ {}", rate(&history.up))));
        div()
            .grid()
            .grid_cols(2)
            .gap_4()
            .child(self.wave_card(IconName::Cpu, cpu, "Processor", self.metrics.cpu(), wave("cpu-wave", &[(&history.cpu, cpu)], Some(100.), cx), cx))
            .child(self.wave_card(
                IconName::MemoryStick,
                memory,
                "Memory",
                format!("{} · {}", percent(self.metrics.memory_percent()), self.metrics.memory()),
                wave("memory-wave", &[(&history.memory, memory)], Some(100.), cx),
                cx,
            ))
            .child(self.wave_card(IconName::Gpu, gpu, "Graphics", percent(self.metrics.gpu_percent()), wave("gpu-wave", &[(&history.gpu, gpu)], Some(100.), cx), cx))
            .child(
                card(cx)
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(icon_tile(IconName::Network, down, cx))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(div().text_xs().text_color(theme.muted_foreground).child("Network"))
                                    .child(network.text_lg().font_weight(FontWeight::SEMIBOLD)),
                            ),
                    )
                    .child(wave("network-wave", &[(&history.down, down), (&history.up, up)], None, cx)),
            )
    }

    fn boot_option(&self, choice: BootChoice, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = cx.theme();
        let gpu = matches!(choice, BootChoice::Gpu);
        let (title, detail, icon, armed) = if gpu {
            ("GPU passthrough", "Windows gets the GPU", IconName::Gpu, Armed::BootGpu)
        } else {
            ("Normal", "Linux keeps the GPU", IconName::Monitor, Armed::BootNormal)
        };
        let current = self.metrics.gpu_profile() == gpu;
        let blocked = self.metrics.running() || self.action.is_some();
        let confirming = self.armed == Some(armed);
        choice_card(if gpu { "boot-gpu" } else { "boot-normal" }, current || confirming, !current && !blocked, cx)
            .flex_1()
            .p_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(icon_tile(icon, if current { theme.primary } else { theme.muted_foreground }, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .child(div().font_weight(FontWeight::MEDIUM).child(title))
                            .child(div().text_xs().text_color(theme.muted_foreground).child(detail)),
                    )
                    .when(current, |view| view.child(chip(Some(IconName::Check), "Current", theme.primary)))
                    .when(confirming, |view| {
                        view.child(
                            pill_button(if gpu { "reboot-gpu" } else { "reboot-normal" })
                                .danger()
                                .small()
                                .icon(Icon::new(IconName::RotateCcw))
                                .label("Reboot now")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.run_action("Rebooting", Action::Reboot { profile: choice }, cx);
                                })),
                        )
                    }),
            )
            .when(!current && !blocked && !confirming, |view| {
                view.on_click(cx.listener(move |this, _, _, cx| this.arm(armed, cx)))
            })
    }

    fn boot_card(&self, cx: &mut Context<Self>) -> Div {
        let note = if self.metrics.running() { "Shut down Windows to switch" } else { "Switching reboots Linux" };
        card(cx)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Boot profile"))
                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child(note)),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.boot_option(BootChoice::Normal, cx))
                    .child(self.boot_option(BootChoice::Gpu, cx)),
            )
    }

    pub(super) fn overview_view(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.first_boot.is_some() || self.config.as_ref().is_some_and(|config| config.needs_first_boot()) {
            return self.first_boot_view(cx);
        }
        let has_gpu = self.config.as_ref().is_some_and(|config| config.gpu.is_some());
        let content = div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .child(self.hero(cx))
            .child(self.waves(cx))
            .when(has_gpu, |view| view.child(self.boot_card(cx)))
            .child(self.error_view());
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .child(aurora())
            .child(
                div()
                    .id("overview-scroll")
                    .absolute()
                    .size_full()
                    .overflow_y_scroll()
                    .child(reveal("overview", content)),
            )
            .into_any_element()
    }
}
