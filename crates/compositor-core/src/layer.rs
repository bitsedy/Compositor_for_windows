use serde::{Deserialize, Serialize};
use uuid::Uuid;
use glam::Vec2;

use crate::blend::LayerBlendMode;
use crate::transform::LayerTransform;
use crate::selection::DocumentSelection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub Uuid);

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Color representation matching Apple's PaletteColor (sRGB channels in 0.0..=1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaletteColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    #[serde(default = "default_alpha")]
    pub alpha: f32,
}

fn default_alpha() -> f32 {
    1.0
}

impl PaletteColor {
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red: red.clamp(0.0, 1.0),
            green: green.clamp(0.0, 1.0),
            blue: blue.clamp(0.0, 1.0),
            alpha: 1.0,
        }
    }

    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: red.clamp(0.0, 1.0),
            green: green.clamp(0.0, 1.0),
            blue: blue.clamp(0.0, 1.0),
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }
}

// Layer Effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeEffect {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_stroke_size")]
    pub size: f32,
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
    #[serde(default = "default_alpha")]
    pub opacity: f32,
    #[serde(default)]
    pub inside: bool,
}

fn default_true() -> bool { true }
fn default_stroke_size() -> f32 { 4.0 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowEffect {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_shadow_angle")]
    pub angle: f32,
    #[serde(default = "default_shadow_distance")]
    pub distance: f32,
    #[serde(default = "default_shadow_blur")]
    pub blur: f32,
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
    #[serde(default = "default_shadow_opacity")]
    pub opacity: f32,
}

fn default_shadow_angle() -> f32 { 90.0 }
fn default_shadow_distance() -> f32 { 20.0 }
fn default_shadow_blur() -> f32 { 20.0 }
fn default_shadow_opacity() -> f32 { 0.5 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorOverlayEffect {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
    #[serde(default = "default_alpha")]
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerShadowEffect {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_shadow_angle")]
    pub angle: f32,
    #[serde(default = "default_inner_shadow_dist")]
    pub distance: f32,
    #[serde(default = "default_inner_shadow_blur")]
    pub blur: f32,
    #[serde(default)]
    pub red: f32,
    #[serde(default)]
    pub green: f32,
    #[serde(default)]
    pub blue: f32,
    #[serde(default = "default_shadow_opacity")]
    pub opacity: f32,
}

fn default_inner_shadow_dist() -> f32 { 10.0 }
fn default_inner_shadow_blur() -> f32 { 10.0 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OuterGlowEffect {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_shadow_distance")]
    pub size: f32,
    #[serde(default = "default_alpha")]
    pub red: f32,
    #[serde(default = "default_alpha")]
    pub green: f32,
    #[serde(default = "default_alpha")]
    pub blue: f32,
    #[serde(default = "default_glow_opacity")]
    pub opacity: f32,
}

fn default_glow_opacity() -> f32 { 0.75 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerGlowEffect {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_inner_shadow_dist")]
    pub size: f32,
    #[serde(default = "default_alpha")]
    pub red: f32,
    #[serde(default = "default_alpha")]
    pub green: f32,
    #[serde(default = "default_alpha")]
    pub blue: f32,
    #[serde(default = "default_glow_opacity")]
    pub opacity: f32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffects {
    pub stroke: Option<StrokeEffect>,
    pub shadow: Option<ShadowEffect>,
    pub color_overlay: Option<ColorOverlayEffect>,
    pub inner_shadow: Option<InnerShadowEffect>,
    pub outer_glow: Option<OuterGlowEffect>,
    pub inner_glow: Option<InnerGlowEffect>,
}

impl LayerEffects {
    pub fn is_empty(&self) -> bool {
        self.stroke.is_none()
            && self.shadow.is_none()
            && self.color_overlay.is_none()
            && self.inner_shadow.is_none()
            && self.outer_glow.is_none()
            && self.inner_glow.is_none()
    }
}

// Layer Mask
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerMask {
    pub is_enabled: bool,
    pub is_linked: bool,
    pub transform: LayerTransform,
    /// Image asset identifier or filename within .comp package (e.g. "mask.png")
    pub asset_filename: Option<String>,
}

// Shape tool metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerShape {
    pub kind: String, // "rectangle", "ellipse"
    pub fill_color: Option<PaletteColor>,
    pub stroke_color: Option<PaletteColor>,
    pub stroke_width: f32,
}

// Text layer metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerText {
    pub text: String,
    pub font_name: String,
    pub font_size: f32,
    pub color: PaletteColor,
    pub is_multiline: bool,
    pub bounds_width: Option<f32>,
    pub bounds_height: Option<f32>,
}

// Adjustments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentKind {
    #[serde(rename = "Hue/Saturation")]
    HueSaturation,
    #[serde(rename = "Levels")]
    Levels,
    #[serde(rename = "Curves")]
    Curves,
    #[serde(rename = "Exposure")]
    Exposure,
    #[serde(rename = "Gradient Map")]
    GradientMap,
    #[serde(rename = "Grain")]
    Grain,
    #[serde(rename = "Add Noise")]
    AddNoise,
    #[serde(rename = "Gaussian Blur")]
    GaussianBlur,
    #[serde(rename = "Motion Blur")]
    MotionBlur,
    #[serde(rename = "Invert")]
    Invert,
    #[serde(rename = "Black & White")]
    BlackWhite,
    #[serde(rename = "Color Balance")]
    ColorBalance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerAdjustment {
    pub kind: AdjustmentKind,
    #[serde(default)]
    pub hue: f32,
    #[serde(default)]
    pub saturation: f32,
    #[serde(default)]
    pub lightness: f32,
    #[serde(default)]
    pub colorize: bool,
}

/// An image layer or group in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageLayer {
    pub id: LayerId,
    pub name: String,
    pub is_visible: bool,
    pub transform: LayerTransform,
    pub parent_id: Option<LayerId>,
    pub is_group: bool,
    pub opacity: f32,
    pub blend_mode: LayerBlendMode,
    pub mask_source_id: Option<LayerId>,
    pub mask: Option<LayerMask>,
    pub adjustment: Option<LayerAdjustment>,
    pub shape: Option<LayerShape>,
    pub effects: Option<LayerEffects>,
    pub text: Option<LayerText>,
    /// Associated image raster file inside the .comp folder (e.g. "layer_0.png")
    pub asset_filename: Option<String>,
}

impl ImageLayer {
    pub fn new_empty(name: impl Into<String>, size: Vec2) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            is_visible: true,
            transform: LayerTransform::new(0.0, 0.0, size.x as f64, size.y as f64),
            parent_id: None,
            is_group: false,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            mask_source_id: None,
            mask: None,
            adjustment: None,
            shape: None,
            effects: None,
            text: None,
            asset_filename: None,
        }
    }

    pub fn new_group(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            is_visible: true,
            transform: LayerTransform::new(0.0, 0.0, 0.0, 0.0),
            parent_id: None,
            is_group: true,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            mask_source_id: None,
            mask: None,
            adjustment: None,
            shape: None,
            effects: None,
            text: None,
            asset_filename: None,
        }
    }
}

/// A guide line placed on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GuideOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasGuide {
    pub id: Uuid,
    pub orientation: GuideOrientation,
    pub position: f32,
}

impl CanvasGuide {
    pub fn new(orientation: GuideOrientation, position: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            orientation,
            position,
        }
    }
}

/// A complete Compositor document canvas state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasDocument {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub resolution: f64,
    /// Layers from bottom to top (rendering order).
    pub layers: Vec<ImageLayer>,
    pub guides: Vec<CanvasGuide>,
    #[serde(skip)]
    pub selection: Option<DocumentSelection>,
}

impl CanvasDocument {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            width,
            height,
            resolution: 72.0,
            layers: Vec::new(),
            guides: Vec::new(),
            selection: None,
        }
    }

    pub fn size_vec2(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Finds a layer index by LayerId.
    pub fn layer_index(&self, id: LayerId) -> Option<usize> {
        self.layers.iter().position(|l| l.id == id)
    }

    /// Finds a layer by LayerId.
    pub fn layer(&self, id: LayerId) -> Option<&ImageLayer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Finds a mutable layer by LayerId.
    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut ImageLayer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
}
