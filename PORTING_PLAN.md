# Compositor Windows Porting Plan

## Executive Summary

Compositor is a Photoshop-style image editor originally written for macOS in ~33.7k lines of Swift and ~2.5k lines of C across 223 files. It relies heavily on Apple-proprietary frameworks: SwiftUI, AppKit, Metal, Core Image, Core Graphics, ImageIO, Accelerate (vImage/vDSP), UniformTypeIdentifiers, and Vision (`VNGenerateForegroundInstanceMaskRequest`).

This project is a complete re-platforming to Windows 10 (22H2+) and Windows 11 on x64 and ARM64 architectures. The target architecture separates the system into a high-performance headless core (Rust), a Direct3D 12/Vulkan GPU pipeline (wgpu + WGSL), and a low-latency native shell (Tauri 2 + native child window surface + Windows Ink + React/TypeScript UI).

---

## 1. Platform Dependency Replacement Matrix

| macOS Dependency | Usage in macOS App | Windows Replacement (Exact Libs / Crates) | License |
|---|---|---|---|
| **SwiftUI + AppKit** | 127 files: all UI panels, floating panels, canvas viewport, menus, tabs, drag-and-drop, shortcut manager, inspector | **Tauri 2** (`@tauri-apps/api`, `@tauri-apps/plugin-shell`, `tauri` crate) + **React 18 / TypeScript / Tailwind CSS / Radix UI**. Canvas viewport is hosted as a Win32 child window directly attached to the Tauri window, bypassing webview DOM and IPC. | MIT / Apache-2.0 |
| **Metal (MSL)** | `GPUCanvas`, `MetalWarp`, `MetalBrushCoverage`, `MetalLayerEffects`, `GPUNoise` | **wgpu 24.0** with **Direct3D 12 backend** (primary) and **Vulkan backend** (fallback). MSL shaders ported 1:1 to **WGSL** compute and render pipelines. | MIT / Apache-2.0 |
| **Core Image** | Filter graphs, `CIFilter`, `CIImage` pipeline, `CIColorCube`, gaussian blur, motion blur, separable blend | Custom compute and fragment passes in **wgpu (WGSL)**, matched with CPU reference implementations in Rust (`compositor-pixel`). | MIT / Apache-2.0 |
| **CIRAWFilter** | Camera RAW develop pipeline, cached decode (`RawImporter.swift`, `RawDevelopSheet.swift`) | **`libraw-rs` / `rawloader` / LibRaw C bindings via `cc`** with custom linear develop pipeline matching `CameraRaw.swift`. | CDDL-1.0 / LGPL-2.1 / MIT |
| **Vision Framework** (`VNGenerateForegroundInstanceMaskRequest`) | Select Subject, Object Selection, Remove Background (`SubjectRemoval.swift`, `ObjectSelection.swift`) | **`ort` (ONNX Runtime 1.20+)** using **DirectML Execution Provider** on D3D12 GPU with CPU fallback; model: BiRefNet-general / U2-Net (MIT/Apache-2.0 license). | MIT / Apache-2.0 |
| **CoreGraphics / ImageIO** | PNG, JPEG, TIFF, 2D geometry, image metadata, color conversions | **`image` (0.25)**, **`png`**, **`jpeg-decoder` / `jpeg-encoder`**, **`tiff`**, **`kurbo`** (2D Bézier geometry), **`euclid` / `glam`** (math). | MIT / Apache-2.0 |
| **SVG (CoreGraphics)** | Vector shape rasterization (`ImageImporter.swift`) | **`resvg`** / **`usvg`**. | MIT / Apache-2.0 |
| **HEIC** | iOS photo imports (`ImageImporter.swift`) | **`libheif-rs`** with libde265 / rav1e decoder. | LGPL-3.0 / MIT |
| **CoreText / DirectWrite** | Text layout, font metrics, multiline paragraph editing (`TypeTool.swift`, `PSDText.swift`) | **`cosmic-text`** or **DirectWrite via `windows::Win32::Graphics::DirectWrite`** for exact glyph layout and typography. | MIT / Apache-2.0 |
| **Accelerate (vDSP / vImage)** | Fast box/gaussian blur, lanczos downsample, histograms, SIMD array math | **`fast_image_resize`** (SIMD AVX2/NEON), **`wide` / `std::simd`**, custom SIMD loops. | MIT / Apache-2.0 |
| **CryptoKit** | SHA-256 for project digest and cache keys (`ProjectDigest.swift`) | **`sha2`** crate. | MIT / Apache-2.0 |
| **UniformTypeIdentifiers** | File type identification (`UTType.compositorProject`, `.photoshopImage`, etc.) | Custom MIME / extension registry based on `infer` and Windows file extension associations. | MIT / Apache-2.0 |
| **.comp package & File Watcher** | macOS bundle directory, NSFileCoordinator, FSEvents / live reload | Win32 Directory + **`notify`** crate (`ReadDirectoryChangesW`) + atomic rename (`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`). | MIT / Apache-2.0 |
| **Sparkle / DMG / Notarization** | macOS updater, dmg builder, notarization | **Tauri Updater** / **WinGet Manifest** / **WiX / MSIX packaging** via `cargo-tauri` and `cargo-wix`. | MIT / Apache-2.0 |
| **Portable C Kernels** | `BrushPixels.c`, `HealPixels.c`, `LevelsPixels.c`, `WandPixels.c`, `NoisePixels.c`, `LensPixels.c`, `ContentFill.c`, `AdjustPixels.c`, `DitherPixels.c` | Compiled natively with MSVC/Clang via **`cc` crate** into `compositor-pixel`, wrapped with safe zero-copy Rust APIs. | MIT / Same as repo |

