//! Small visual pieces shared by every view.
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Icon, Sizable,
    button::{Button, ButtonRounded},
    chart::AreaChart,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::VecDeque;
use std::time::Duration;

pub(super) fn heading(title: &str, detail: &str, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_2xl()
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
        .py_2()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_owned()),
        )
        .child(div().text_sm().text_right().child(value.into()))
}

/// Translucent surface so the aurora behind a page glows through.
pub(super) fn card(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .bg(cx.theme().group_box.opacity(0.78))
        .border_1()
        .border_color(cx.theme().border.opacity(0.8))
        .rounded(cx.theme().radius_lg)
}

/// A selectable card: accent outline and a faint accent wash when chosen.
pub(super) fn choice_card(id: impl Into<ElementId>, selected: bool, enabled: bool, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    let accent = theme.primary;
    let base = card(cx).id(id).when(!enabled, |view| view.opacity(0.55));
    if selected {
        base.border_color(accent.opacity(0.7)).bg(linear_gradient(
            120.,
            linear_color_stop(accent.opacity(0.14), 0.),
            linear_color_stop(theme.group_box.opacity(0.78), 0.75),
        ))
    } else if enabled {
        let hover = theme.muted_foreground.opacity(0.4);
        base.cursor_pointer().hover(move |style| style.border_color(hover))
    } else {
        base
    }
}

/// Primary actions use fully rounded buttons.
pub(super) fn pill_button(id: impl Into<ElementId>) -> Button {
    Button::new(id).rounded(ButtonRounded::Size(px(999.)))
}

pub(super) fn icon_tile(icon: IconName, color: Hsla, cx: &App) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size_10()
        .rounded(cx.theme().radius)
        .bg(color.opacity(0.14))
        .child(Icon::new(icon).text_color(color).with_size(px(18.)))
}

pub(super) fn pill(color: Hsla) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap_2()
        .px_3()
        .py_1()
        .rounded_full()
        .bg(color.opacity(0.14))
        .text_xs()
        .text_color(color)
}

pub(super) fn chip(icon: Option<IconName>, label: impl Into<SharedString>, color: Hsla) -> Div {
    pill(color)
        .gap_1p5()
        .when_some(icon, |view, icon| {
            view.child(Icon::new(icon).text_color(color).with_size(px(12.)))
        })
        .child(label.into())
}

pub(super) fn legend(color: Hsla, label: &str, value: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .text_xs()
        .child(div().size_2().rounded_full().bg(color))
        .child(div().text_color(cx.theme().muted_foreground).child(label.to_owned()))
        .child(value.into())
}

/// Proportional bar that sweeps in from the left the first time it is shown.
pub(super) fn split_bar(id: impl Into<ElementId>, segments: Vec<(f32, Hsla)>, cx: &App) -> Div {
    let total: f32 = segments.iter().map(|(amount, _)| amount.max(0.)).sum();
    let mut inner = div().flex().h_full().gap_1();
    for (amount, color) in segments {
        if total > 0. && amount > 0. {
            inner = inner.child(div().h_full().rounded_full().bg(color).w(relative(amount / total)));
        }
    }
    div().h_3().w_full().rounded_full().bg(cx.theme().muted).overflow_hidden().child(
        inner.with_animation(
            id,
            Animation::new(Duration::from_millis(800)).with_easing(ease_out_quint()),
            |bar, progress| bar.w(relative(progress)),
        ),
    )
}

/// One rounded square per host thread, the first `windows` of them given to Windows.
pub(super) fn thread_grid(total: usize, windows: usize, cx: &App) -> Div {
    let theme = cx.theme();
    div().flex().flex_wrap().gap_1p5().children((0..total).map(|thread| {
        let color = if thread < windows { theme.primary } else { theme.info.opacity(0.5) };
        div().size_4().rounded(px(5.)).bg(color)
    }))
}

/// The four-pane window mark, drawn rather than borrowed from a logo.
pub(super) fn windows_mark(pane: Pixels, color: Hsla) -> Div {
    let tile = move |alpha: f32| div().size(pane).rounded(pane * 0.28).bg(color.opacity(alpha));
    div()
        .flex()
        .flex_col()
        .gap(pane * 0.18)
        .child(div().flex().gap(pane * 0.18).child(tile(1.)).child(tile(0.78)))
        .child(div().flex().gap(pane * 0.18).child(tile(0.78)).child(tile(0.55)))
}

/// Soft pastel glow pinned behind the top of a page.
pub(super) fn aurora() -> Img {
    img("illustrations/aurora.png")
        .absolute()
        .top_0()
        .left_0()
        .w_full()
        .h(px(300.))
        .object_fit(ObjectFit::Cover)
}

/// Fades content up into place; a new `id` replays it.
pub(super) fn reveal(id: impl Into<ElementId>, content: Div) -> AnyElement {
    content
        .relative()
        .with_animation(
            id,
            Animation::new(Duration::from_millis(420)).with_easing(ease_out_quint()),
            |view, progress| view.opacity(progress).top(px((1. - progress) * 14.)),
        )
        .into_any_element()
}

/// A dot that sends out one fading ring whenever `pulse` changes.
pub(super) fn live_dot(pulse: usize, color: Hsla, animate: bool) -> Div {
    div()
        .relative()
        .size_2()
        .child(div().absolute().size_full().rounded_full().bg(color))
        .when(animate, |view| {
            view.child(
                div().absolute().rounded_full().border_1().border_color(color).with_animation(
                    ("live-ring", pulse),
                    Animation::new(Duration::from_millis(1400)).with_easing(ease_out_quint()),
                    |ring, progress| {
                        let size = 8. + progress * 14.;
                        ring.size(px(size))
                            .left(px((8. - size) / 2.))
                            .top(px((8. - size) / 2.))
                            .opacity(0.8 * (1. - progress))
                    },
                ),
            )
        })
}

/// A smooth filled wave per series; flat baseline until two samples exist.
pub(super) fn wave(
    id: &'static str,
    series: &[(&VecDeque<f64>, Hsla)],
    max: Option<f64>,
    cx: &App,
) -> Div {
    let points = series.iter().map(|(values, _)| values.len()).min().unwrap_or(0);
    if points < 2 {
        return div()
            .flex()
            .items_end()
            .h_32()
            .child(div().h_px().w_full().bg(cx.theme().border));
    }
    let rows: Vec<(SharedString, Vec<f64>)> = (0..points)
        .map(|index| {
            let values = series
                .iter()
                .map(|(values, _)| values[values.len() - points + index])
                .collect();
            (index.to_string().into(), values)
        })
        .collect();
    let top = max.unwrap_or_else(|| {
        rows.iter()
            .flat_map(|(_, values)| values.iter().copied())
            .fold(1., f64::max)
            * 1.15
    });
    let mut chart = AreaChart::new(rows)
        .id(id)
        .x(|(index, _)| index.clone())
        .x_axis(false)
        .grid(false)
        .y_domain(0., top)
        .interactive(false);
    for (index, (_, color)) in series.iter().enumerate() {
        chart = chart
            .y(move |(_, values): &(SharedString, Vec<f64>)| values[index])
            .stroke(*color)
            .fill(linear_gradient(
                180.,
                linear_color_stop(color.opacity(0.42), 0.),
                linear_color_stop(color.opacity(0.02), 1.),
            ))
            .natural();
    }
    div().h_32().child(chart)
}

pub(super) fn gib(bytes: u64) -> String {
    let gib = bytes as f64 / 1024_f64.powi(3);
    if gib >= 1024. { format!("{:.1} TiB", gib / 1024.) } else { format!("{gib:.0} GiB") }
}
