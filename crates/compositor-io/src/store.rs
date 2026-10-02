use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use image::ImageFormat;
use uuid::Uuid;
use thiserror::Error;

use compositor_core::DocumentLimits;
use crate::manifest::{ProjectManifest, SUPPORTED_VERSIONS};

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("This is not a valid Compositor project, or its metadata is damaged.")]
    Invalid,
    #[error("This project uses format version {0}. This app supports versions 1–11.")]
    Version(u32),
    #[error("An image inside the project is missing or damaged. The current document has not been replaced.")]
    MissingImage,
    #[error("This project exceeds the supported canvas, layer, file-size, or 200-megapixel document limit.")]
    TooLarge,
    #[error("An image could not be saved. The previous project has not been replaced.")]
    Encode,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Represents an imported or decoded in-memory image layer asset.
#[derive(Clone, Debug)]
pub struct LoadedImageAsset {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// An in-memory loaded .comp project.
#[derive(Debug)]
pub struct ProjectSnapshot {
    pub manifest: ProjectManifest,
    pub images: HashMap<Uuid, LoadedImageAsset>,
    pub masks: HashMap<Uuid, LoadedImageAsset>,
}

#[derive(Debug, serde::Deserialize)]
struct ManifestHeader {
    pub format: String,
    pub version: u32,
}

pub struct ProjectStore;

impl ProjectStore {
    /// Loads a `.comp` project directory.
    pub fn load(folder_path: &Path) -> Result<ProjectSnapshot, ProjectError> {
        let manifest_path = folder_path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(ProjectError::Invalid);
        }

        let mut file = File::open(&manifest_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let header: ManifestHeader = serde_json::from_str(&content).map_err(|_| ProjectError::Invalid)?;

        if header.format != "com.compositor.project" {
            return Err(ProjectError::Invalid);
        }

        if !SUPPORTED_VERSIONS.contains(&header.version) {
            return Err(ProjectError::Version(header.version));
        }

        let manifest: ProjectManifest = serde_json::from_str(&content).map_err(|_| ProjectError::Invalid)?;

        if manifest.color_space != "sRGB" {
            return Err(ProjectError::Invalid);
        }

        if manifest.width == 0
            || manifest.height == 0
            || manifest.width > DocumentLimits::MAX_SIDE
            || manifest.height > DocumentLimits::MAX_SIDE
            || (manifest.width * manifest.height) > DocumentLimits::MAX_SURFACE_PIXELS
            || manifest.layers.len() > 10_000
        {
            return Err(ProjectError::TooLarge);
        }

        let mut images = HashMap::new();
        let mut masks = HashMap::new();

        for layer in &manifest.layers {
            if let Some(ref img_file) = layer.image_file {
                let img_in_images = folder_path.join("images").join(img_file);
                let img_in_root = folder_path.join(img_file);
                let img_path = if img_in_images.exists() {
                    img_in_images
                } else if img_in_root.exists() {
                    img_in_root
                } else {
                    return Err(ProjectError::MissingImage);
                };

                let img = image::open(&img_path).map_err(|_| ProjectError::MissingImage)?.to_rgba8();
                images.insert(
                    layer.id,
                    LoadedImageAsset {
                        width: img.width(),
                        height: img.height(),
                        rgba: img.into_raw(),
                    },
                );
            }

            if let Some(ref mask_file) = layer.mask_file {
                let mask_in_images = folder_path.join("images").join(mask_file);
                let mask_in_root = folder_path.join(mask_file);
                let mask_path = if mask_in_images.exists() {
                    mask_in_images
                } else if mask_in_root.exists() {
                    mask_in_root
                } else {
                    return Err(ProjectError::MissingImage);
                };

                let mask_img = image::open(&mask_path).map_err(|_| ProjectError::MissingImage)?.to_rgba8();
                masks.insert(
                    layer.id,
                    LoadedImageAsset {
                        width: mask_img.width(),
                        height: mask_img.height(),
                        rgba: mask_img.into_raw(),
                    },
                );
            }
        }

        Ok(ProjectSnapshot {
            manifest,
            images,
            masks,
        })
    }

    /// Saves a `.comp` project snapshot to a directory.
    pub fn save(snapshot: &ProjectSnapshot, folder_path: &Path) -> Result<(), ProjectError> {
        if snapshot.manifest.width == 0
            || snapshot.manifest.height == 0
            || snapshot.manifest.width > DocumentLimits::MAX_SIDE
            || snapshot.manifest.height > DocumentLimits::MAX_SIDE
            || (snapshot.manifest.width * snapshot.manifest.height) > DocumentLimits::MAX_SURFACE_PIXELS
            || snapshot.manifest.layers.len() > 10_000
        {
            return Err(ProjectError::TooLarge);
        }

        if !folder_path.exists() {
            fs::create_dir_all(folder_path)?;
        }

        let images_dir = folder_path.join("images");
        if !images_dir.exists() {
            fs::create_dir_all(&images_dir)?;
        }

        // Write images
        for (layer_id, asset) in &snapshot.images {
            let filename = format!("{}.png", layer_id);
            let out_path = images_dir.join(&filename);
            let img = image::RgbaImage::from_raw(asset.width, asset.height, asset.rgba.clone())
                .ok_or(ProjectError::Encode)?;
            img.save_with_format(&out_path, ImageFormat::Png).map_err(|_| ProjectError::Encode)?;
        }

        // Write masks
        for (layer_id, asset) in &snapshot.masks {
            let filename = format!("{}.mask.png", layer_id);
            let out_path = images_dir.join(&filename);
            let img = image::RgbaImage::from_raw(asset.width, asset.height, asset.rgba.clone())
                .ok_or(ProjectError::Encode)?;
            img.save_with_format(&out_path, ImageFormat::Png).map_err(|_| ProjectError::Encode)?;
        }

        // Write manifest
        let manifest_json = serde_json::to_string_pretty(&snapshot.manifest)?;
        if manifest_json.len() > 4 * 1024 * 1024 {
            return Err(ProjectError::TooLarge);
        }
        let manifest_path = folder_path.join("manifest.json");
        let mut file = File::create(&manifest_path)?;
        file.write_all(manifest_json.as_bytes())?;

        Ok(())
    }
}