---

## 2. File-by-File Mapping

### 2.1 Core Document Logic (`Compositor/Document/` -> `compositor-core` & `compositor-pixel`)

| Original File | Language | Target Destination | Module / Responsibility |
|---|---|---|---|
| `DocumentLimits.swift` | Swift | `crates/compositor-core/src/limits.rs` | Canvas & document limits (30k maxSide, 200MP surface, RAM-scaled budget) |
| `LayerTransform.swift` | Swift | `crates/compositor-core/src/transform.rs` | `LayerTransform`, `LayerSampling`, snapping, group & corner transforms |
| `LayerAppearance.swift` | Swift | `crates/compositor-core/src/layer/appearance.rs` | Opacity, blend modes (Photoshop order), visibility |
| `LayerGroups.swift` | Swift | `crates/compositor-core/src/layer/group.rs` | Group hierarchy, nesting rules (64 levels max), cycle detection |
| `LayerMask.swift` | Swift | `crates/compositor-core/src/layer/mask.rs` | 8-bit grayscale layer & folder masks, linked/unlinked placements |
| `LiveLayerMask.swift` | Swift | `crates/compositor-core/src/layer/clipping.rs` | Clipping masks (`maskSourceID` directed acyclic graph) |
| `LayerEffects.swift` | Swift | `crates/compositor-core/src/layer/effects.rs` | Stroke, Drop Shadow, Inner Shadow, Color Overlay, Outer/Inner Glow models |
| `LayerAdjustment.swift` | Swift | `crates/compositor-core/src/layer/adjustment.rs` | All 12 adjustment kinds, parameters, validation, ranges |
| `LayerFlip.swift` | Swift | `crates/compositor-core/src/layer/flip.rs` | Horizontal and vertical layer & canvas flipping |
| `LayerMerge.swift` | Swift | `crates/compositor-core/src/layer/merge.rs` | Merge down, merge layers, merge group (Ctrl+E) |
| `DocumentHistory.swift` | Swift | `crates/compositor-core/src/history.rs` | Undo/redo stack, memory trimming, revision IDs |
| `Selection.swift` | Swift | `crates/compositor-core/src/selection/mod.rs` | DocumentSelection (Bézier / polygon path, antialias, feather) |
| `SelectionEdits.swift` | Swift | `crates/compositor-core/src/selection/edits.rs` | Expand, Contract, Feather, Invert, Clear, Fill selection |
| `SelectionClipboard.swift` | Swift | `crates/compositor-core/src/selection/clipboard.rs` | Copy merged, copy layer pixels, paste |
| `FloatingSelection.swift` | Swift | `crates/compositor-core/src/selection/floating.rs` | Lifted pixel drags and floating selections |
| `ColorRangeSelection.swift`| Swift | `crates/compositor-core/src/selection/color_range.rs`| Color range selection using `color_range_mask` |
| `MagicWand.swift` | Swift | `crates/compositor-core/src/selection/wand.rs` | Wand selection invoking `wand_mask` & `wand_trace` |
| `MaskTracing.swift` | Swift | `crates/compositor-core/src/selection/tracing.rs` | Tracing raster mask bounds into vector selection loops |
| `Guides.swift` | Swift | `crates/compositor-core/src/guides.rs` | Horizontal & vertical alignment guides (v8+) |
| `CanvasSize.swift` | Swift | `crates/compositor-core/src/canvas/size.rs` | Canvas size resizing, anchor point offsets |
| `Crop.swift` | Swift | `crates/compositor-core/src/canvas/crop.rs` | Interactive crop, aspect ratio locking, symmetric crop |
| `ImageTrim.swift` | Swift | `crates/compositor-core/src/canvas/trim.rs` | Transparent / top-left color edge trimming |
| `Distort.swift` | Swift | `crates/compositor-core/src/transform/distort.rs` | Free distort quadriateral transformation & perspective warp |
| `BrushStroke.swift` | Swift | `crates/compositor-core/src/brush/stroke.rs` | Stroke geometry, tile replacement tracking, smoothing |
| `ToolDefaults.swift` | Swift | `crates/compositor-core/src/tool_defaults.rs` | Tool default states, persistent tool settings |
| `ProjectWorkspace.swift` | Swift | `crates/compositor-core/src/workspace.rs` | Multi-document tab workspace, active document management |
| `EditorSession.swift` | Swift | `crates/compositor-core/src/session.rs` | Master editor state machine (headless engine API) |
| `EditorSession+Brush.swift`| Swift| `crates/compositor-core/src/session_brush.rs` | Session brush handling and pointer event ingestion |
| `EditorSession+Projects.swift`| Swift| `crates/compositor-core/src/session_projects.rs` | Session document lifecycle (open, save, revert) |
| `Levels.swift` | Swift | `crates/compositor-pixel/src/adjustments/levels.rs` | Levels lookup table and histogram calculation |
| `LevelsAutomatic.swift` | Swift | `crates/compositor-pixel/src/adjustments/levels_auto.rs` | Auto levels calculation from image histogram |
| `Curves.swift` | Swift | `crates/compositor-pixel/src/adjustments/curves.rs` | Cubic spline curves evaluation and LUT generation |
| `HueSaturation.swift` | Swift | `crates/compositor-pixel/src/adjustments/hue_sat.rs` | HSV/HSL color transformations and color cube |
| `ImageAdjustments.swift` | Swift | `crates/compositor-pixel/src/adjustments/mod.rs` | Exposure, Black & White, Color Balance, Invert |
| `PixelAdjust.swift` | Swift | `crates/compositor-pixel/src/adjustments/pixel_adjust.rs` | CPU adjustment dispatch on RGBA buffers |
| `PixelInvert.swift` | Swift | `crates/compositor-pixel/src/adjustments/invert.rs` | Fast SIMD invert on RGBA and mask buffers |
| `AdjustmentEditing.swift`| Swift | `crates/compositor-core/src/adjustment_editing.rs` | Active adjustment parameter editing session |
| `Filters.swift` | Swift | `crates/compositor-pixel/src/filters/mod.rs` | Gaussian blur, motion blur, vignette, bloom |
| `Dither.swift` | Swift | `crates/compositor-pixel/src/filters/dither.rs` | Atkinson, Floyd-Steinberg, Bayer ordered dithering |
| `CameraRaw.swift` | Swift | `crates/compositor-pixel/src/camera_raw/mod.rs` | Camera Raw parameter definitions & pipeline |
| `CameraRawColor.swift` | Swift | `crates/compositor-pixel/src/camera_raw/color.rs` | Color mixer, color grading wheels, tone LUTs |
| `CameraRawDetailOptics.swift`| Swift| `crates/compositor-pixel/src/camera_raw/detail_optics.rs`| Sharpening, noise reduction, defringe, CA |
| `CameraRawGeometryCalibration.swift`| Swift| `crates/compositor-pixel/src/camera_raw/calibration.rs`| Camera calibration and lens profile |
| `ColorPalette.swift` | Swift | `crates/compositor-core/src/color_palette.rs` | Foreground/background swatches, default B/W |
| `CloneStamp.swift` | Swift | `crates/compositor-pixel/src/tools/clone_stamp.rs` | Clone stamp sampling (single layer or all layers) |
| `BlurTool.swift` | Swift | `crates/compositor-pixel/src/tools/blur_tool.rs` | Interactive blur brush for pixels and masks |
| `ContentFill.swift` | Swift | `crates/compositor-pixel/src/tools/content_fill.rs` | Content-aware fill patch synthesis |
| `GuidedMatte.swift` | Swift | `crates/compositor-pixel/src/tools/guided_matte.rs` | Edge-preserving alpha matting filter |
| `ObjectSelection.swift` | Swift | `crates/compositor-ai/src/object_selection.rs` | Bounding box / click to salient object mask |
| `SubjectRemoval.swift` | Swift | `crates/compositor-ai/src/subject_removal.rs` | Select subject & Remove background inference pipeline |
| `ShapeTool.swift` | Swift | `crates/compositor-core/src/tools/shape.rs` | Parametric shapes: rect, rounded rect, ellipse, line |
| `SmudgeLiquify.swift` | Swift | `crates/compositor-pixel/src/tools/liquify.rs` | CPU fallback for smudge and forward warp liquify |
| `TypeTool.swift` | Swift | `crates/compositor-core/src/tools/type_tool.rs` | Inline text models, paragraph box size, run styles |

