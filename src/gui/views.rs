use super::{Resource, WindowSide, theme::Appearance};
use crate::cli::Action;
use gpui_kit::component::{
    ActiveTheme, Disableable, Sizable, TitleBar,
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenuItem},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(super) fn heading(title: &str, detail: &str, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_owned()),
        )
        .when(!detail.is_empty(), |view| {
            view.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(detail.to_owned()),
            )
        })
}

pub(super) fn field(label: &str, value: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_6()
        .py_3()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_owned()),
        )
        .child(div().text_sm().text_right().child(value.into()))
}

impl WindowSide {
    pub(super) fn resource_field(
        &self,
        label: &'static str,
        value: String,
        resource: Resource,
        cx: &Context<Self>,
    ) -> Div {
        let (id, slider) = match resource {
            Resource::Memory => ("memory", &self.memory),
            Resource::Cpus => ("cpus", &self.cpus),
        };
        let state = slider.read(cx);
        let current = state.value().start();
        let busy = self.planning || self.applying;
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .py_3()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new(SharedString::from(format!("decrease-{id}")))
                            .ghost()
                            .small()
                            .label("−")
                            .accessibility_label(format!("Decrease {label}"))
                            .tooltip(format!("Decrease {label}"))
                            .disabled(busy || current <= state.min_value())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.adjust_resource(resource, -1., window, cx)
                            })),
                    )
                    .child(div().text_sm().child(value))
                    .child(
                        Button::new(SharedString::from(format!("increase-{id}")))
                            .ghost()
                            .small()
                            .label("+")
                            .accessibility_label(format!("Increase {label}"))
                            .tooltip(format!("Increase {label}"))
                            .disabled(busy || current >= state.max_value())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.adjust_resource(resource, 1., window, cx)
                            })),
                    ),
            )
    }

    pub(super) fn header(&self, cx: &Context<Self>) -> TitleBar {
        let weak = cx.entity().downgrade();
        let current = self.appearance;
        TitleBar::new().child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .flex_1()
                .px_4()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .child("window-side"),
                )
                .child(
                    Button::new("appearance")
                        .ghost()
                        .small()
                        .label("Appearance")
                        .dropdown_menu(move |mut menu, _, _| {
                            for (label, appearance) in
                                [("Teal", Appearance::Teal), ("Omarchy", Appearance::Omarchy)]
                            {
                                let weak = weak.clone();
                                menu = menu.item(
                                    PopupMenuItem::new(label)
                                        .checked(current == appearance)
                                        .on_click(move |_, _, cx| {
                                            let _ = weak.update(cx, |this, cx| {
                                                this.set_appearance(appearance, cx)
                                            });
                                        }),
                                );
                            }
                            menu
                        }),
                ),
        )
    }

    pub(super) fn error_view(&self, cx: &App) -> Div {
        let error = if self.setup_open {
            self.error.as_ref()
        } else {
            self.error.as_ref().or(self.metrics_error.as_ref())
        };
        div().when_some(error, |view, error| {
            view.p_3()
                .border_1()
                .border_color(cx.theme().danger)
                .rounded(cx.theme().radius)
                .text_sm()
                .text_color(cx.theme().foreground)
                .child(error.clone())
        })
    }

    pub(super) fn status_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let running = self.metrics.running();
        let busy = self.action.is_some();
        let known = matches!(self.metrics.state(), "Running" | "Stopped");
        let state = self.action.unwrap_or(self.metrics.state());
        let mut facts = div()
            .flex()
            .flex_col()
            .gap_0()
            .p_4()
            .bg(cx.theme().group_box)
            .rounded(cx.theme().radius)
            .child(field("Cpu", self.metrics.cpu(), cx))
            .child(field("Memory", self.metrics.memory(), cx))
            .child(field("Network total", self.metrics.network(), cx))
            .child(field("Windows uptime", self.metrics.uptime(), cx));
        facts = facts.child(
            div()
                .border_t_1()
                .border_color(cx.theme().border)
                .mt_2()
                .pt_2()
                .child(field("Boot profile", self.metrics.profile(), cx)),
        );
        let configure = Button::new("configure")
            .label("Configure…")
            .ghost()
            .disabled(busy)
            .on_click(cx.listener(|this, _, window, cx| {
                this.focus.focus(window, cx);
                this.setup_open = true;
                this.step = 0;
                this.error = None;
                cx.notify();
            }));
        let actions = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("open-windows")
                    .label(if running {
                        "Open Windows…"
                    } else {
                        "Start Windows"
                    })
                    .disabled(busy || !known)
                    .loading(busy)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.run_action(
                            if running { "Opening" } else { "Starting" },
                            if running {
                                Action::Console
                            } else {
                                Action::Start
                            },
                            cx,
                        );
                    })),
            )
            .when(running, |view| {
                view.child(
                    Button::new("stop-windows")
                        .label("Shut down")
                        .outline()
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.run_action("Stopping", Action::Stop, cx)
                        })),
                )
            })
            .child(div().flex_1())
            .child(configure);
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .id("status-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_6()
                            .p_6()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(heading("Windows", "", cx))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .text_sm()
                                            .child(div().size_2().rounded_full().bg(if running {
                                                cx.theme().success
                                            } else {
                                                cx.theme().muted_foreground
                                            }))
                                            .child(state.to_owned()),
                                    ),
                            )
                            .child(facts)
                            .child(self.error_view(cx)),
                    ),
            )
            .child(
                div()
                    .p_4()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(actions),
            )
            .into_any_element()
    }
}
