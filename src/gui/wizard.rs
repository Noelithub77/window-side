//! Setup frame: step navigation, the active step and its footer actions.
use super::{
    WindowSide,
    widgets::{card, heading, reveal},
};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Sizable,
    button::{Button, ButtonVariants},
    skeleton::Skeleton,
    spinner::Spinner,
    stepper::{Stepper, StepperItem},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

const STEPS: [(&str, IconName); 4] = [
    ("Windows", IconName::HardDrive),
    ("Graphics", IconName::Gpu),
    ("Resources", IconName::Cpu),
    ("Review", IconName::ClipboardCheck),
];

impl WindowSide {
    fn detecting_view(&self, cx: &App) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(Spinner::new().color(cx.theme().primary).with_size(px(20.)))
                    .child(heading("Finding Windows", "Reading disks, graphics and resources…", cx)),
            )
            .children((0..2).map(|_| {
                card(cx)
                    .child(div().flex().gap_3().child(Skeleton::new().size_10()).child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .flex_1()
                            .child(Skeleton::new().h_4().w(relative(0.4)))
                            .child(Skeleton::new().h_3().w(relative(0.25))),
                    ))
                    .child(Skeleton::new().h_2p5().w_full())
            }))
    }

    fn sidebar(&self, busy: bool, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme();
        let host = self.inventory.as_ref().map(|inventory| {
            (
                inventory.system.cpus.len(),
                inventory.system.memory_mib / 1024,
            )
        });
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w_56()
            .p_5()
            .gap_6()
            .bg(theme.sidebar)
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child("Setup"),
            )
            .child(
                Stepper::new("setup-steps")
                    .vertical()
                    .selected_index(self.step)
                    .disabled(busy)
                    .items(STEPS.into_iter().enumerate().map(|(step, (label, icon))| {
                        StepperItem::new()
                            .icon(Icon::new(icon))
                            .child(label)
                            .disabled(step > self.step)
                    }))
                    .on_click(cx.listener(|this, step, window, cx| {
                        if *step < this.step {
                            this.focus.focus(window, cx);
                            this.step = *step;
                            this.plan = None;
                            this.error = None;
                            cx.notify();
                        }
                    })),
            )
            .child(div().flex_1())
            .when_some(host, |view, (threads, memory)| {
                let row = |icon: IconName, value: String| {
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(icon).with_size(px(14.)))
                        .child(value)
                };
                view.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .pt_4()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(div().text_xs().font_weight(FontWeight::MEDIUM).child("This PC"))
                        .child(row(IconName::Cpu, format!("{threads} threads")))
                        .child(row(IconName::MemoryStick, format!("{memory} GiB memory"))),
                )
            })
    }

    fn footer(&self, busy: bool, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_6()
            .py_4()
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("back")
                    .label("Back")
                    .icon(Icon::new(IconName::ArrowLeft))
                    .ghost()
                    .disabled(busy || self.step == 0)
                    .on_click(cx.listener(|this, _, window, cx| this.back(window, cx))),
            )
            .child(div().flex_1())
            .when(self.config.is_some(), |view| {
                view.child(
                    Button::new("cancel-setup")
                        .label("Cancel")
                        .ghost()
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.focus.focus(window, cx);
                            this.setup_open = false;
                            this.error = None;
                            cx.notify();
                        })),
                )
            })
            .when(self.step == 0, |view| {
                view.child(
                    Button::new("scan")
                        .label("Scan again")
                        .icon(Icon::new(IconName::RefreshCw))
                        .outline()
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| this.detect(cx))),
                )
            })
            .child(if self.step == 3 {
                Button::new("apply-setup")
                    .label("Apply setup…")
                    .icon(Icon::new(IconName::ShieldCheck))
                    .primary()
                    .loading(self.applying)
                    .disabled(busy || self.plan.is_none() || self.fixture)
                    .on_click(cx.listener(|this, _, window, cx| this.apply_plan(window, cx)))
            } else {
                Button::new("next-step")
                    .label(if self.step == 2 { "Review" } else { "Continue" })
                    .primary()
                    .loading(self.planning)
                    .disabled(busy || self.choices.is_none())
                    .on_click(cx.listener(|this, _, window, cx| this.next(window, cx)))
            })
    }

    pub(super) fn setup_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let busy = self.detecting || self.planning || self.applying;
        let (key, body) = if self.detecting {
            (SharedString::from("detecting"), self.detecting_view(cx))
        } else {
            (
                SharedString::from(format!("step-{}", self.step)),
                match self.step {
                    0 => self.windows_step(cx),
                    1 => self.graphics_step(cx),
                    2 => self.resources_step(cx),
                    _ => self.review_step(cx),
                },
            )
        };
        div()
            .flex()
            .flex_1()
            .min_h_0()
            .child(self.sidebar(busy, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .id("setup-scroll")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .child(reveal(
                                key,
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .p_6()
                                    .child(body)
                                    .child(self.error_view()),
                            )),
                    )
                    .child(self.footer(busy, cx)),
            )
            .into_any_element()
    }
}