### 2.2 I/O Subsystem (`Compositor/IO/` -> `compositor-io` & `compositor-core`)

| Original File | Language | Target Destination | Module / Responsibility |
|---|---|---|---|
| `ProjectStore.swift` | Swift | `crates/compositor-io/src/project_store.rs` | Manifest v1–11 serialization, image asset saving/loading |
| `ProjectDigest.swift` | Swift | `crates/compositor-io/src/project_digest.rs` | Package digest and change validation |
| `ProjectWatcher.swift` | Swift | `crates/compositor-io/src/project_watcher.rs` | Directory watcher via `notify` crate for live agent edits |
| `ProjectController.swift` | Swift | `crates/compositor-io/src/project_controller.rs` | Project lifecycle coordinator (open/save/save-as) |
| `ProjectController+ExternalChanges.swift` | Swift | `crates/compositor-io/src/project_controller_reload.rs` | Live reload debouncing and external conflict handling |
| `RecentProjects.swift` | Swift | `crates/compositor-io/src/recent_projects.rs` | Windows Jump List and MRU registry integration |
| `CanvasResizer.swift` | Swift | `crates/compositor-io/src/canvas_resizer.rs` | Coordinate transform on canvas size changes |
| `ImageResizer.swift` | Swift | `crates/compositor-io/src/image_resizer.rs` | High-quality image resampling (`fast_image_resize`) |
| `ImageImporter.swift` | Swift | `crates/compositor-io/src/importers/mod.rs` | PNG, JPEG, TIFF, SVG, HEIC format decoders |
| `ImageExporter.swift` | Swift | `crates/compositor-io/src/exporters/mod.rs` | PNG and JPEG exporters with DPI metadata |
| `ImageFileDrop.swift` | Swift | `crates/compositor-desktop/src/file_drop.rs` | Explorer drag-and-drop ingestion |
| `RawImporter.swift` | Swift | `crates/compositor-io/src/importers/raw.rs` | Camera RAW decoder via LibRaw / rawler |
| `CompositorApplicationDelegate.swift` | Swift | `crates/compositor-desktop/src/main.rs` | Windows process initialization, single instance, args |
| `PSD/PSDTypes.swift` | Swift | `crates/compositor-io/src/psd/types.rs` | PSD document structures, layer records, error types |
| `PSD/PSDReader.swift` | Swift | `crates/compositor-io/src/psd/reader.rs` | Binary 8BPS parser (header, resources, layer section) |
| `PSD/PSDChannelCoder.swift` | Swift | `crates/compositor-io/src/psd/channel_coder.rs` | Raw, RLE (PackBits), ZIP decompression |
| `PSD/PSDVector.swift` | Swift | `crates/compositor-io/src/psd/vector.rs` | Shape path resource parsing (rectangles, ellipses) |
| `PSD/PSDText.swift` | Swift | `crates/compositor-io/src/psd/text.rs` | TySh descriptor parsing for editable text layers |
| `PSD/PSDDocumentBuilder.swift`| Swift| `crates/compositor-io/src/psd/builder.rs` | Converting PSD model to Compositor document format |

