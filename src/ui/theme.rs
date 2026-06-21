use crate::config::settings::ThemeName;
use ratatui::style::{Color, Modifier, Style};

pub const MIZU_DEEP: Color = Color::Rgb(4, 16, 42);
pub const MIZU_ABYSS: Color = Color::Rgb(2, 8, 26);
pub const MIZU_BLUE: Color = Color::Rgb(38, 102, 184);
pub const MIZU_TEAL: Color = Color::Rgb(64, 196, 196);
pub const MIZU_CYAN: Color = Color::Rgb(120, 220, 232);
pub const MIZU_FOAM: Color = Color::Rgb(220, 244, 255);
pub const MIZU_DIM: Color = Color::Rgb(96, 132, 168);
pub const MIZU_RIPPLE: Color = Color::Rgb(28, 72, 140);
pub const MIZU_WAVE: Color = Color::Rgb(16, 52, 104);
pub const MIZU_SURF: Color = Color::Rgb(80, 210, 180);
pub const MIZU_ACCENT: Color = Color::Rgb(160, 240, 200);
pub const MIZU_WARM: Color = Color::Rgb(255, 160, 100);

pub struct Palette {
    pub abyss: Color,
    pub deep: Color,
    pub ripple: Color,
    pub wave: Color,
    pub blue: Color,
    pub surf: Color,
    pub teal: Color,
    pub cyan: Color,
    pub foam: Color,
    pub accent: Color,
    pub warm: Color,
    pub dim: Color,
}

pub fn mix(a: Color, b: Color, t: f64) -> Color {
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    Color::Rgb(
        (ar as f64 + (br as f64 - ar as f64) * t) as u8,
        (ag as f64 + (bg as f64 - ag as f64) * t) as u8,
        (ab as f64 + (bb as f64 - ab as f64) * t) as u8,
    )
}

pub fn rgb(c: Color) -> (u8, u8, u8) {
    if let Color::Rgb(r, g, b) = c {
        (r, g, b)
    } else {
        (255, 255, 255)
    }
}

pub struct MizuTheme {
    pub name: ThemeName,
    pub palette: Palette,
}

impl MizuTheme {
    pub fn from_name(name: ThemeName) -> Self {
        let palette = match name {
            ThemeName::Mizu => Palette {
                abyss: MIZU_ABYSS,
                deep: MIZU_DEEP,
                ripple: MIZU_RIPPLE,
                wave: MIZU_WAVE,
                blue: MIZU_BLUE,
                surf: MIZU_SURF,
                teal: MIZU_TEAL,
                cyan: MIZU_CYAN,
                foam: MIZU_FOAM,
                accent: MIZU_ACCENT,
                warm: MIZU_WARM,
                dim: MIZU_DIM,
            },
            ThemeName::Abyss => Palette {
                abyss: Color::Rgb(1, 3, 12),
                deep: Color::Rgb(2, 6, 20),
                ripple: Color::Rgb(14, 38, 78),
                wave: Color::Rgb(8, 26, 56),
                blue: Color::Rgb(26, 70, 140),
                surf: Color::Rgb(40, 150, 130),
                teal: Color::Rgb(36, 150, 150),
                cyan: Color::Rgb(80, 170, 200),
                foam: Color::Rgb(180, 210, 240),
                accent: Color::Rgb(120, 200, 170),
                warm: Color::Rgb(200, 120, 80),
                dim: Color::Rgb(58, 90, 120),
            },
            ThemeName::Coral => Palette {
                abyss: Color::Rgb(20, 6, 12),
                deep: Color::Rgb(34, 10, 18),
                ripple: Color::Rgb(90, 30, 44),
                wave: Color::Rgb(120, 36, 52),
                blue: Color::Rgb(150, 60, 70),
                surf: Color::Rgb(210, 110, 120),
                teal: Color::Rgb(180, 90, 100),
                cyan: Color::Rgb(240, 160, 160),
                foam: Color::Rgb(255, 224, 220),
                accent: Color::Rgb(255, 200, 140),
                warm: Color::Rgb(255, 130, 90),
                dim: Color::Rgb(140, 80, 92),
            },
        };
        Self { name, palette }
    }

    pub fn gauge_color(&self, ratio: f64) -> Color {
        let t = ratio.clamp(0.0, 1.0);
        if t < 0.5 {
            mix(self.palette.teal, self.palette.cyan, t / 0.5)
        } else {
            mix(self.palette.cyan, self.palette.warm, (t - 0.5) / 0.5)
        }
    }

    pub fn size_color(&self, ratio: f64) -> Style {
        let t = ratio.clamp(0.0, 1.0);
        let color = if t > 0.5 {
            self.palette.warm
        } else if t > 0.1 {
            self.palette.accent
        } else {
            self.palette.surf
        };
        Style::default().fg(color)
    }

    pub fn panel_title(&self) -> Style {
        Style::default()
            .fg(self.palette.surf)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border(&self) -> Style {
        Style::default().fg(self.palette.ripple)
    }

    pub fn panel_bg(&self) -> Style {
        Style::default().bg(self.palette.abyss)
    }

    pub fn label_style(&self) -> Style {
        Style::default()
            .fg(self.palette.foam)
            .add_modifier(Modifier::BOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_themes_build() {
        for name in [ThemeName::Mizu, ThemeName::Abyss, ThemeName::Coral] {
            let t = MizuTheme::from_name(name);
            assert_eq!(t.name, name);
        }
    }

    #[test]
    fn gauge_color_clamps() {
        let t = MizuTheme::from_name(ThemeName::Mizu);
        let _ = t.gauge_color(-5.0);
        let _ = t.gauge_color(5.0);
    }

    #[test]
    fn size_color_thresholds() {
        let t = MizuTheme::from_name(ThemeName::Mizu);
        let small = t.size_color(0.05);
        let med = t.size_color(0.3);
        let big = t.size_color(0.8);
        assert_ne!(small, med);
        assert_ne!(med, big);
        assert_ne!(small, big);
    }

    #[test]
    fn mix_is_symmetric_at_midpoint() {
        let a = Color::Rgb(0, 0, 0);
        let b = Color::Rgb(100, 100, 100);
        assert_eq!(mix(a, b, 0.5), Color::Rgb(50, 50, 50));
    }
}
