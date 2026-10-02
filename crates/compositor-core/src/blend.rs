use serde::{Deserialize, Serialize};

/// Blend modes supported by Compositor, ordered and grouped identically to Photoshop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerBlendMode {
    #[serde(rename = "Normal")]
    Normal,

    // Darken group
    #[serde(rename = "Darken")]
    Darken,
    #[serde(rename = "Multiply")]
    Multiply,
    #[serde(rename = "Color Burn")]
    ColorBurn,
    #[serde(rename = "Linear Burn")]
    LinearBurn,

    // Lighten group
    #[serde(rename = "Lighten")]
    Lighten,
    #[serde(rename = "Screen")]
    Screen,
    #[serde(rename = "Color Dodge")]
    ColorDodge,
    #[serde(rename = "Linear Dodge (Add)")]
    LinearDodge,

    // Contrast group
    #[serde(rename = "Overlay")]
    Overlay,
    #[serde(rename = "Soft Light")]
    SoftLight,
    #[serde(rename = "Hard Light")]
    HardLight,
    #[serde(rename = "Vivid Light")]
    VividLight,
    #[serde(rename = "Linear Light")]
    LinearLight,
    #[serde(rename = "Pin Light")]
    PinLight,
    #[serde(rename = "Hard Mix")]
    HardMix,

    // Comparative group
    #[serde(rename = "Difference")]
    Difference,
    #[serde(rename = "Exclusion")]
    Exclusion,
    #[serde(rename = "Subtract")]
    Subtract,
    #[serde(rename = "Divide")]
    Divide,

    // Component group
    #[serde(rename = "Hue")]
    Hue,
    #[serde(rename = "Saturation")]
    Saturation,
    #[serde(rename = "Color")]
    Color,
    #[serde(rename = "Luminosity")]
    Luminosity,
}

impl Default for LayerBlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl LayerBlendMode {
    /// Photoshop's UI grouping.
    pub fn groups() -> &'static [&'static [LayerBlendMode]] {
        &[
            &[LayerBlendMode::Normal],
            &[
                LayerBlendMode::Darken,
                LayerBlendMode::Multiply,
                LayerBlendMode::ColorBurn,
                LayerBlendMode::LinearBurn,
            ],
            &[
                LayerBlendMode::Lighten,
                LayerBlendMode::Screen,
                LayerBlendMode::ColorDodge,
                LayerBlendMode::LinearDodge,
            ],
            &[
                LayerBlendMode::Overlay,
                LayerBlendMode::SoftLight,
                LayerBlendMode::HardLight,
                LayerBlendMode::VividLight,
                LayerBlendMode::LinearLight,
                LayerBlendMode::PinLight,
                LayerBlendMode::HardMix,
            ],
            &[
                LayerBlendMode::Difference,
                LayerBlendMode::Exclusion,
                LayerBlendMode::Subtract,
                LayerBlendMode::Divide,
            ],
            &[
                LayerBlendMode::Hue,
                LayerBlendMode::Saturation,
                LayerBlendMode::Color,
                LayerBlendMode::Luminosity,
            ],
        ]
    }

    /// User-facing display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Darken => "Darken",
            Self::Multiply => "Multiply",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::ColorDodge => "Color Dodge",
            Self::LinearDodge => "Linear Dodge (Add)",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft Light",
            Self::HardLight => "Hard Light",
            Self::VividLight => "Vivid Light",
            Self::LinearLight => "Linear Light",
            Self::PinLight => "Pin Light",
            Self::HardMix => "Hard Mix",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Subtract => "Subtract",
            Self::Divide => "Divide",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
        }
    }

    /// Blend single color channels (in 0.0 ..= 1.0 range).
    /// `b` is backdrop, `s` is source.
    #[inline]
    pub fn blend_channel(&self, b: f32, s: f32) -> f32 {
        match self {
            Self::Normal => s,
            Self::Darken => b.min(s),
            Self::Multiply => b * s,
            Self::ColorBurn => {
                if s <= 0.0 {
                    0.0
                } else {
                    1.0 - ((1.0 - b) / s).min(1.0)
                }
            }
            Self::LinearBurn => (b + s - 1.0).max(0.0),
            Self::Lighten => b.max(s),
            Self::Screen => b + s - (b * s),
            Self::ColorDodge => {
                if s >= 1.0 {
                    1.0
                } else {
                    (b / (1.0 - s)).min(1.0)
                }
            }
            Self::LinearDodge => (b + s).min(1.0),
            Self::Overlay => {
                if b <= 0.5 {
                    2.0 * b * s
                } else {
                    1.0 - 2.0 * (1.0 - b) * (1.0 - s)
                }
            }
            Self::SoftLight => {
                if s <= 0.5 {
                    b - (1.0 - 2.0 * s) * b * (1.0 - b)
                } else {
                    let d = if b <= 0.25 {
                        ((16.0 * b - 12.0) * b + 4.0) * b
                    } else {
                        b.sqrt()
                    };
                    b + (2.0 * s - 1.0) * (d - b)
                }
            }
            Self::HardLight => {
                if s <= 0.5 {
                    2.0 * b * s
                } else {
                    1.0 - 2.0 * (1.0 - b) * (1.0 - s)
                }
            }
            Self::VividLight => {
                if s <= 0.5 {
                    if s <= 0.0 {
                        0.0
                    } else {
                        1.0 - ((1.0 - b) / (2.0 * s)).min(1.0)
                    }
                } else if s >= 1.0 {
                    1.0
                } else {
                    (b / (2.0 * (1.0 - s))).min(1.0)
                }
            }
            Self::LinearLight => (b + 2.0 * s - 1.0).clamp(0.0, 1.0),
            Self::PinLight => {
                if s <= 0.5 {
                    b.min(2.0 * s)
                } else {
                    b.max(2.0 * (s - 0.5))
                }
            }
            Self::HardMix => {
                let v = if s <= 0.5 {
                    if s <= 0.0 { 0.0 } else { 1.0 - ((1.0 - b) / (2.0 * s)).min(1.0) }
                } else if s >= 1.0 {
                    1.0
                } else {
                    (b / (2.0 * (1.0 - s))).min(1.0)
                };
                if v < 0.5 { 0.0 } else { 1.0 }
            }
            Self::Difference => (b - s).abs(),
            Self::Exclusion => b + s - 2.0 * b * s,
            Self::Subtract => (b - s).max(0.0),
            Self::Divide => {
                if s <= 0.0 {
                    1.0
                } else {
                    (b / s).min(1.0)
                }
            }
            // Non-separable modes require RGB/HSL conversion
            Self::Hue | Self::Saturation | Self::Color | Self::Luminosity => s,
        }
    }
}