### 2.3 Rendering Subsystem (`Compositor/Rendering/` -> `compositor-gpu` & `compositor-pixel`)

| Original File | Language | Target Destination | Module / Responsibility |
|---|---|---|---|
| `AdjustPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/AdjustPixels.c` / `.h` | Compiled via `cc`; safe Rust bindings in `adjust_pixels.rs` |
| `BrushPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/BrushPixels.c` / `.h` | Compiled via `cc`; alpha bounds & un/premultiply |
| `ContentFill.c` / `.h` | C | `crates/compositor-pixel/c_src/ContentFill.c` / `.h` | Compiled via `cc`; content fill synthesis |
| `DitherPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/DitherPixels.c` / `.h` | Compiled via `cc`; dither algorithms |
| `HealPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/HealPixels.c` / `.h` | Compiled via `cc`; spot heal patch matching & membrane fill |
| `LensPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/LensPixels.c` / `.h` | Compiled via `cc`; radial lens distortion |
| `LevelsPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/LevelsPixels.c` / `.h` | Compiled via `cc`; levels tables & histogram |
| `NoisePixels.c` / `.h` | C | `crates/compositor-pixel/c_src/NoisePixels.c` / `.h` | Compiled via `cc`; uniform & gaussian noise |
| `WandPixels.c` / `.h` | C | `crates/compositor-pixel/c_src/WandPixels.c` / `.h` | Compiled via `cc`; wand flood fill, color range, trace loops |
| `GPUCanvas.swift` | Swift | `crates/compositor-gpu/src/canvas.rs` | wgpu canvas manager, texture cache, downsample levels |
| `MetalBrushCoverage.swift` | Swift | `crates/compositor-gpu/src/brush_coverage.rs` | WGSL continuous brush coverage compute pipeline |
| `MetalLayerEffects.swift` | Swift | `crates/compositor-gpu/src/layer_effects.rs` | WGSL stroke, drop shadow, glow, overlay compute passes |
| `MetalWarp.swift` | Swift | `crates/compositor-gpu/src/warp.rs` | WGSL smudge and forward warp compute passes |
| `GPUNoise.swift` | Swift | `crates/compositor-gpu/src/noise.rs` | WGSL noise and film grain compute passes |
| `LayerRenderer.swift` | Swift | `crates/compositor-gpu/src/layer_renderer.rs` | Layer composition pipeline, blending shaders |
| `TiledLayerRenderer.swift`| Swift | `crates/compositor-gpu/src/tiled_renderer.rs` | 256x256 and 1024x1024 tiled replacement manager |
| `DownsampleCache.swift` | Swift | `crates/compositor-gpu/src/downsample_cache.rs` | Multi-level texture mip/halving cache |
| `EffectsPreviewCache.swift`| Swift | `crates/compositor-gpu/src/effects_cache.rs` | Background worker effect rendering cache |
| `RasterSnapshot.swift` | Swift | `crates/compositor-core/src/raster_snapshot.rs` | Immutable tile snapshot, copy-on-write spatial index |
| `SeparableBlend.swift` | Swift | `crates/compositor-pixel/src/blend/separable.rs` | CPU reference separable blend modes |
| `LayerEffectsSurface.swift`| Swift| `crates/compositor-pixel/src/effects/cpu_effects.rs`| CPU fallback for layer effects |
| `AdjustmentSurface.swift` | Swift | `crates/compositor-gpu/src/adjustment_surface.rs` | GPU adjustment pass execution |
| `LiveMaskRenderer.swift` | Swift | `crates/compositor-gpu/src/live_mask.rs` | GPU clipping mask graph evaluator |
| `CanvasViewport.swift` | Swift | `crates/compositor-gpu/src/viewport.rs` | Viewport transform, pan, zoom, fit, 1:1, DPI scaling |
| `EditorCanvas.swift` | Swift | `crates/compositor-desktop/src/canvas_view.rs` | Native Win32 child window embedding wgpu surface |
| `BrushCursorOverlay.swift`| Swift| `crates/compositor-desktop/src/overlays/brush_cursor.rs`| Win32 DPI-aware brush outline cursor |
| `CanvasLinesOverlay.swift`| Swift| `crates/compositor-desktop/src/overlays/canvas_lines.rs`| Guides, layout grid, pixel grid overlay |
| `SampleRingOverlay.swift` | Swift | `crates/compositor-desktop/src/overlays/sample_ring.rs` | Eyedropper sample ring overlay |
| `TransformOverlay.swift` | Swift | `crates/compositor-desktop/src/overlays/transform_overlay.rs`| Transform bounding box and 8 handles |
| `InlineTextEditor.swift` | Swift | `crates/compositor-desktop/src/text_editor.rs` | Win32 / DirectWrite inline text entry overlay |

