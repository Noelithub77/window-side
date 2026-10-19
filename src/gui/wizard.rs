use super::{
    Resource, WindowSide,
    views::{field, heading},
};
use crate::{config::DiskMode, detect::gpus::Blocker};
use gpui_kit::component::{
    ActiveTheme, Disableable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    radio::Radio,
    slider::Slider,
    stepper::{Stepper, StepperItem},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl WindowSide {
    fn windows_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap_4().child(heading(
            "Choose Windows",
            "Detected partitions",
            cx,
        ));
        let Some(inventory) = &self.inventory else {
            return content;
        };
        if inventory.windows.is_empty() {
            return content.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("No Windows partitions found. Connect the drive and scan again."),
            );
        }
        for install in &inventory.windows {
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
            let details = format!(
                "{} · {:.0} GiB · {}",
                partition,
                install.size as f64 / 1024_f64.powi(3),
                if install.mode == DiskMode::Partition {
                    "Shared with Linux"
                } else {
                    "Dedicated drive"
                }
            );
            content = content.child(
                div()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(if selected {
                        cx.theme().group_box
                    } else {
                        cx.theme().background
                    })
                    .border_1()
                    .border_color(if selected {
                        cx.theme().primary
                    } else {
                        cx.theme().border
                    })
                    .child(
                        Radio::new(SharedString::from(partition.clone()))
                            .label(disk)
                            .checked(selected)
                            .on_change(cx.listener(move |this, _, _, cx| {
                                if let Some(choices) = &mut this.choices {
                                    choices.windows = partition.clone();
                                }
                                this.plan = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .mt_2()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(details),
                    )
                    .when(install.bitlocker, |view| {
                        view.child(
                            div()
                                .mt_2()
                                .text_sm()
                                .text_color(cx.theme().warning)
                                .child("BitLocker · Recovery key required for the VM"),
                        )
                    }),
            );
        }
        content
    }

    fn graphics_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap_4().child(heading(
            "Graphics",
            "Passthrough reserves a GPU for Windows in the GPU boot profile.",
            cx,
        ));
        let Some(inventory) = &self.inventory else {
            return content;
        };
        content = content.child(
            Radio::new("no-gpu")
                .label("No passthrough")
                .checked(
                    self.choices
                        .as_ref()
                        .is_none_or(|choices| choices.gpu.is_none()),
                )
                .on_change(cx.listener(|this, _, _, cx| {
                    if let Some(choices) = &mut this.choices {
                        choices.gpu = None;
                    }
                    this.plan = None;
                    cx.notify();
                })),
        );
        for gpu in &inventory.gpus {
            let address = gpu.address.clone();
            let usable = inventory
                .usable_gpus()
                .any(|candidate| candidate.address == address);
            let detail = if inventory.system.limine_kernel.is_none() {
                "GPU boot entry requires Limine"
            } else {
                match gpu.blocker {
                    Some(Blocker::HostDisplay) => "In use by the Linux display",
                    Some(Blocker::SharedGroup) => "Shares its device group",
                    Some(Blocker::IommuOff) => "IOMMU will be enabled on reboot",
                    None => "Isolated device group",
                }
            };
            content =
                content.child(
                    div()
                        .py_2()
                        .child(
                            Radio::new(SharedString::from(address.clone()))
                                .label(gpu.name.clone())
                                .disabled(!usable)
                                .checked(self.choices.as_ref().is_some_and(|choices| {
                                    choices.gpu.as_deref() == Some(&address)
                                }))
                                .on_change(cx.listener(move |this, _, _, cx| {
                                    if let Some(choices) = &mut this.choices {
                                        choices.gpu = Some(address.clone());
                                    }
                                    this.plan = None;
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .mt_2()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(detail),
                        ),
                );
        }
        content
    }

    fn resources_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap_6().child(heading(
            "Resources",
            "Leave room for Linux and your other apps.",
            cx,
        ));
        let Some(inventory) = &self.inventory else {
            return content;
        };
        let memory = self.memory.read(cx).value().start();
        let cpus = self.cpus.read(cx).value().start();
        content = content
            .child(
                div()
                    .child(self.resource_field(
                        "Windows memory",
                        format!(
                            "{memory:.0} GiB / {:.0} GiB",
                            inventory.system.memory_mib as f64 / 1024.
                        ),
                        Resource::Memory,
                        cx,
                    ))
                    .child(Slider::new(&self.memory).disabled(self.planning || self.applying)),
            )
            .child(
                div()
                    .child(self.resource_field(
                        "Windows cpu threads",
                        format!("{cpus:.0} / {}", inventory.system.cpus.len()),
                        Resource::Cpus,
                        cx,
                    ))
                    .child(Slider::new(&self.cpus).disabled(self.planning || self.applying)),
            );
        if inventory.system.omarchy {
            content = content.child(
                Checkbox::new("install-plugin")
                    .disabled(self.planning || self.applying)
                    .label("Add the Omarchy panel")
                    .checked(self.choices.as_ref().is_some_and(|choices| choices.plugin))
                    .on_change(cx.listener(|this, checked, _, cx| {
                        if let Some(choices) = &mut this.choices {
                            choices.plugin = *checked;
                        }
                        this.plan = None;
                        cx.notify();
                    })),
            );
        }
        content
    }

    fn review_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap_4().child(heading(
            "Review setup",
            "Changes are applied after administrator approval.",
            cx,
        ));
        let Some(plan) = &self.plan else {
            return content;
        };
        content = content.child(
            div()
                .p_4()
                .bg(cx.theme().group_box)
                .rounded(cx.theme().radius)
                .child(field(
                    "Windows",
                    self.choices
                        .as_ref()
                        .map_or("—", |choices| choices.windows.as_str()),
                    cx,
                ))
                .child(field(
                    "Memory",
                    format!("{} GiB", plan.config.vm.memory_mib / 1024),
                    cx,
                ))
                .child(field("Cpu threads", plan.config.vm.vcpus.to_string(), cx))
                .child(field(
                    "Graphics",
                    if plan.config.gpu.is_some() {
                        "Passthrough"
                    } else {
                        "Virtual display"
                    },
                    cx,
                ))
                .child(field(
                    "Omarchy panel",
                    if plan.plugin { "Enabled" } else { "Off" },
                    cx,
                )),
        );
        if !plan.root.packages.is_empty() {
            content = content.child(field("Packages", plan.root.packages.join(", "), cx));
        }
        for note in &plan.notes {
            content = content.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(note.clone()),
            );
        }
        content = content.child(
            Button::new("show-files")
                .ghost()
                .label(format!(
                    "{} changes",
                    if self.show_files { "Hide" } else { "Show" }
                ))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_files = !this.show_files;
                    cx.notify();
                })),
        );
        if self.show_files {
            let paths = plan.root.files.iter().map(|file| file.path.clone()).chain([
                "~/.config/window-side/config.toml".into(),
                "~/.config/window-side/domain.xml".into(),
            ]);
            let paths = paths
                .chain(
                    plan.plugin
                        .then_some("~/.config/omarchy/plugins/noel.window-side/".into()),
                )
                .chain(
                    plan.root
                        .gpu_kernel
                        .as_ref()
                        .map(|_| "/boot/limine.conf and kernel boot images".into()),
                )
                .chain(
                    plan.root
                        .virtual_disk
                        .as_ref()
                        .map(|_| "/var/lib/window-side/ boot-file images".into()),
                );
            for path in paths {
                content = content.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(path),
                );
            }
        }
        if self.show_files && !plan.root.groups.is_empty() {
            content = content.child(field("Add to groups", plan.root.groups.join(", "), cx));
        }
        content
    }

    pub(super) fn setup_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let busy = self.detecting || self.planning || self.applying;
        let body = if self.detecting {
            div().flex().flex_col().gap_4().child(heading(
                "Finding Windows",
                "Reading disks, graphics and resources…",
                cx,
            ))
        } else {
            match self.step {
                0 => self.windows_step(cx),
                1 => self.graphics_step(cx),
                2 => self.resources_step(cx),
                _ => self.review_step(cx),
            }
        };
        let mut footer = div().flex().items_center().justify_between().gap_2().child(
            Button::new("back")
                .label("Back")
                .ghost()
                .disabled(busy || self.step == 0)
                .on_click(cx.listener(|this, _, window, cx| this.back(window, cx))),
        );
        footer = footer.child(
            div()
                .flex()
                .items_center()
                .gap_2()
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
                            .outline()
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.detect(cx))),
                    )
                })
                .child(if self.step == 3 {
                    Button::new("apply-setup")
                        .label("Apply setup…")
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
                }),
        );
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .p_6()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        Stepper::new("setup-steps")
                            .selected_index(self.step)
                            .disabled(busy)
                            .items(
                                ["Windows", "Graphics", "Resources", "Review"]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(step, label)| {
                                        StepperItem::new().child(label).disabled(step > self.step)
                                    }),
                            )
                            .on_click(cx.listener(|this, step, window, cx| {
                                if *step < this.step {
                                    this.focus.focus(window, cx);
                                    this.step = *step;
                                    this.plan = None;
                                    this.error = None;
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(
                div()
                    .id("setup-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_6()
                            .child(body)
                            .child(self.error_view(cx)),
                    ),
            )
            .child(
                div()
                    .p_4()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(footer),
            )
            .into_any_element()
    }
}
