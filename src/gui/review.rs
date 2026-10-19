//! The final setup step: what will be applied, before administrator approval.
use super::{
    WindowSide,
    widgets::{card, field, heading, icon_tile},
};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme, Icon, Sizable,
    alert::Alert,
    button::{Button, ButtonVariants},
};
use gpui_kit::*;

fn summary_tile(icon: IconName, label: &str, value: impl Into<SharedString>, cx: &App) -> Div {
    card(cx)
        .flex_row()
        .items_center()
        .gap_3()
        .p_4()
        .child(icon_tile(icon, cx.theme().primary, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .gap_0p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(label.to_owned()),
                )
                .child(div().font_weight(FontWeight::MEDIUM).truncate().child(value.into())),
        )
}

impl WindowSide {
    pub(super) fn review_step(&self, cx: &Context<Self>) -> Div {
        let mut content = div().flex().flex_col().gap_4().child(heading(
            "Review setup",
            "Changes are applied after administrator approval.",
            cx,
        ));
        let Some(plan) = &self.plan else {
            return content;
        };
        let passthrough = plan.config.gpu.is_some();
        content = content.child(
            div()
                .grid()
                .grid_cols(2)
                .gap_3()
                .child(summary_tile(
                    IconName::HardDrive,
                    "Windows",
                    self.choices
                        .as_ref()
                        .map_or_else(|| "—".to_owned(), |choices| choices.windows.clone()),
                    cx,
                ))
                .child(summary_tile(
                    if passthrough { IconName::Gpu } else { IconName::Monitor },
                    "Graphics",
                    if passthrough { "Passthrough" } else { "Virtual display" },
                    cx,
                ))
                .child(summary_tile(
                    IconName::MemoryStick,
                    "Memory",
                    format!("{} GiB", plan.config.vm.memory_mib / 1024),
                    cx,
                ))
                .child(summary_tile(
                    IconName::Cpu,
                    "CPU threads",
                    plan.config.vm.vcpus.to_string(),
                    cx,
                ))
                .child(summary_tile(
                    IconName::LayoutDashboard,
                    "Omarchy panel",
                    if plan.plugin { "Enabled" } else { "Off" },
                    cx,
                )),
        );
        if !plan.root.packages.is_empty() {
            content = content.child(summary_tile(
                IconName::Layers,
                "Packages to install",
                plan.root.packages.join(", "),
                cx,
            ));
        }
        for (index, note) in plan.notes.iter().enumerate() {
            content = content.child(Alert::info(("note", index), SharedString::from(note.clone())));
        }
        let mut paths: Vec<String> = plan.root.files.iter().map(|file| file.path.clone()).collect();
        paths.extend([
            "~/.config/window-side/config.toml".into(),
            "~/.config/window-side/domain.xml".into(),
        ]);
        paths.extend(plan.plugin.then(|| "~/.config/omarchy/plugins/noel.window-side/".into()));
        paths.extend(
            plan.root
                .gpu_kernel
                .as_ref()
                .map(|_| "/boot/limine.conf and kernel boot images".into()),
        );
        paths.extend(
            plan.root
                .virtual_disk
                .as_ref()
                .map(|_| "/var/lib/window-side/ boot-file images".into()),
        );
        let theme = cx.theme();
        let mut changes = card(cx).gap_2().p_4().child(
            Button::new("show-files")
                .ghost()
                .small()
                .icon(Icon::new(if self.show_files {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                }))
                .label(format!(
                    "{} changes · {} files",
                    if self.show_files { "Hide" } else { "Show" },
                    paths.len()
                ))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_files = !this.show_files;
                    cx.notify();
                })),
        );
        if self.show_files {
            for path in paths {
                changes = changes.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .pl_2()
                        .text_sm()
                        .font_family(theme.mono_font_family.clone())
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::FileText).with_size(px(14.)))
                        .child(path),
                );
            }
            if !plan.root.groups.is_empty() {
                changes = changes.child(field("Add to groups", plan.root.groups.join(", "), cx));
            }
        }
        content.child(changes)
    }
}