### 2.4 UI Subsystem (`Compositor/UI/` -> `ui/` React + TS + Tauri Shell)

| Original File | Language | Target Destination | Component Responsibility |
|---|---|---|---|
| `LayersPanel.swift` | Swift | `ui/src/components/layers/LayersPanel.tsx` | Tree view of layers/folders, visibility, lock, clipping |
| `NativeLayerList.swift` | Swift | `ui/src/components/layers/LayerList.tsx` | Drag-reorder, nest, Alt-drag duplicate, inline rename |
| `LayerAppearanceControls.swift`| Swift| `ui/src/components/layers/LayerAppearance.tsx` | Opacity slider, blend mode dropdown |
| `BlendModePicker.swift` | Swift | `ui/src/components/layers/BlendModePicker.tsx` | Full 24 Photoshop blend modes in Photoshop order |
| `LayerMaskMenu.swift` | Swift | `ui/src/components/layers/LayerMaskMenu.tsx` | Add/delete/disable mask, reveal/hide all |
| `EffectsSheet.swift` | Swift | `ui/src/components/dialogs/EffectsDialog.tsx` | Stroke, shadow, glow, overlay inspector dialog |
| `BrushControls.swift` | Swift | `ui/src/components/toolbar/BrushControls.tsx` | Size, hardness, opacity, smoothing, paint/erase toggle |
| `CropControls.swift` | Swift | `ui/src/components/toolbar/CropControls.tsx` | Aspect ratio presets, swap orientation, apply/cancel |
| `GradientControls.swift` | Swift | `ui/src/components/toolbar/GradientControls.tsx` | Gradient type, color stops, reverse |
| `LassoControls.swift` | Swift | `ui/src/components/toolbar/LassoControls.tsx` | Freehand vs Polygonal toggle, feather, antialias |
| `ShapeControls.swift` | Swift | `ui/src/components/toolbar/ShapeControls.tsx` | Rectangle, rounded rect, ellipse, line parameters |
| `TypeControls.swift` | Swift | `ui/src/components/toolbar/TypeControls.tsx` | Font family, size, tracking, line height, alignment |
| `NavigationToolHeader.swift`| Swift| `ui/src/components/toolbar/ToolHeader.tsx` | Dynamic header based on active tool |
| `ToolHeaderStyle.swift` | Swift | `ui/src/components/toolbar/ToolHeader.module.css` | Header layout styling |
| `ColorPickerSheet.swift` | Swift | `ui/src/components/dialogs/ColorPickerDialog.tsx`| Full HSV/RGB/Hex color picker |
| `ColorPaletteControls.swift`| Swift| `ui/src/components/toolbar/ColorPalette.tsx` | Foreground/background color chips, swap, reset |
| `ColorRangeSheet.swift` | Swift | `ui/src/components/dialogs/ColorRangeDialog.tsx`| Select > Color Range dialog with fuzziness slider |
| `CameraRawControls.swift` | Swift | `ui/src/components/panels/CameraRawPanel.tsx` | Master Camera Raw collapsible panel |
| `CameraRawColorControls.swift`| Swift| `ui/src/components/panels/CameraRawColor.tsx` | White balance, exposure, contrast, vibrance |
| `CameraRawDetailOpticsControls.swift`| Swift| `ui/src/components/panels/CameraRawDetail.tsx`| Sharpen, noise reduction, optics correction |
| `CameraRawGeometryCalibrationControls.swift`| Swift| `ui/src/components/panels/CameraRawGeometry.tsx`| Geometry distortion, camera calibration |
| `CameraRawSlider.swift` | Swift | `ui/src/components/controls/ScrubSlider.tsx` | Double-click reset, Option-click, scrubbable slider |
| `CurvesControls.swift` | Swift | `ui/src/components/dialogs/CurvesDialog.tsx` | Spline curve editor with RGB and per-channel curves |
| `LevelsSheet.swift` | Swift | `ui/src/components/dialogs/LevelsDialog.tsx` | Histogram, input/output sliders, Auto button |
| `HueSaturationSheet.swift`| Swift| `ui/src/components/dialogs/HueSaturationDialog.tsx`| Hue, Saturation, Lightness, Colorize checkbox |
| `FilterSheet.swift` | Swift | `ui/src/components/dialogs/FilterDialog.tsx` | Gaussian Blur, Motion Blur, Noise, Vignette, Bloom |
| `CanvasSizeSheet.swift` | Swift | `ui/src/components/dialogs/CanvasSizeDialog.tsx`| Width/height inputs, 3x3 anchor position grid |
| `ImageSizeSheet.swift` | Swift | `ui/src/components/dialogs/ImageSizeDialog.tsx`| Resample image, constrain proportions, DPI input |
| `TrimSheet.swift` | Swift | `ui/src/components/dialogs/TrimDialog.tsx` | Trim based on top-left pixel or transparency |
| `NewCanvasSheet.swift` | Swift | `ui/src/components/dialogs/NewCanvasDialog.tsx`| Preset dimensions, custom size, background color |
| `JPEGExportSheet.swift` | Swift | `ui/src/components/dialogs/JPEGExportDialog.tsx`| Quality slider, live file-size & visual preview |
| `PSDConversionSheet.swift`| Swift| `ui/src/components/dialogs/PSDConversionReport.tsx`| Pre-import conversion warnings dialog |
| `RawDevelopSheet.swift` | Swift | `ui/src/components/dialogs/RawDevelopDialog.tsx`| RAW preview develop step before canvas import |
| `GridSettingsSheet.swift` | Swift | `ui/src/components/dialogs/GridSettingsDialog.tsx`| Layout grid spacing and subdivision count |
| `CanvasRulers.swift` | Swift | `ui/src/components/canvas/Rulers.tsx` | Top and left interactive pixel rulers with guide drag |
| `CanvasThumbnail.swift` | Swift | `ui/src/components/panels/NavigatorPanel.tsx` | Document thumbnail preview and pan viewport box |
| `TransformInspector.swift`| Swift| `ui/src/components/toolbar/TransformFields.tsx`| Numeric X, Y, W, H, Angle, Flip buttons |
| `ProjectTabs.swift` | Swift | `ui/src/components/tabs/ProjectTabs.tsx` | Multi-tab bar, tab close, reorder |
| `ProjectTabLayout.swift`| Swift | `ui/src/components/tabs/TabLayout.tsx` | Tab layout and active tab routing |
| `NumericScrub.swift` | Swift | `ui/src/components/controls/NumericScrub.tsx` | Photoshop-style mouse drag label scrubbing |
| `SliderSnap.swift` | Swift | `ui/src/components/controls/SliderSnap.ts` | Magnetism / snapping logic for slider controls |
| `KeyboardShortcuts.swift` | Swift | `ui/src/shortcuts/KeyboardShortcuts.ts` | Windows keymap (Ctrl replaces Cmd, Alt replaces Opt) |
| `HeldModifiers.swift` | Swift | `ui/src/shortcuts/HeldModifiers.ts` | Shift, Ctrl, Alt tracking for cursor and tool states |
| `FloatingPanel.swift` | Swift | `ui/src/components/floating/FloatingPanel.tsx`| Draggable floating tool panels and inspectors |
| `IndicatorlessScrollView.swift`| Swift| `ui/src/components/controls/ScrollView.tsx` | Clean scrollable container |
| `ProjectWindowBridge.swift`| Swift| `ui/src/bridge/ProjectBridge.ts` | Tauri IPC invoke bindings between UI and Core |

