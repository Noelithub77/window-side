use crate::config::{config_dir, home};
use anyhow::{Context, Result};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, rgb};
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
    pub(super) fn load() -> Self {
        fs::read_to_string(config_dir().join("appearance.toml"))
            .ok()
            .and_then(|text| toml::from_str::<Preferences>(&text).ok())
            .unwrap_or_default()
            .appearance
    }

    pub(super) fn save(self) -> Result<()> {
        fs::create_dir_all(config_dir())?;
        fs::write(
            config_dir().join("appearance.toml"),
            toml::to_string(&Preferences { appearance: self })?,
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
            let c = &mut theme.colors;
            c.background = palette.background;
            c.foreground = palette.foreground;
            c.group_box = palette.surface;
            c.group_box_foreground = palette.foreground;
            c.popover = palette.surface;
            c.popover_foreground = palette.foreground;
            c.sidebar = palette.surface;
            c.sidebar_foreground = palette.foreground;
            c.title_bar = palette.background;
            c.title_bar_border = palette.border;
            c.muted = palette.surface;
            c.muted_foreground = palette.muted;
            c.border = palette.border;
            c.input = palette.border;
            c.ring = palette.accent;
            c.caret = palette.accent;
            c.accent = palette.selected;
            c.accent_foreground = palette.foreground;
            c.selection = palette.selected;
            c.primary = palette.accent;
            c.primary_foreground = palette.background;
            c.primary_hover = palette.hover;
            c.primary_active = palette.active;
            c.button_primary = palette.accent;
            c.button_primary_foreground = palette.background;
            c.button_primary_hover = palette.hover;
            c.button_primary_active = palette.active;
            c.button = palette.surface;
            c.button_foreground = palette.foreground;
            c.button_hover = palette.selected;
            c.button_active = palette.border;
            c.secondary = palette.selected;
            c.secondary_foreground = palette.foreground;
            c.secondary_hover = palette.border;
            c.secondary_active = palette.selected;
            c.list = palette.background;
            c.list_active = palette.selected;
            c.list_active_border = palette.accent;
            c.list_hover = palette.surface;
            c.tab_bar = palette.background;
            c.tab_active = palette.surface;
            c.tab_active_foreground = palette.foreground;
            c.tab_foreground = palette.muted;
            c.slider_bar = palette.accent;
            c.slider_thumb = palette.hover;
            c.progress_bar = palette.accent;
            c.success = palette.accent;
            c.success_foreground = palette.background;
        });
        Ok(())
    }
}

struct Palette {
    background: Hsla,
    surface: Hsla,
    selected: Hsla,
    border: Hsla,
    foreground: Hsla,
    muted: Hsla,
    accent: Hsla,
    hover: Hsla,
    active: Hsla,
}

impl Palette {
    fn teal() -> Self {
        Self {
            background: rgb(0x000000).into(),
            surface: rgb(0x081c15).into(),
            selected: rgb(0x1b4332).into(),
            border: rgb(0x2d6a4f).into(),
            foreground: rgb(0xd8f3dc).into(),
            muted: rgb(0x95d5b2).into(),
            accent: rgb(0x74c69d).into(),
            hover: rgb(0xb7e4c7).into(),
            active: rgb(0x52b788).into(),
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
        let foreground = color("foreground")?;
        let accent = color("accent")?;
        Ok(Self {
            background,
            foreground,
            accent,
            surface: color("color0")?,
            selected: color("color8")?,
            border: color("color8")?,
            muted: color("color7")?,
            hover: color("color6")?,
            active: accent,
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
    fn teal_text_and_actions_remain_readable_on_black_and_green() {
        let palette = Palette::teal();
        for background in [palette.background, palette.surface, palette.selected] {
            assert!(contrast(palette.foreground, background) >= 4.5);
            assert!(contrast(palette.muted, background) >= 4.5);
        }
        for fill in [palette.accent, palette.hover, palette.active] {
            assert!(contrast(palette.background, fill) >= 4.5);
        }
        assert!(contrast(palette.accent, palette.background) >= 3.);
    }
}
