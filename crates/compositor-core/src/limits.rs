use std::cmp::{max, min};

/// The size and memory ceilings a document is held to, in one place.
///
/// Ported 1:1 from `DocumentLimits.swift`.
pub struct DocumentLimits;

impl DocumentLimits {
    /// Longest side, in pixels, of any canvas, layer, mask or generated surface.
    pub const MAX_SIDE: usize = 30_000;
    pub const MAX_SIDE_EXTENT: f64 = Self::MAX_SIDE as f64;

    /// Largest single surface: a canvas, an export, a filter target, an adjustment or mask render.
    /// At RGBA8 one allocation is at most 800 MB, and a filter holds a few of them at once.
    pub const MAX_SURFACE_PIXELS: usize = 200_000_000;
    pub const MAX_SURFACE_EXTENT: f64 = Self::MAX_SURFACE_PIXELS as f64;

    /// Total imported raster one document may hold, summed across every layer and mask.
    /// Scaled to the machine: a quarter of its memory at 4 bytes a pixel, never less than
    /// one surface and never more than 800 MP (3.2 GB of layers).
    pub fn document_pixel_budget() -> usize {
        let physical_mem = Self::get_physical_memory();
        let scaled = physical_mem / 16;
        min(800_000_000, max(Self::MAX_SURFACE_PIXELS, scaled as usize))
    }

    pub fn max_surface_megapixels() -> usize {
        Self::MAX_SURFACE_PIXELS / 1_000_000
    }

    pub fn document_budget_megapixels() -> usize {
        Self::document_pixel_budget() / 1_000_000
    }

    #[cfg(windows)]
    fn get_physical_memory() -> u64 {
        use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        unsafe {
            if GlobalMemoryStatusEx(&mut status).is_ok() {
                status.ullTotalPhys
            } else {
                16 * 1024 * 1024 * 1024 // Default fallback: 16 GB
            }
        }
    }

    #[cfg(not(windows))]
    fn get_physical_memory() -> u64 {
        16 * 1024 * 1024 * 1024 // Default fallback: 16 GB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limits() {
        assert_eq!(DocumentLimits::MAX_SIDE, 30_000);
        assert_eq!(DocumentLimits::MAX_SURFACE_PIXELS, 200_000_000);
        assert_eq!(DocumentLimits::max_surface_megapixels(), 200);
        let budget = DocumentLimits::document_pixel_budget();
        assert!(budget >= DocumentLimits::MAX_SURFACE_PIXELS);
        assert!(budget <= 800_000_000);
    }
}
