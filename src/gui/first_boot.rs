use super::{WindowSide, views::{field, heading}};
use crate::cli::{Action, FirstBootAction, SetupStep};
use crate::first_boot::Stage;
use gpui_kit::component::{ActiveTheme, Disableable, button::Button};
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder;

impl WindowSide {
    pub(super) fn first_boot_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let stage = self.first_boot.as_ref().map(|status| status.stage);
        let busy = self.action.is_some();
        let (state, detail, label, command) = match stage {
            Some(Stage::Start) => ("Not started", "Start Windows, then install the guest tools from its driver CD.", "Start setup…", FirstBootAction::Start),
            Some(Stage::Drivers) => ("Drivers needed", "In Windows, run virtio-win-guest-tools.exe from the driver CD. Then return here.", "Open Windows…", FirstBootAction::Start),
            Some(Stage::SignIn) => ("Sign-in needed", "Sign in to Windows, then return here to finish setup.", "Open Windows…", FirstBootAction::Start),
            Some(Stage::Ready) => ("Ready to finish", "Restarting closes Windows apps and finishes driver setup.", "Restart Windows", FirstBootAction::Finish),
            Some(Stage::GpuBoot) => ("GPU boot needed", "Shut down Windows, then reboot Linux into the GPU profile from the window-side panel.", "Check boot profile", FirstBootAction::Status),
            _ => ("Checking", "", "Start setup…", FirstBootAction::Start),
        };
        let content = div().flex().flex_col().gap_6().p_6()
            .child(div().flex().items_center().justify_between()
                .child(heading("First boot", "", cx))
                .child(div().flex().items_center().gap_2().text_sm()
                    .child(div().size_2().rounded_full().bg(cx.theme().warning))
                    .child(self.action.unwrap_or(state).to_owned())))
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child(detail.to_owned()))
            .when_some(self.first_boot.as_ref(), |view, status| {
                view.child(div().flex().flex_col()
                    .child(field("Storage driver", if status.storage_ready == Some(true) { "Ready" } else { "Pending" }, cx))
                    .child(field("Network driver", if status.network_ready == Some(true) { "Ready" } else { "Pending" }, cx)))
            })
            .child(self.error_view(cx));
        div().flex().flex_col().flex_1().min_h_0()
            .child(div().id("first-boot-scroll").flex_1().min_h_0().overflow_y_scroll().child(content))
            .child(div().flex().items_center().gap_2().p_4().border_t_1().border_color(cx.theme().border)
                .child(Button::new("first-boot-primary").label(label).disabled(busy || stage.is_none()).loading(busy)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.run_action(if matches!(command, FirstBootAction::Finish) { "Finishing setup" } else { "Opening setup" },
                            Action::Setup { step: SetupStep::FirstBoot { action: command } }, cx);
                    })))
                .when(stage != Some(Stage::GpuBoot), |view| view.child(Button::new("first-boot-check").label("Check again").disabled(busy || self.refreshing)
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))))))
            .into_any_element()
    }
}