---

## 3. Test Mapping (~45 Test Suites)

Every test in `CompositorTests/` is ported to a Rust unit/integration test in `crates/compositor-core/tests/`, `crates/compositor-pixel/tests/`, or Tauri integration tests:

| Swift Test Suite | Target Rust / TS Test File | Key Validations |
|---|---|---|
| `AdjustmentLayerTests.swift` | `tests/adjustment_layer_tests.rs` | All 12 adjustment layers, mask clipping, serialization |
| `BlendShortcutTests.swift` | `tests/blend_shortcut_tests.rs` | Shift+Plus/Minus shortcut navigation through blend modes |
| `BlurBrushTests.swift` | `tests/blur_brush_tests.rs` | Blur brush tool on image and mask pixels |
| `BrushIntersectionTests.swift`| `tests/brush_intersection_tests.rs`| Continuous brush self-crossing optical density integration |
| `BrushPerformanceTests.swift` | `benches/brush_benchmark.rs` | 4000x4000 800px brush latency < 8ms, 60+ fps |
| `BrushTests.swift` | `tests/brush_tests.rs` | Brush hardness, opacity, smoothing, paint vs erase |
| `CameraRawSliderTests.swift` | `tests/camera_raw_slider_tests.rs`| Slider snapping, double-click reset, range clamps |
| `CameraRawTests.swift` | `tests/camera_raw_tests.rs` | Light, Color, Curves, Mixer, Grading, Optics, Calibration |
| `CanvasEntryTests.swift` | `tests/canvas_entry_tests.rs` | Canvas coordinate transforms and bounds checks |
| `CanvasSizeTests.swift` | `tests/canvas_size_tests.rs` | Canvas expansion, contraction, anchor placements |
| `CanvasThumbnailTests.swift` | `tests/canvas_thumbnail_tests.rs`| High-speed thumbnail generation and updates |
| `CloneStampTests.swift` | `tests/clone_stamp_tests.rs` | Aligned and unaligned sample offsets, single vs all layers |
| `ColorPickerTests.swift` | `tests/color_picker_tests.rs` | RGB <-> HSV round-trip conversions, hex parsing |
| `CompositorTests.swift` | `tests/core_smoke_tests.rs` | Basic document creation, layer addition, deletion |
| `CropTests.swift` | `tests/crop_tests.rs` | Rectangular crop, aspect ratio locking, symmetric crop |
| `CropToCanvasImportTests.swift`| `tests/crop_to_canvas_tests.rs`| Automatic layer cropping when exceeding memory limits |
| `CursorTests.swift` | `tests/cursor_tests.rs` | Brush outline cursor generation and sizing |
| `DistortTests.swift` | `tests/distort_tests.rs` | 4-corner perspective warp, convex quadrilateral check |
| `DitherTests.swift` | `tests/dither_tests.rs` | Atkinson, Floyd-Steinberg, Bayer pattern outputs |
| `DownsampleTests.swift` | `tests/downsample_tests.rs` | Sharp 2x downsampling levels, Lanczos scale |
| `ExportTests.swift` | `tests/export_tests.rs` | PNG/JPEG flattened export, DPI header tags |
| `ExternalChangeTests.swift` | `tests/external_change_tests.rs` | Live reload upon external file write without app restart |
| `FilterTests.swift` | `tests/filter_tests.rs` | Gaussian blur, motion blur, noise, vignette filters |
| `FinishingFilterTests.swift` | `tests/finishing_filter_tests.rs`| Tonal contrast, bloom, glow filters |
| `FloatingPanelTests.swift` | `ui/tests/floating_panel.spec.ts`| Panel dragging, snapping, docking |
| `GPUCanvasTests.swift` | `tests/gpu_canvas_tests.rs` | wgpu pipeline tests, texture lifecycle, draw calls |
| `GradientTests.swift` | `tests/gradient_tests.rs` | Linear and radial gradients, dithering, alpha blending |
| `GroupingSelectionTests.swift`| `tests/grouping_selection_tests.rs`| Selecting group children, group move, collapse |
| `GroupTests.swift` | `tests/group_tests.rs` | Group creation, nesting depth (64 max), opacity inherit |
| `GuideTests.swift` | `tests/guide_tests.rs` | Horizontal/vertical guide placement, snapping, deletion |
| `HistoryTests.swift` | `tests/history_tests.rs` | Undo/redo stack, byte memory budgeting, branching edits |
| `HueSaturationTests.swift` | `tests/hue_saturation_tests.rs`| Color range hue shift, saturation boost, colorize |
| `ImageAdjustmentTests.swift` | `tests/image_adjustment_tests.rs`| Black & White weighting, Color Balance shadows/mid/high |
| `ImageImportTests.swift` | `tests/image_import_tests.rs` | JPEG, PNG, TIFF, SVG, HEIC file loading |
| `ImageSizeTests.swift` | `tests/image_size_tests.rs` | Bicubic and Lanczos image resizing, resolution changes |
| `ImageTrimTests.swift` | `tests/image_trim_tests.rs` | Bounding box trim of transparent or solid pixels |
| `InnerGlowTests.swift` | `tests/inner_glow_tests.rs` | Inner glow effect GPU & CPU reference agreement |
| `JPEGExportTests.swift` | `tests/jpeg_export_tests.rs` | Live preview byte sizing and quality levels |
| `LargeCanvasBrushTests.swift` | `tests/large_canvas_brush.rs` | Memory limits on 100MP canvases during painting |
| `LayerAppearanceTests.swift` | `tests/layer_appearance_tests.rs`| Opacity composition and 24 blend modes |
| `LayerMaskTests.swift` | `tests/layer_mask_tests.rs` | Raster mask painting, invert, feather, link/unlink |
| `LayerTests.swift` | `tests/layer_tests.rs` | Layer reordering, transforms, bounds calculation |
| `LevelsTests.swift` | `tests/levels_tests.rs` | Input black/gamma/white, output levels, auto levels |
| `LiveMaskTests.swift` | `tests/live_mask_tests.rs` | Clipping mask chain evaluation and validation |
| `MagicWandTests.swift` | `tests/magic_wand_tests.rs` | Tolerance flood fill, contiguous vs global |
| `MaskAloneTests.swift` | `tests/mask_alone_tests.rs` | Unlinked mask independent transformation |
| `MaskTransformTests.swift` | `tests/mask_transform_tests.rs` | Linked vs unlinked mask coordinate mapping |
| `MetalWarpTests.swift` | `tests/warp_tests.rs` | Forward warp and smudge GPU compute kernel tests |
| `NativeResolutionPaintTests.swift`| `tests/native_paint_tests.rs`| 1:1 pixel painting regardless of canvas display zoom |
| `OuterGlowTests.swift` | `tests/outer_glow_tests.rs` | Outer glow spread, blur, and color composite |
| `ProjectTabLayoutTests.swift` | `ui/tests/tab_layout.spec.ts` | Multi-tab drag reordering and tab split |
| `ProjectTests.swift` | `tests/project_tests.rs` | Manifest v1–11 round-trip, atomic save, move resilience |
| `ProjectWorkspaceTests.swift` | `tests/workspace_tests.rs` | Multi-document switching and dirty state management |
| `PSBImportTests.swift` | `tests/psb_import_tests.rs` | Large Photoshop document (>30k pixels or >2GB) import |
| `PSDAdjustmentTests.swift` | `tests/psd_adjustment_tests.rs`| Converting PSD adjustment layers to Compositor layers |
| `PSDFixture.swift` | `crates/compositor-io/src/psd/fixture.rs`| Binary PSD/PSB generator for test suite |
| `PSDRoundTripTests.swift` | `tests/psd_round_trip_tests.rs`| Comprehensive PSD import testing against fixtures |
| `PSDVectorFixtures.swift` | `crates/compositor-io/src/psd/vector_fixtures.rs`| Vector shape fixtures for path decoding tests |
| `RasterSnapshotTests.swift` | `tests/raster_snapshot_tests.rs`| Spatial tile indexing, copy-on-write sharing |
| `ResizeSnapTests.swift` | `tests/resize_snap_tests.rs` | Handle dragging snapping to canvas edges and center |
| `SelectionClipboardTests.swift`| `tests/selection_clipboard.rs`| Copy/cut/paste selection, move pixel payload |
| `SelectionEditTests.swift` | `tests/selection_edit_tests.rs` | Expand/contract/feather selection outlines |
| `SelectionFeatherTests.swift` | `tests/selection_feather.rs` | Gaussian falloff edge accuracy |
| `SelectionTests.swift` | `tests/selection_tests.rs` | Marquee, lasso, boolean operations (add/subtract) |
| `ShapeToolTests.swift` | `tests/shape_tool_tests.rs` | Parametric shape rendering, scaling without raster blur |
| `SliderSnapTests.swift` | `ui/tests/slider_snap.spec.ts` | Numeric scrub magnetic snap points |
| `SmartEditTests.swift` | `tests/smart_edit_tests.rs` | Non-destructive transform history preservation |
| `SpotHealingTests.swift` | `tests/spot_healing_tests.rs` | Content-aware, proximity, texture spot healing |
| `TiledLayerTests.swift` | `tests/tiled_layer_tests.rs` | Seam-free tile boundary composition at arbitrary zoom |
| `TitleBarDragTests.swift` | `ui/tests/title_bar.spec.ts` | Custom window chrome dragging on Windows |
| `TransformPressTests.swift` | `tests/transform_press_tests.rs`| Shift-constrain and Alt-center transform modifiers |
| `TransformTests.swift` | `tests/transform_tests.rs` | Non-destructive rotate, scale, flip, bounding box |
| `TypeToolTests.swift` | `tests/type_tool_tests.rs` | Multiline wrap, color runs (v10), font runs (v11) |

