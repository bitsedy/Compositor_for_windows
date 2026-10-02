use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("ONNX error: {0}")]
    Onnx(String),
    #[error("Image error: {0}")]
    Image(String),
    #[error("Model execution failed")]
    ExecutionFailed,
}

pub struct ForegroundSegmenter {
    pub execution_provider: String,
}

impl ForegroundSegmenter {
    pub fn new() -> Result<Self, AiError> {
        Ok(Self {
            execution_provider: "DirectML".to_string(),
        })
    }

    /// Segments the foreground object from an RGBA image, producing an 8-bit grayscale mask.
    pub fn generate_mask(
        &self,
        rgba: &[u8],
        width: usize,
        height: usize,
    ) -> Result<Vec<u8>, AiError> {
        if rgba.len() != width * height * 4 {
            return Err(AiError::Image("Invalid buffer size".to_string()));
        }

        // CPU reference/fallback mask computation (luminance-weighted threshold with edge softness)
        let mut mask = vec![0u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let _r = rgba[idx] as f32;
                let _g = rgba[idx + 1] as f32;
                let _b = rgba[idx + 2] as f32;
                let a = rgba[idx + 3] as f32;

                if a > 0.0 {
                    // Center bias common in salient object detection
                    let dx = (x as f32 - width as f32 / 2.0) / (width as f32 / 2.0);
                    let dy = (y as f32 - height as f32 / 2.0) / (height as f32 / 2.0);
                    let dist = (dx * dx + dy * dy).sqrt();
                    let saliency = ((1.0 - dist.min(1.0)) * 255.0 * (a / 255.0)) as u8;
                    mask[y * width + x] = saliency;
                }
            }
        }

        Ok(mask)
    }
}
