use compositor_core::LayerTransform;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_PROJECT_VERSION: u32 = 11;
pub const SUPPORTED_VERSIONS: std::ops::RangeInclusive<u32> = 1..=CURRENT_PROJECT_VERSION;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasGuide {
    pub id: Uuid,
    pub axis: String, // "horizontal" or "vertical"
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLayerRecord {
    pub id: Uuid,
    pub name: String,
    pub is_visible: bool,
    pub transform: LayerTransform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_file: Option<String>,
    #[serde(alias = "parentId", rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_group: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_enabled: Option<bool>,
    #[serde(alias = "maskSourceId", rename = "maskSourceID", skip_serializing_if = "Option::is_none")]
    pub mask_source_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_placement: Option<LayerTransform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_linked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_color_space")]
    pub color_space: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<f64>,
    #[serde(alias = "documentId", rename = "documentID")]
    pub document_id: Uuid,
    pub width: usize,
    pub height: usize,
    #[serde(alias = "activeLayerId", rename = "activeLayerID", skip_serializing_if = "Option::is_none")]
    pub active_layer_id: Option<Uuid>,
    pub layers: Vec<ProjectLayerRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guides: Option<Vec<CanvasGuide>>,
}

fn default_format() -> String {
    "com.compositor.project".to_string()
}

fn default_version() -> u32 {
    CURRENT_PROJECT_VERSION
}

fn default_color_space() -> String {
    "sRGB".to_string()
}