---

## 4. Key Risks and Mitigations

1. **Windows Ink & Pointer Latency**:
   - *Risk*: WM_POINTER / Ink message overhead causing brush lag.
   - *Mitigation*: Process `WM_POINTERUPDATE` with `GetPointerPenInfoHistory` to retrieve high-frequency sub-pixel packets directly on the Win32 window thread. Send dabs directly to the GPU command buffer without crossing the webview boundary.
2. **DirectML Performance on Integrated GPUs**:
   - *Risk*: Heavy salient object models exceeding memory or taking >500ms on low-end Intel/AMD iGPUs.
   - *Mitigation*: Select quantized FP16 models (e.g., BiRefNet-general-tiny or U2-Net-p), optimize graph execution in DirectML, and fall back to multi-threaded CPU SIMD if D3D12 device lacks tensor acceleration.
3. **Color Space & Blend Parity**:
   - *Risk*: Windows D3D12 linear vs sRGB blending discrepancies compared to macOS sRGB blending.
   - *Mitigation*: Implement custom sRGB blend formulas directly in WGSL fragment/compute shaders instead of relying on hardware blend state, matching the Adobe/Compositor sRGB math exactly.
4. **.comp File Format Cross-Platform Interop**:
   - *Risk*: Path separators, JSON floating-point formatting, or endianness discrepancies breaking round-trip compatibility.
   - *Mitigation*: Strict JSON serialization matching Swift's `JSONEncoder` (`sortedKeys`, pretty-printed, forward slashes in asset paths). Rigorous round-trip tests against macOS-generated manifests.

---

## 5. Phase Plan and Gates

- **Phase 0: Spikes and Scaffolding** (GATE: 3 spikes meet latency & fps budgets; workspace compiles clean).
- **Phase 1: Headless Core** (GATE: All document, history, selection, limits, and .comp v1–11 round-trip tests pass).
- **Phase 2: Pixels** (GATE: All C kernels ported/wrapped; blend modes, adjustments, dither, heal, wand match golden images).
- **Phase 3: GPU Renderer** (GATE: wgpu tiled renderer, effects, warp, noise pass parity against CPU reference within 1/255 delta).
- **Phase 4: I/O** (GATE: Image importers/exporters, RAW pipeline, and PSD/PSB round-trip tests green).
- **Phase 5: Windows UI** (GATE: Tauri 2 + React UI matches macOS layouts and passes end-to-end workflow tests).
- **Phase 6: Platform Integration & Release** (GATE: Per-monitor DPI, Ink, file associations, MSIX installer, clean install/uninstall on Windows 10/11 x64 and ARM64).
- **Phase 7: Hardening & Parity Verification** (GATE: Fuzzing passes, soak tests green, `PARITY_REPORT.md` shows 100% verified features).
