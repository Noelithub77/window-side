//! Window frame: title bar navigation, the shared error banner and page routing.
use super::{Page, WindowSide, widgets::windows_mark};
use gpui_kit::component::{
    ActiveTheme, Sizable, TitleBar,
    alert::Alert,
    button::{Button, ButtonVariants},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl WindowSide {
    fn nav(&self, cx: &Context<Self>) -> Div {
        let tab = |id: &'static str, label: &'static str, page: Page| {
            let selected = self.page == page;
            Button::new(id)
                .small()
                .label(label)
                .when(selected, |button| button.primary())
                .when(!selected, |button| button.ghost())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.page = page;
                    this.armed = None;
                    cx.notify();
                }))
        };
        div()
            .flex()
            .gap_1()
            .p_1()
            .rounded_full()
            .bg(cx.theme().muted)
            .child(tab("nav-overview", "Overview", Page::Overview))
            .child(tab("nav-settings", "Settings", Page::Settings))
    }

    pub(super) fn header(&self, cx: &Context<Self>) -> TitleBar {
        let navigable = !self.welcome && !self.setup_open && self.config.is_some();
        TitleBar::new().child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .flex_1()
                .px_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(windows_mark(px(6.), cx.theme().primary))
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).child("window-side")),
                )
                .when(navigable, |view| view.child(self.nav(cx))),
        )
    }

    pub(super) fn error_view(&self) -> Div {
        let error = if self.setup_open {
            self.error.as_ref()
        } else {
            self.error.as_ref().or(self.metrics_error.as_ref())
        };
        div().when_some(error, |view, error| {
            view.child(Alert::error("error", SharedString::from(error.clone())))
        })
    }

    pub(super) fn page_view(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.welcome {
            self.welcome_view(cx)
        } else if self.setup_open {
            self.setup_view(cx)
        } else {
            match self.page {
                Page::Overview => self.overview_view(cx),
                Page::Settings => self.settings_view(cx),
            }
        }
    }
}
