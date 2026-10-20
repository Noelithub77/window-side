use crate::config::{config_dir, home};
use anyhow::{Context, Result};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, px, rgb};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Appearance {
    #[default]
    Teal,
    Omarchy,
}

#[derive(Default, Deserialize, Serialize)]
struct Preferences {
    #[serde(default)]
    appearance: Appearance,
}

impl Appearance {
    /// `preferences.json`, or the `appearance.toml` written before it.
    pub(super) fn load() -> Self {
        let json = fs::read_to_string(config_dir().join("preferences.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<Preferences>(&text).ok());
        json.or_else(|| {
            fs::read_to_string(config_dir().join("appearance.toml"))
                .ok()
                .and_then(|text| toml::from_str::<Preferences>(&text).ok())
        })
        .unwrap_or_default()
        .appearance
    }

    pub(super) fn save(self) -> Result<()> {
        fs::create_dir_all(config_dir())?;
        fs::write(
            config_dir().join("preferences.json"),
            serde_json::to_string_pretty(&Preferences { appearance: self })? + "\n",
        )?;
        Ok(())
    }

    pub(super) fn apply(self, cx: &mut App) -> Result<()> {
        let palette = match self {
            Self::Teal => Palette::teal(),
            Self::Omarchy => Palette::omarchy()?,
        };
        Theme::change(ThemeMode::Dark, None, cx);
        Theme::update(cx, |theme| {
            theme.radius = px(12.);
            theme.radius_lg = px(22.);
            let c = &mut theme.colors;
            c.background = palette.background;
            c.foreground = palette.foreground;
            c.group_box = palette.surface;
            c.group_box_foreground = palette.foreground;
            c.popover = palette.raised;
            c.popover_foreground = palette.foreground;
            c.sidebar = palette.surface;
            c.sidebar_foreground = palette.foreground;
            c.sidebar_border = palette.border;
            c.title_bar = palette.background;
            c.title_bar_border = palette.border;
            c.muted = palette.raised;
            c.muted_foreground = palette.muted;
            c.border = palette.border;
            c.input = palette.border;
            c.ring = palette.accent;
            c.caret = palette.accent;
            c.accent = palette.raised;
            c.accent_foreground = palette.foreground;
            c.selection = palette.accent.opacity(0.3);
            c.primary = palette.accent;
            c.primary_foreground = palette.background;
            c.primary_hover = palette.hover;
            c.primary_active = palette.active;
            c.button_primary = palette.accent;
            c.button_primary_foreground = palette.background;
            c.button_primary_hover = palette.hover;
            c.button_primary_active = palette.active;
            c.button = palette.raised;
            c.button_foreground = palette.foreground;
            c.button_hover = palette.border;
            c.button_active = palette.raised;
            c.secondary = palette.raised;
            c.secondary_foreground = palette.foreground;
            c.secondary_hover = palette.border;
            c.secondary_active = palette.raised;
            c.list = palette.background;
            c.list_active = palette.raised;
            c.list_active_border = palette.accent;
            c.list_hover = palette.surface;
            c.tab_bar = palette.background;
            c.tab_active = palette.surface;
            c.tab_active_foreground = palette.foreground;
            c.tab_foreground = palette.muted;
            c.slider_bar = palette.accent;
            c.slider_thumb = palette.foreground;
            c.switch = palette.border;
            c.progress_bar = palette.accent;
            c.skeleton = palette.raised;
            c.success = palette.accent;
            c.success_foreground = palette.background;
            c.warning = palette.warning;
            c.warning_foreground = palette.background;
            c.danger = palette.danger;
            c.danger_foreground = palette.background;
            c.info = palette.linux;
            c.info_foreground = palette.background;
            c.chart_1 = palette.accent;
            c.chart_2 = palette.linux;
            c.chart_3 = palette.pink;
            c.chart_4 = palette.sky;
            c.chart_5 = palette.warning;
            c.chart_grid = palette.border;
        });
        Ok(())
    }
}

/// Windows uses the accent; Linux's share of the machine uses `info` (chart 2).
/// Charts: cpu 1, memory 2, gpu 3, network down 4 and up 5.
struct Palette {
    background: Hsla,
    surface: Hsla,
    raised: Hsla,
    border: Hsla,
    foreground: Hsla,
    muted: Hsla,
    accent: Hsla,
    hover: Hsla,
    active: Hsla,
    linux: Hsla,
    pink: Hsla,
    sky: Hsla,
    warning: Hsla,
    danger: Hsla,
}

impl Palette {
    fn teal() -> Self {
        Self {
            background: rgb(0x0c0c12).into(),
            surface: rgb(0x14141c).into(),
            raised: rgb(0x1e1e2a).into(),
            border: rgb(0x2a2a38).into(),
            foreground: rgb(0xecebf5).into(),
            muted: rgb(0x9c9bb0).into(),
            accent: rgb(0x8ce9d2).into(),
            hover: rgb(0xb0f2e2).into(),
            active: rgb(0x6fdcc1).into(),
            linux: rgb(0xb8a9ff).into(),
            pink: rgb(0xff9ec7).into(),
            sky: rgb(0x95c8ff).into(),
            warning: rgb(0xffc78f).into(),
            danger: rgb(0xff8f8f).into(),
        }
    }

    fn omarchy() -> Result<Self> {
        let text = fs::read_to_string(home().join(".config/omarchy/current/theme/colors.toml"))
            .context("The active Omarchy theme has no colors.toml")?;
        let colors: toml::Table =
            toml::from_str(&text).context("Could not read Omarchy theme colors")?;
        let color = |key: &str| -> Result<Hsla> {
            let value = colors
                .get(key)
                .and_then(toml::Value::as_str)
                .with_context(|| format!("Theme has no {key} color"))?;
            let hex = value.trim_start_matches('#');
            anyhow::ensure!(hex.len() == 6, "Invalid {key} color");
            Ok(rgb(u32::from_str_radix(hex, 16)?).into())
        };
        let background = color("background")?;
        let accent = color("accent")?;
        Ok(Self {
            background,
            foreground: color("foreground")?,
            accent,
            surface: color("color0")?,
            raised: background.blend(color("color8")?.opacity(0.35)),
            border: color("color8")?,
            muted: color("color7")?,
            hover: color("color6")?,
            active: accent,
            linux: color("color5")?,
            pink: color("color1")?,
            sky: color("color4")?,
            warning: color("color3")?,
            danger: color("color1")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(color: Hsla) -> f32 {
        let rgb = color.to_rgb();
        let linear = |value: f32| {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(rgb.r) + 0.7152 * linear(rgb.g) + 0.0722 * linear(rgb.b)
    }

    fn contrast(first: Hsla, second: Hsla) -> f32 {
        let (first, second) = (luminance(first), luminance(second));
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    #[test]
    fn teal_text_and_actions_remain_readable_on_every_surface() {
        let palette = Palette::teal();
        for background in [palette.background, palette.surface, palette.raised] {
            assert!(contrast(palette.foreground, background) >= 4.5);
            assert!(contrast(palette.muted, background) >= 4.5);
        }
        for fill in [palette.accent, palette.hover, palette.active] {
            assert!(contrast(palette.background, fill) >= 4.5);
        }
        assert!(contrast(palette.accent, palette.background) >= 3.);
        for chart in [palette.linux, palette.pink, palette.sky, palette.warning] {
            assert!(contrast(chart, palette.surface) >= 3.);
        }
    }
}
