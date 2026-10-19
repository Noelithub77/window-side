//! The Windows, graphics and resources setup steps.
use super::{
    Resource, WindowSide,
    widgets::{choice_card, chip, gib, heading, icon_tile, legend, split_bar, thread_grid},
};
use crate::{config::DiskMode, detect::gpus::Blocker};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, Sizable,
    button::{Button, ButtonVariants},
    radio::Radio,
    slider::Slider,
    switch::Switch,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

fn title_block(title: impl Into<SharedString>, detail: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap_0p5()
        .child(div().font_weight(FontWeight::MEDIUM).truncate().child(title.into()))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .truncate()
                .child(detail.into()),
        )
}

impl WindowSide {
    fn select_windows(&mut self, partition: &str, cx: &mut Context<Self>) {
        if let Some(choices) = &mut self.choices {
            choices.windows = partition.to_owned();
        }
        self.plan = None;
        cx.notify();
    }

    fn select_gpu(&mut self, gpu: Option<String>, cx: &mut Context<Self>) {
        if let Some(choices) = &mut self.choices {
            choices.gpu = gpu;
        }
        self.plan = None;
        cx.notify();
    }

    pub(super) fn windows_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div()
            .flex()
            .flex_col()
            .gap_4()
            .child(heading("Choose Windows", "Detected partitions", cx));
        let Some(inventory) = &self.inventory else {
            return content;
        };
        if inventory.windows.is_empty() {
            return content.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .py_12()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(Icon::new(IconName::HardDrive).with_size(px(32.)))
                    .child("No Windows partitions found"),
            );
        }
        let theme = cx.theme();
        for (index, install) in inventory.windows.iter().enumerate() {
            let partition = install.partition.clone();
            let selected = self
                .choices
                .as_ref()
                .is_some_and(|choice| choice.windows == partition);
            let disk = if install.disk_model.is_empty() {
                install.disk.clone()
            } else {
                install.disk_model.clone()
            };
            let other = install.disk_size.saturating_sub(install.size);
            let (other_label, other_color) = if install.shares_linux {
                ("Linux", theme.info)
            } else {
                ("Other", theme.muted_foreground.opacity(0.5))
            };
            let select = partition.clone();
            content = content.child(
                choice_card(("drive", index), selected, true, cx)
                    .on_click(cx.listener(move |this, _, _, cx| this.select_windows(&select, cx)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(icon_tile(
                                IconName::HardDrive,
                                if selected { theme.primary } else { theme.muted_foreground },
                                cx,
                            ))
                            .child(title_block(disk, format!("{partition} · {}", gib(install.size)), cx))
                            .when(install.bitlocker, |view| {
                                view.child(chip(Some(IconName::LockKeyhole), "BitLocker", theme.warning))
                            })
                            .child(chip(
                                None,
                                if install.mode == DiskMode::Partition {
                                    "Shared drive"
                                } else {
                                    "Dedicated drive"
                                },
                                theme.muted_foreground,
                            ))
                            .child(Radio::new(SharedString::from(partition.clone())).checked(selected).on_change(
                                cx.listener(move |this, _, _, cx| this.select_windows(&partition, cx)),
                            )),
                    )
                    .child(split_bar(
                        ("drive-bar", index),
                        vec![(install.size as f32, theme.primary), (other as f32, other_color)],
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(legend(theme.primary, "Windows", gib(install.size), cx))
                            .when(other > 0, |view| {
                                view.child(legend(other_color, other_label, gib(other), cx))
                            })
                            .child(div().flex_1())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{} drive", gib(install.disk_size))),
                            ),
                    )
                    .when(install.bitlocker, |view| {
                        view.child(
                            div()
                                .text_xs()
                                .text_color(theme.warning)
                                .child("Recovery key required for the VM"),
                        )
                    }),
            );
        }
        content
    }

    pub(super) fn graphics_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap_3().child(heading(
            "Graphics",
            "Passthrough reserves a GPU for Windows in the GPU boot profile.",
            cx,
        ));
        let Some(inventory) = &self.inventory else {
            return content;
        };
        let theme = cx.theme();
        let none = self.choices.as_ref().is_none_or(|choices| choices.gpu.is_none());
        content = content.child(
            choice_card("no-gpu-card", none, true, cx)
                .on_click(cx.listener(|this, _, _, cx| this.select_gpu(None, cx)))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(icon_tile(
                            IconName::Monitor,
                            if none { theme.primary } else { theme.muted_foreground },
                            cx,
                        ))
                        .child(title_block("No passthrough", "Virtual display · Linux keeps every GPU", cx))
                        .child(
                            Radio::new("no-gpu")
                                .checked(none)
                                .on_change(cx.listener(|this, _, _, cx| this.select_gpu(None, cx))),
                        ),
                ),
        );
        for gpu in &inventory.gpus {
            let address = gpu.address.clone();
            let usable = inventory
                .usable_gpus()
                .any(|candidate| candidate.address == address);
            let selected = self
                .choices
                .as_ref()
                .is_some_and(|choices| choices.gpu.as_deref() == Some(&address));
            let (status, icon, color) = if inventory.system.limine_kernel.is_none() {
                ("Requires Limine", IconName::TriangleAlert, theme.warning)
            } else {
                match gpu.blocker {
                    Some(Blocker::HostDisplay) => ("Drives the Linux display", IconName::TriangleAlert, theme.warning),
                    Some(Blocker::SharedGroup) => ("Shared device group", IconName::TriangleAlert, theme.warning),
                    Some(Blocker::IommuOff) => ("IOMMU on after reboot", IconName::Info, theme.info),
                    None => ("Isolated", IconName::ShieldCheck, theme.primary),
                }
            };
            let select = address.clone();
            content = content.child(
                choice_card(SharedString::from(format!("gpu-{address}")), selected, usable, cx)
                    .when(usable, |view| {
                        view.on_click(cx.listener(move |this, _, _, cx| {
                            this.select_gpu(Some(select.clone()), cx)
                        }))
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(icon_tile(
                                IconName::Gpu,
                                if selected { theme.primary } else { theme.muted_foreground },
                                cx,
                            ))
                            .child(title_block(gpu.name.clone(), address.clone(), cx))
                            .child(chip(Some(icon), status, color))
                            .child(
                                Radio::new(SharedString::from(address.clone()))
                                    .disabled(!usable)
                                    .checked(selected)
                                    .on_change(cx.listener(move |this, _, _, cx| {
                                        this.select_gpu(Some(address.clone()), cx)
                                    })),
                            ),
                    ),
            );
        }
        content
    }

    fn resource_card(&self, resource: Resource, cx: &Context<Self>) -> Div {
        let Some(inventory) = &self.inventory else {
            return div();
        };
        let theme = cx.theme();
        let (id, slider, icon, title, unit, total) = match resource {
            Resource::Memory => (
                "memory",
                &self.memory,
                IconName::MemoryStick,
                "Windows memory",
                " GiB",
                (inventory.system.memory_mib / 1024) as usize,
            ),
            Resource::Cpus => (
                "cpus",
                &self.cpus,
                IconName::Cpu,
                "Windows CPU threads",
                "",
                inventory.system.cpus.len(),
            ),
        };
        let state = slider.read(cx);
        let current = state.value().start();
        let windows = (current.round() as usize).min(total);
        let linux = total - windows;
        let busy = self.planning || self.applying;
        let step_button = |name: &str, label: &'static str, delta: f32, disabled: bool| {
            Button::new(SharedString::from(format!("{name}-{id}")))
                .ghost()
                .small()
                .icon(Icon::new(if delta < 0. { IconName::Minus } else { IconName::Plus }))
                .accessibility_label(format!("{label} {title}"))
                .tooltip(format!("{label} {title}"))
                .disabled(disabled)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.adjust_resource(resource, delta, window, cx)
                }))
        };
        let visual = match resource {
            Resource::Memory => split_bar(
                "memory-split",
                vec![(windows as f32, theme.primary), (linux as f32, theme.info)],
                cx,
            ),
            Resource::Cpus => thread_grid(total, windows, cx),
        };
        let unit_label = |count: usize| {
            if unit.is_empty() { count.to_string() } else { format!("{count}{unit}") }
        };
        super::widgets::card(cx)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(icon_tile(icon, theme.primary, cx))
                    .child(title_block(title, format!("of {}", unit_label(total)), cx))
                    .child(step_button("decrease", "Decrease", -1., busy || current <= state.min_value()))
                    .child(
                        div()
                            .min_w_16()
                            .text_center()
                            .text_2xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(unit_label(windows)),
                    )
                    .child(step_button("increase", "Increase", 1., busy || current >= state.max_value())),
            )
            .child(Slider::new(slider).disabled(busy))
            .child(visual)
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(legend(theme.primary, "Windows", unit_label(windows), cx))
                    .child(legend(theme.info, "Linux", unit_label(linux), cx)),
            )
    }

    pub(super) fn resources_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div()
            .flex()
            .flex_col()
            .gap_4()
            .child(heading("Resources", "Leave room for Linux and your other apps.", cx));
        let Some(inventory) = &self.inventory else {
            return content;
        };
        content = content
            .child(self.resource_card(Resource::Memory, cx))
            .child(self.resource_card(Resource::Cpus, cx));
        if inventory.system.omarchy {
            let theme = cx.theme();
            content = content.child(
                super::widgets::card(cx).child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(icon_tile(IconName::Layers, theme.primary, cx))
                        .child(title_block("Omarchy panel", "Start and watch Windows from the bar", cx))
                        .child(
                            Switch::new("install-plugin")
                                .disabled(self.planning || self.applying)
                                .accessibility_label("Add the Omarchy panel")
                                .checked(self.choices.as_ref().is_some_and(|choices| choices.plugin))
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    if let Some(choices) = &mut this.choices {
                                        choices.plugin = *checked;
                                    }
                                    this.plan = None;
                                    cx.notify();
                                })),
                        ),
                ),
            );
        }
        content
    }
}
