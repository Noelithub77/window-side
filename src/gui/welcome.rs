//! First screen before setup: what window-side does and what this PC already has.
use super::{
    WindowSide,
    widgets::{aurora, chip, pill_button, reveal},
};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Sizable,
    button::ButtonVariants,
    menu::{DropdownMenu, PopupMenuItem},
    spinner::Spinner,
};
use gpui_kit::*;

impl WindowSide {
    fn readiness(&self, cx: &App) -> Div {
        let theme = cx.theme();
        let row = div().flex().flex_wrap().justify_center().gap_2();
        let Some(inventory) = &self.inventory else {
            return row.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(Spinner::new().color(theme.primary).with_size(px(14.)))
                    .child("Looking for Windows…"),
            );
        };
        let found = inventory.windows.len();
        let missing = inventory.system.missing.len();
        row.child(if found > 0 {
            chip(Some(IconName::CircleCheck), format!("Windows found on {found} partition{}", if found == 1 { "" } else { "s" }), theme.primary)
        } else {
            chip(Some(IconName::TriangleAlert), "No Windows found", theme.warning)
        })
        .child(if inventory.system.iommu_enabled {
            chip(Some(IconName::ShieldCheck), "IOMMU on", theme.info)
        } else {
            chip(Some(IconName::Info), "IOMMU off", theme.muted_foreground)
        })
        .child(if missing == 0 {
            chip(Some(IconName::CircleCheck), "Tools ready", theme.primary)
        } else {
            chip(Some(IconName::HardDriveDownload), format!("{missing} package{} to install", if missing == 1 { "" } else { "s" }), theme.warning)
        })
    }

    /// Hands setup to a local agent, or copies the prompt for one elsewhere.
    fn agent_button(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.copied {
            return pill_button("agent-setup")
                .ghost()
                .icon(Icon::new(IconName::Check))
                .label("Prompt copied")
                .into_any_element();
        }
        if self.agents.is_empty() {
            return pill_button("agent-setup")
                .ghost()
                .icon(Icon::new(IconName::Copy))
                .label("Copy agent prompt")
                .on_click(cx.listener(|this, _, _, cx| this.copy_prompt(cx)))
                .into_any_element();
        }
        let weak = cx.entity().downgrade();
        let agents = self.agents.clone();
        pill_button("agent-setup")
            .ghost()
            .icon(Icon::new(IconName::Bot))
            .label("Set up with an agent")
            .dropdown_menu(move |mut menu, _, _| {
                for agent in agents.iter().copied() {
                    let weak = weak.clone();
                    menu = menu.item(PopupMenuItem::new(agent.name()).icon(IconName::SquareTerminal).on_click(
                        move |_, _, cx| {
                            let _ = weak.update(cx, |this, cx| this.open_agent(agent, cx));
                        },
                    ));
                }
                let weak = weak.clone();
                menu.separator().item(PopupMenuItem::new("Copy prompt").icon(IconName::Copy).on_click(
                    move |_, _, cx| {
                        let _ = weak.update(cx, |this, cx| this.copy_prompt(cx));
                    },
                ))
            })
            .into_any_element()
    }

    pub(super) fn welcome_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let content = div()
            .flex()
            .flex_col()
            .items_center()
            .gap_5()
            .px_8()
            .pb_8()
            .child(img("illustrations/welcome.png").w(px(440.)).h(px(271.)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_3xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Windows, right beside Linux"),
                    )
                    .child(
                        div()
                            .text_color(theme.muted_foreground)
                            .child("Run the Windows you already installed, without rebooting."),
                    ),
            )
            .child(self.readiness(cx))
            .child(
                pill_button("get-started")
                    .primary()
                    .large()
                    .px_8()
                    .label("Get started")
                    .icon(Icon::new(IconName::ArrowRight))
                    .disabled(self.inventory.is_none())
                    .on_click(cx.listener(|this, _, window, cx| this.next(window, cx))),
            )
            .child(self.agent_button(cx))
            .child(self.error_view());
        div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .justify_center()
            .overflow_hidden()
            .child(aurora())
            .child(reveal("welcome", content))
            .into_any_element()
    }
}
