use super::{
    WindowSide,
    widgets::{card, chip, heading, pill, reveal},
};
use crate::cli::{Action, FirstBootAction, SetupStep};
use crate::first_boot::Stage;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Sizable,
    alert::Alert,
    button::{Button, ButtonVariants},
    spinner::Spinner,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

const MILESTONES: [(&str, IconName); 4] = [
    ("Start Windows", IconName::Play),
    ("Install drivers", IconName::HardDriveDownload),
    ("Sign in", IconName::LogIn),
    ("Finish setup", IconName::RotateCcw),
];

impl WindowSide {
    fn milestone(&self, index: usize, current: usize, detail: &str, cx: &App) -> Div {
        let theme = cx.theme();
        let (label, icon) = MILESTONES[index];
        let done = index < current;
        let active = index == current;
        let color = if done || active { theme.primary } else { theme.muted_foreground };
        let node = div()
            .flex()
            .items_center()
            .justify_center()
            .size_9()
            .rounded_full()
            .border_1()
            .border_color(if done || active { theme.primary } else { theme.border })
            .bg(if done { theme.primary } else if active { theme.primary.opacity(0.14) } else { theme.background })
            .child(
                Icon::new(if done { IconName::Check } else { icon })
                    .text_color(if done { theme.primary_foreground } else { color })
                    .with_size(px(16.)),
            );
        let rail = div()
            .flex()
            .flex_col()
            .items_center()
            .child(node)
            .when(index + 1 < MILESTONES.len(), |view| {
                view.child(div().w_px().h_8().bg(if done { theme.primary } else { theme.border }))
            });
        div()
            .flex()
            .gap_4()
            .child(rail)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pt_2()
                    .child(
                        div()
                            .font_weight(if active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(if done || active { theme.foreground } else { theme.muted_foreground })
                            .child(label),
                    )
                    .when(active && !detail.is_empty(), |view| {
                        view.child(div().text_sm().text_color(theme.muted_foreground).child(detail.to_owned()))
                    }),
            )
    }

    pub(super) fn first_boot_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let stage = self.first_boot.as_ref().map(|status| status.stage);
        let busy = self.action.is_some();
        let (state, detail, label, command, current) = match stage {
            Some(Stage::Start) => ("Not started", "Start Windows, then install the guest tools from its driver CD.", "Start setup…", FirstBootAction::Start, 0),
            Some(Stage::Drivers) => ("Drivers needed", "In Windows, run virtio-win-guest-tools.exe from the driver CD. Then return here.", "Open Windows…", FirstBootAction::Start, 1),
            Some(Stage::SignIn) => ("Sign-in needed", "Sign in to Windows, then return here to finish setup.", "Open Windows…", FirstBootAction::Start, 2),
            Some(Stage::Ready) => ("Ready to finish", "Restarting closes Windows apps and finishes driver setup.", "Restart Windows", FirstBootAction::Finish, 3),
            Some(Stage::GpuBoot) => ("GPU boot needed", "", "Check boot profile", FirstBootAction::Status, 3),
            _ => ("Checking", "", "Start setup…", FirstBootAction::Start, 0),
        };
        let theme = cx.theme();
        let status_pill = match self.action {
            Some(action) => pill(theme.primary)
                .child(Spinner::new().color(theme.primary).with_size(px(12.)))
                .child(action),
            None => chip(None, state, if current == 3 { theme.primary } else { theme.warning }),
        };
        let driver = |name: &'static str, ready: Option<bool>| {
            if ready == Some(true) {
                chip(Some(IconName::CircleCheck), name, theme.primary)
            } else {
                chip(Some(IconName::CircleDashed), name, theme.muted_foreground)
            }
        };
        let timeline = card(cx)
            .gap_0()
            .p_6()
            .children((0..MILESTONES.len()).map(|index| self.milestone(index, current, detail, cx)));
        let content = div()
            .flex()
            .flex_col()
            .gap_5()
            .p_6()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .child(heading("First boot", "Install drivers once, then Windows runs on VirtIO.", cx))
                    .child(status_pill),
            )
            .when(stage == Some(Stage::GpuBoot), |view| {
                view.child(
                    Alert::warning(
                        "gpu-boot",
                        "Shut down Windows, then reboot Linux into the GPU profile from the window-side panel.",
                    )
                    .title("GPU boot needed"),
                )
            })
            .child(timeline)
            .when_some(self.first_boot.as_ref(), |view, status| {
                view.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().text_color(theme.muted_foreground).mr_1().child("Drivers"))
                        .child(driver("Storage", status.storage_ready))
                        .child(driver("Network", status.network_ready)),
                )
            })
            .child(self.error_view());
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .id("first-boot-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(reveal(SharedString::from(format!("first-boot-{current}")), content)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_6()
                    .py_4()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("first-boot-primary")
                            .primary()
                            .icon(Icon::new(if current == 3 { IconName::RotateCcw } else { IconName::MonitorPlay }))
                            .label(label)
                            .disabled(busy || stage.is_none())
                            .loading(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.run_action(
                                    if matches!(command, FirstBootAction::Finish) { "Finishing setup" } else { "Opening setup" },
                                    Action::Setup { step: SetupStep::FirstBoot { action: command } },
                                    cx,
                                );
                            })),
                    )
                    .when(stage != Some(Stage::GpuBoot), |view| {
                        view.child(
                            Button::new("first-boot-check")
                                .outline()
                                .icon(Icon::new(IconName::RefreshCw))
                                .label("Check again")
                                .disabled(busy || self.refreshing)
                                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                        )
                    }),
            )
            .into_any_element()
    }
}
