use super::{Resource, WindowSide, fixture};
use crate::setup;
use gpui_kit::*;

impl WindowSide {
    pub(super) fn adjust_resource(
        &mut self,
        resource: Resource,
        delta: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.planning || self.applying {
            return;
        }
        let slider = match resource {
            Resource::Memory => self.memory.clone(),
            Resource::Cpus => self.cpus.clone(),
        };
        slider.update(cx, |state, cx| {
            let value = (state.value().start() + delta).clamp(state.min_value(), state.max_value());
            state.set_value(value, window, cx);
        });
        self.plan = None;
        cx.notify();
    }

    pub(super) fn prepare_plan(&mut self, cx: &mut Context<Self>) {
        let (Some(inventory), Some(choices)) = (&self.inventory, &mut self.choices) else {
            return;
        };
        choices.memory_mib =
            (self.memory.read(cx).value().start().round() as u32).saturating_mul(1024);
        choices.vcpus = self.cpus.read(cx).value().start().round() as u32;
        if self.fixture {
            self.plan = Some(fixture::plan(choices));
            self.step = 3;
            return;
        }
        self.planning = true;
        let inventory = inventory.clone();
        let choices = choices.clone();
        let existing = self.config.clone();
        let task = cx
            .background_spawn(async move { setup::plan(&inventory, &choices, existing.as_ref()) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.planning = false;
                match result {
                    Ok(plan) => {
                        this.plan = Some(plan);
                        this.step = 3;
                    }
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn apply_plan(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.applying || self.fixture {
            return;
        }
        let Some(plan) = self.plan.clone() else {
            return;
        };
        self.focus.focus(window, cx);
        self.applying = true;
        self.error = None;
        let task = cx.background_spawn(async move { setup::apply(&plan) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.applying = false;
                match result {
                    Ok(()) => {
                        this.setup_open = false;
                        this.detect(cx);
                    }
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
