//! Settings: the VM, sign-in, display, updates, appearance and optional add-ons.
use super::{
    WindowSide,
    theme::Appearance,
    widgets::{aurora, card, chip, field, icon_tile, pill_button, reveal},
};
use crate::cli::{Action, SetupStep, SignInAction};
use crate::config::DiskMode;
use crate::guest::commands::UpdateAction;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Sizable,
    button::{Button, ButtonVariants},
    input::Input,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

const RESOLUTIONS: [(u32, u32); 3] = [(1920, 1080), (2560, 1440), (3840, 2160)];

fn section(icon: IconName, color: Hsla, title: &str, detail: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(icon_tile(icon, color, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(div().font_weight(FontWeight::MEDIUM).child(title.to_owned()))
                .child(div().text_xs().text_color(cx.theme().muted_foreground).truncate().child(detail.into())),
        )
}

impl WindowSide {
    fn omarchy(&self) -> bool {
        self.inventory.as_ref().is_some_and(|inventory| inventory.system.omarchy)
    }

    fn vm_card(&self, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let Some(config) = &self.config else {
            return div();
        };
        card(cx)
            .child(
                section(IconName::Monitor, theme.chart_1, "Windows VM", config.vm.name.clone(), cx).child(
                    pill_button("change-setup")
                        .outline()
                        .small()
                        .label("Change setup…")
                        .disabled(self.action.is_some() || self.metrics.running())
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.focus.focus(window, cx);
                            this.setup_open = true;
                            this.step = 0;
                            this.error = None;
                            cx.notify();
                        })),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(field("Memory", format!("{} GiB", config.vm.memory_mib / 1024), cx))
                    .child(field("CPU threads", config.vm.vcpus.to_string(), cx))
                    .child(field(
                        "Graphics",
                        config.gpu.as_ref().map_or_else(|| "Virtual display".to_owned(), |gpu| format!("Passthrough · {}", gpu.address)),
                        cx,
                    ))
                    .child(field(
                        "Disk",
                        match config.disk.mode {
                            DiskMode::Whole => "Whole drive",
                            DiskMode::Partition => "Windows partition only",
                        },
                        cx,
                    )),
            )
    }

    fn sign_in_card(&self, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let busy = self.action.is_some();
        let stored = self.credential == Some(true);
        let mut body = card(cx).child(
            section(IconName::LockKeyhole, theme.chart_2, "Automatic sign-in", "Kept in your keyring, typed at the Windows sign-in screen", cx)
                .when_some(self.credential, |view, stored| {
                    view.child(if stored {
                        chip(Some(IconName::CircleCheck), "Saved", theme.primary)
                    } else {
                        chip(None, "Not saved", theme.muted_foreground)
                    })
                }),
        );
        body = if stored {
            body.child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        pill_button("sign-in-now")
                            .outline()
                            .icon(IconName::LogIn)
                            .label("Sign in now")
                            .disabled(busy || !self.metrics.running())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.run_action("Signing in", Action::SignIn { action: SignInAction::Run, wait: 40 }, cx);
                            })),
                    )
                    .child(
                        pill_button("forget-sign-in")
                            .ghost()
                            .label("Forget")
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.forget_credential(cx))),
                    ),
            )
        } else {
            body.child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&self.password).mask_toggle()))
                    .child(
                        pill_button("save-sign-in")
                            .primary()
                            .label("Save")
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, window, cx| this.save_credential(window, cx))),
                    ),
            )
        };
        body
    }

    fn display_card(&self, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let ready = self.metrics.running() && self.action.is_none();
        let has_gpu = self.config.as_ref().is_some_and(|config| config.gpu.is_some());
        card(cx)
            .child(section(
                IconName::MonitorPlay,
                theme.chart_4,
                "Display",
                if self.metrics.running() { "Applies to running Windows" } else { "Start Windows to change" },
                cx,
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().text_color(theme.muted_foreground).mr_2().child("Resolution"))
                    .children(RESOLUTIONS.into_iter().map(|(width, height)| {
                        pill_button(SharedString::from(format!("resolution-{width}")))
                            .outline()
                            .small()
                            .label(format!("{width} × {height}"))
                            .disabled(!ready)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.run_action("Setting resolution", Action::Resolution { width, height, json: false }, cx);
                            }))
                    })),
            )
            .when(has_gpu, |view| {
                view.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().text_sm().text_color(theme.muted_foreground).child("Looking Glass display driver"))
                        .child(
                            pill_button("looking-glass-setup")
                                .outline()
                                .small()
                                .label("Install in Windows")
                                .disabled(!ready)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.run_action("Installing display", Action::LookingGlassSetup, cx);
                                })),
                        ),
                )
            })
    }

    fn updates_card(&self, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let ready = self.metrics.running() && self.action.is_none();
        let pending = self.metrics.pending_updates();
        let detail = match pending {
            Some(count) => format!("{count} update{} ready to install", if count == 1 { "" } else { "s" }),
            None => "Windows is up to date".to_owned(),
        };
        card(cx).child(
            section(IconName::HardDriveDownload, theme.chart_5, "Windows Update", detail, cx)
                .child(
                    pill_button("updates-check")
                        .ghost()
                        .small()
                        .icon(IconName::RefreshCw)
                        .label("Check")
                        .disabled(!ready)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.run_action("Checking updates", Action::Updates { action: UpdateAction::Check }, cx);
                        })),
                )
                .when(pending.is_some(), |view| {
                    view.child(
                        pill_button("updates-install")
                            .primary()
                            .small()
                            .label("Install")
                            .disabled(!ready)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.run_action("Installing updates", Action::Updates { action: UpdateAction::Install }, cx);
                            })),
                    )
                }),
        )
    }

    fn appearance_card(&self, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let current = self.appearance;
        let option = |id: &'static str, label: &'static str, appearance: Appearance| {
            let selected = current == appearance;
            Button::new(id)
                .small()
                .label(label)
                .when(selected, |button| button.primary())
                .when(!selected, |button| button.ghost())
                .on_click(cx.listener(move |this, _, _, cx| this.set_appearance(appearance, cx)))
        };
        card(cx).child(
            section(IconName::Palette, theme.chart_3, "Appearance", "Colours for this window", cx).child(
                div()
                    .flex()
                    .gap_1()
                    .p_1()
                    .rounded_full()
                    .bg(theme.muted)
                    .child(option("appearance-teal", "Pastel", Appearance::Teal))
                    .when(self.omarchy(), |view| view.child(option("appearance-omarchy", "Omarchy", Appearance::Omarchy))),
            ),
        )
    }

    fn omarchy_card(&self, cx: &mut Context<Self>) -> Div {
        card(cx).child(
            section(IconName::Layers, cx.theme().info, "Omarchy panel", "Status and controls in the Omarchy bar", cx).child(
                pill_button("install-plugin")
                    .outline()
                    .small()
                    .label("Install or repair")
                    .disabled(self.action.is_some())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.run_action("Installing panel", Action::Setup { step: SetupStep::Plugin }, cx);
                    })),
            ),
        )
    }

    pub(super) fn settings_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let content = div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .child(self.error_view())
            .child(self.vm_card(cx))
            .child(self.sign_in_card(cx))
            .child(self.display_card(cx))
            .child(self.updates_card(cx))
            .child(self.appearance_card(cx))
            .when(self.omarchy(), |view| view.child(self.omarchy_card(cx)))
            .child(
                div()
                    .text_xs()
                    .text_center()
                    .text_color(cx.theme().muted_foreground)
                    .child(concat!("window-side ", env!("CARGO_PKG_VERSION"))),
            );
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .child(aurora())
            .child(
                div()
                    .id("settings-scroll")
                    .absolute()
                    .size_full()
                    .overflow_y_scroll()
                    .child(reveal("settings", content)),
            )
            .into_any_element()
    }
}
