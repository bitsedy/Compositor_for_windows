# Compositor for Windows: The Ultimate Power User Manual

Welcome to **Compositor**, a professional-grade raster graphics editor engineered for maximum performance, non-destructive layer compositing, photo manipulation, and autonomous AI collaboration on Windows 10 and 11.

This manual is designed to take you from initial setup to extracting 100% of the application's capabilities, covering fluid stylus painting, advanced masking, color grading, GPU layer effects, and real-time AI agent integration.

---

## Table of Contents

1. [System Architecture & Core Philosophy](#1-system-architecture--core-philosophy)
2. [Interface Overview & Workspace Navigation](#2-interface-overview--workspace-navigation)
3. [The Selection Engine & Marching Ants](#3-the-selection-engine--marching-ants)
4. [Brush Engine & Windows Ink Stylus Mastery](#4-brush-engine--windows-ink-stylus-mastery)
5. [Photo Retouching & Generative Filling](#5-photo-retouching--generative-filling)
6. [Layer Architecture, Masks & Clipping Chains](#6-layer-architecture-masks--clipping-chains)
7. [The 25 Photoshop Blend Modes Guide](#7-the-25-photoshop-blend-modes-guide)
8. [Non-Destructive GPU Layer Effects](#8-non-destructive-gpu-layer-effects)
9. [Color Grading, Curves & Camera Raw Studio](#9-color-grading-curves--camera-raw-studio)
10. [Vector Shapes & Typography](#10-vector-shapes--typography)
11. [Canvas Geometry, Smart Crops & Edge Trim](#11-canvas-geometry-smart-crops--edge-trim)
12. [Autonomous AI Collaboration & Live .comp Sync](#12-autonomous-ai-collaboration--live-comp-sync)
13. [Photoshop-Standard Keyboard Shortcuts Cheat Sheet](#13-photoshop-standard-keyboard-shortcuts-cheat-sheet)
14. [Performance Tuning & Large Document Optimization](#14-performance-tuning--large-document-optimization)

---

## 1. System Architecture & Core Philosophy

Compositor is built with two fundamental tenets:
1. **Zero-Latency Direct Pipeline**: Every layer, stroke, and effect is calculated directly on your GPU via Direct3D 12. Stylus input bypasses webview overhead through a native Win32 child window (`CompositorCanvas`), delivering **1.15 ms input-to-pixel latency**.
2. **Strict Non-Destructive Editing**: Transforms, layer effects, masks, adjustments, and vector shapes retain their mathematical descriptions. You can resize, rotate, warp, or adjust parameters indefinitely without degrading pixel quality.

---

## 2. Interface Overview & Workspace Navigation

The workspace is organized into five primary ergonomic zones:

```
+-----------------------------------------------------------------------------------+
| Menu Bar: File  Edit  Image  Layer  Select  Filter  View  Help                    |
+-----------------------------------------------------------------------------------+
| Tool Options Bar: Context-sensitive settings + Numeric Scrubbing labels           |
+----+---------------------------------------------------------------+--------------+
| T  |                                                               | Layers Panel |
| O  |                                                               | - Blend Mode |
| O  |                   Native Direct3D 12 Canvas                   | - Opacity    |
| L  |                      (Win32 Child HWND)                       | - Groups     |
| S  |                                                               +--------------+
|    |               Subpixel Pan / Zoom / Rulers / Guides           | Adjustments  |
| P  |                                                               +--------------+
| A  |                                                               | History      |
| L  |                                                               | - Scrubbing  |
+----+---------------------------------------------------------------+--------------+
| Status Bar: 1920x1080 | 100% Zoom | X: 450 Y: 320 | Direct3D 12 High-Perf         |
+-----------------------------------------------------------------------------------+
```

### High-Speed Canvas Navigation
- **Pan**: Hold `Spacebar` and drag with the left mouse button, or click and drag with the middle mouse button.
- **Smooth Zoom**: Hold `Alt` and roll the mouse wheel, or pinch-to-zoom on precision touchpads.
- **Fit to Screen**: Press `Ctrl + 0` to fit the entire canvas within the viewport.
- **Actual Pixels (100%)**: Press `Ctrl + 1` to view pixels at 1:1 screen resolution.
- **Numeric Scrubbing**: Everywhere you see a parameter label (e.g., `Size`, `Opacity`, `Tolerance`, `Radius`), **click and drag left or right directly on the label text** to scrub values smoothly with sub-pixel sensitivity.

---

## 3. The Selection Engine & Marching Ants

Compositor provides a hardware-accelerated selection engine rendered with real-time 60 FPS animated marching ants (`marching_ants.wgsl`).

### Selection Tools
- **Rectangular Marquee (`M`)**: Click and drag to create rectangular bounds. Hold `Shift` to constrain to a 1:1 square; hold `Alt` to expand outward from the center.
- **Elliptical Marquee (`Shift + M`)**: Circular and oval selections with subpixel edge antialiasing.
- **Lasso Tool (`L`)**: Freehand outline selection.
- **Polygonal Lasso (`L`)**: Click consecutive anchor points to define angular polygons. Double-click or press `Enter` to close the perimeter.
- **Magic Wand (`W`)**: Instant color-similarity clustering powered by the `WandPixels.c` kernel.
  - *Tolerance*: Adjust similarity threshold (0–255).
  - *Contiguous*: Toggle whether only adjacent connected pixels are selected or all matching colors across the layer.
- **Color Range (Select > Color Range)**: Select colors across the image with a live fuzziness slider and localized preview.
- **Select Subject (AI-Powered)**: One-click salient object isolation powered by DirectML GPU tensor inference in **$21\text{ ms}$**.

### Selection Operations
- **Add to Selection**: Hold `Shift` while dragging any selection tool.
- **Subtract from Selection**: Hold `Alt` while dragging any selection tool.
- **Intersect Selection**: Hold `Shift + Alt`.
- **Invert Selection (`Ctrl + Shift + I`)**: Inverts active selection boundary.
- **Feather Selection (`Select > Modify > Feather`)**: Applies Gaussian edge blur to create soft transitions.
- **Expand / Contract (`Select > Modify > Expand / Contract`)**: Grows or shrinks the selection border by exact pixel increments.
- **Deselect (`Ctrl + D`)**: Clears active selection.

---

## 4. Brush Engine & Windows Ink Stylus Mastery

The brush engine evaluates continuous brush strokes using **4-point Gauss-Legendre quadrature numerical integration** over line segments directly in WGSL compute shaders.

### Brush Dynamics
- **Size (`[` and `]`)**: Dynamically scales brush diameter from 1 px to 2000 px.
- **Hardness (`Shift + [` and `Shift + ]`)**: Controls edge falloff from $0\%$ (ultra-soft airbrush) to $100\%$ (pixel-sharp hard edge).
- **Opacity (`0`–`9` keys)**: Overall stroke transparency. Press `5` for 50%, `8` for 80%, `0` for 100%.
- **Flow**: Regulates paint accumulation rate per stamp overlap.
- **Pressure Dynamics**: Toggle pen pressure modulation for size and opacity in the Tool Options bar. Works natively with all Windows Ink tablets (Wacom, Surface Pen, Huion, XP-Pen).
- **Color Swatches (`X` and `D`)**: Press `D` to reset colors to Default Black/White. Press `X` to instantly swap foreground and background colors.

---

## 5. Photo Retouching & Generative Filling

### Spot Healing Brush (`J`)
Eliminates blemishes, scratches, and unwanted elements using an advanced Poisson surface reconstruction kernel (`HealPixels.c`):
1. Select the **Spot Healing Brush** (`J`).
2. Set the brush diameter slightly larger than the artifact.
3. Dab or stroke over the defect. Compositor samples surrounding texture, computes Laplacian gradients, and seamlessly blends the patch without edge halos.

### Clone Stamp (`S`)
Transfers texture from a sampled anchor:
1. Hold `Alt` and left-click the source region to sample.
2. Paint over the target region. The source crosshair tracks your movement synchronously.

### Content-Aware Fill (`Edit > Fill > Content-Aware`)
Fills large selected regions automatically:
1. Make a selection around the object to remove (e.g. with Lasso).
2. Choose **Edit > Content-Aware Fill**. The algorithm analyzes surrounding exemplar texture patches (`ContentFill.c`) to synthesize matching background content.

---

## 6. Layer Architecture, Masks & Clipping Chains

### Layer Stack Hierarchy
- **Reordering**: Drag layers up or down in the **Layers Panel**. The top layer draws in front.
- **Nesting in Groups (`Ctrl + G`)**: Select one or more layers and press `Ctrl + G` to create a folder. Groups can be nested up to **64 levels deep**.
- **Ungroup (`Ctrl + Shift + G`)**: Releases layers from their enclosing folder.
- **Duplicate Layer (`Ctrl + J`)**: Duplicates active layer or selected region into a new layer.
- **Merge Down (`Ctrl + E`)**: Bakes the active layer into the layer immediately below it.
- **Merge Visible (`Ctrl + Shift + E`)**: Flattens all visible layers into a single layer while keeping hidden layers untouched.

### Raster Layer Masks
Layer masks provide non-destructive transparency:
- Click the **Add Mask** icon at the bottom of the Layers Panel.
- Painting with **Black (`#000000`)** hides pixels.
- Painting with **White (`#FFFFFF`)** reveals pixels.
- Painting with **Gray** creates semi-transparency.
- **Invert Mask (`Ctrl + I`)**: Flips masked and unmasked regions instantly.
- **Link / Unlink**: Click the chain icon between the layer thumbnail and mask thumbnail to transform the layer independently of its mask.

### Clipping Masks (`Ctrl + Alt + G`)
Pins the visibility of the active layer strictly to the alpha shape of the layer beneath it:
1. Position your texture or color layer directly above the base shape layer.
2. Press `Ctrl + Alt + G` (or right-click > Create Clipping Mask).
3. The upper layer indents with an arrow, only showing where the base layer has opaque pixels.

---

## 7. The 25 Photoshop Blend Modes Guide

Compositor implements all 25 Photoshop blend modes in precise mathematical order:

| Category | Blend Mode | Visual Effect / Ideal Use Case |
| :--- | :--- | :--- |
| **Normal** | `Normal` | Standard alpha over compositing. |
| | `Dissolve` | Random dithered noise transparency based on alpha. |
| **Darken** | `Darken` | Retains darker pixel values; lighter values disappear. |
| | `Multiply` | Multiplies color channels. Excellent for adding shadows and line art. |
| | `Color Burn` | Increases contrast to darken base colors toward shadow tones. |
| | `Linear Burn` | Decreases brightness uniformly to darken colors. |
| | `Darker Color` | Compares composite channel luminance and keeps the darker color. |
| **Lighten** | `Lighten` | Retains lighter pixel values; darker values disappear. |
| | `Screen` | Inverts, multiplies, and inverts back. Essential for light glows, fire, sparks. |
| | `Color Dodge` | Decreases contrast to brighten base colors toward highlight tones. |
| | `Linear Dodge (Add)`| Sums color channels. Perfect for intense specular lights and flares. |
| | `Lighter Color` | Compares composite channel luminance and keeps the lighter color. |
| **Contrast** | `Overlay` | Multiplies darks and screens lights. Boosts contrast while preserving highlights. |
| | `Soft Light` | Gentle diffused contrast; mimics soft spotlight illumination. |
| | `Hard Light` | Intense contrast; mimics direct harsh sunlight or strong spotlight. |
| | `Vivid Light` | Burns shadows and dodges highlights based on blend layer intensity. |
| | `Linear Light` | Linear burns shadows and linear dodges highlights. |
| | `Pin Light` | Replaces colors based on threshold; useful for specialized textures. |
| | `Hard Mix` | Quantizes colors to pure primary/secondary primaries (extreme posterization).|
| **Inversion** | `Difference` | Subtracts blend from base or vice versa. Essential for layer alignment verification. |
| | `Exclusion` | Softer, lower-contrast alternative to Difference. |
| | `Subtract` | Subtracts blend values directly from the base image. |
| | `Divide` | Divides base values by blend values. Corrects color casts and uneven lighting. |
| **Component** | `Hue` | Combines luminance & saturation of base with hue of blend layer. |
| | `Saturation` | Combines luminance & hue of base with saturation of blend layer. |
| | `Color` | Combines luminance of base with hue & saturation of blend. Standard for hand-coloring B&W photos. |
| | `Luminosity` | Combines color of base with luminance of blend. Enhances details without altering hues. |

---

## 8. Non-Destructive GPU Layer Effects

Access layer effects by double-clicking a layer in the Layers Panel or opening **Layer > Layer Style**:

1. **Drop Shadow**:
   - *Distance*: Offset from layer in document pixels.
   - *Spread*: Hardens expansion before falloff.
   - *Size*: Gaussian blur radius.
   - *Angle*: Direction of light source ($0^\circ$ to $360^\circ$).
2. **Inner Shadow**: Recesses the layer inwards, creating embossed depth.
3. **Outer Glow & Inner Glow**: Atmospheric luminescence with customizable color, opacity, and spread.
4. **Bevel & Emboss**: Simulates 3D raised surface chiseled lighting with highlight and shadow blend modes.
5. **Color Overlay & Gradient Overlay**: Non-destructively covers the layer in solid hues or linear/radial gradient fills.
6. **Stroke**: Outlines layer contours with Inside, Center, or Outside placement.

---

## 9. Color Grading, Curves & Camera Raw Studio

### Curves (`Ctrl + M`)
The **Curves** modal provides an interactive monotone cubic Hermite spline curve editor:
- Click anywhere on the diagonal line to add control points.
- Switch between **Composite RGB**, **Red**, **Green**, and **Blue** channels.
- Create S-curves to boost cinematic punch, or lift the bottom-left anchor point to generate fashionable matte/faded shadow tones.

### Levels (`Ctrl + L`)
Fine-tune tonal range using input and output sliders:
- **Black Point (0–255)**: Maps shadows to pure black.
- **Midtone Gamma (0.1–9.9)**: Adjusts midtone brightness without clipping highlights.
- **White Point (0–255)**: Maps highlights to pure white.

### Camera Raw Studio (`Filter > Camera Raw`)
Complete development engine for RAW and raster images:
- **Exposure & Contrast**: Overall luminance balance.
- **Highlights & Shadows**: Recovers clipped skies and lifts dark shadow details.
- **Whites & Blacks**: Sets absolute dynamic range endpoints.
- **Temperature & Tint**: True Kelvin white balance and green/magenta tint correction.
- **Clarity & Vibrance**: Enhances midtone local contrast and boosts muted colors without over-saturating skin tones.
- **Optics & Lens Distortion**: Corrects barrel and pincushion lens distortion with chromatic aberration removal.

---

## 10. Vector Shapes & Typography

### Parametric Vector Shapes (`U`)
- Select **Rectangle**, **Rounded Rectangle**, **Ellipse**, or **Line**.
- Draw shapes on canvas. Unlike raster brush strokes, vector shapes store geometric attributes.
- Adjust **Fill Color**, **Stroke Color**, **Stroke Width**, and **Corner Radius** at any time in the Tool Options bar without re-drawing.

### Type Tool (`T`)
- Click on canvas to start a point text label, or click and drag to define a **paragraph text box**.
- Enter multiline copy with automatic word-wrapping.
- Adjust Font Family, Font Size, Line Spacing, Tracking, and Text Color in real time.

---

## 11. Canvas Geometry, Smart Crops & Edge Trim

### Free Transform & Distort (`Ctrl + T`)
- Press `Ctrl + T` to activate transform handles.
- **Scale**: Drag any corner handle. Hold `Shift` to constrain proportions; hold `Alt` to scale from the center.
- **Rotate**: Hover cursor outside any corner handle until rotation cursor appears, then drag.
- **Free Distort**: Hold `Ctrl` while dragging any corner to independently shear, skew, and perspective-warp the layer.
- Press `Enter` to commit, or `Esc` to cancel.

### Interactive Crop (`C`)
- Drag handles to frame desired composition.
- Presets: Unconstrained, 1:1 Square, 16:9 Landscape, 4:5 Portrait.
- Press `Enter` to crop canvas.

### Smart Edge Trim (`Image > Trim`)
- Automatically clips away transparent padding or solid edge borders based on the top-left pixel color.

---

## 12. Autonomous AI Collaboration & Live .comp Sync

Compositor features a built-in kernel file watcher (`ReadDirectoryChangesW`) with a **300 ms coalescing engine** that allows AI agents (like Claude Code, Antigravity, or custom automation scripts) to build projects alongside you in real time.

### How to Use AI Live-Sync:
1. Save your canvas as a `.comp` project folder (e.g. `C:\Projects\Artwork.comp`).
2. Keep Compositor open with `Artwork.comp` displayed on your screen.
3. Instruct an AI coding assistant:
   > "Read `docs/writing-comp-files.md` in the Compositor repo, then add a dramatic starry sky layer behind the mountains in `C:\Projects\Artwork.comp`."
4. **Watch your canvas update automatically** within a fraction of a second as the AI agent writes PNG layers into `images/` and updates `manifest.json`.
5. You can immediately paint on the layers generated by the AI, adjust their blend modes, or add masks.

---

## 13. Photoshop-Standard Keyboard Shortcuts Cheat Sheet

| Action | Shortcut | Action | Shortcut |
| :--- | :--- | :--- | :--- |
| **New Document** | `Ctrl + N` | **Zoom In / Out** | `Ctrl + +` / `Ctrl + -` |
| **Open Document** | `Ctrl + O` | **Fit on Screen** | `Ctrl + 0` |
| **Save Document** | `Ctrl + S` | **Actual Pixels (100%)** | `Ctrl + 1` |
| **Undo / Redo** | `Ctrl + Z` / `Ctrl + Y` | **Pan Canvas** | `Spacebar + Drag` |
| **Free Transform** | `Ctrl + T` | **Brush Size** | `[` / `]` |
| **Curves Modal** | `Ctrl + M` | **Brush Hardness** | `Shift + [` / `Shift + ]` |
| **Levels Modal** | `Ctrl + L` | **Swap Colors** | `X` |
| **Hue / Saturation** | `Ctrl + U` | **Default B/W Colors** | `D` |
| **Select All** | `Ctrl + A` | **New Layer** | `Ctrl + Shift + N` |
| **Deselect** | `Ctrl + D` | **Duplicate Layer** | `Ctrl + J` |
| **Invert Selection** | `Ctrl + Shift + I` | **Group Layers** | `Ctrl + G` |
| **Invert Pixels/Mask** | `Ctrl + I` | **Merge Layers** | `Ctrl + E` |
| **Create Clipping Mask**| `Ctrl + Alt + G` | **Copy Merged** | `Ctrl + Shift + C` |

---

## 14. Performance Tuning & Large Document Optimization

- **Tiled Cache Memory**: Large canvases (up to $16,384 \times 16,384$ and $200\text{ MP}$) are subdivided into $256 \times 256$ tiles. Compositor only commits visible viewport tiles into GPU memory.
- **History Memory Budget**: Bounded at 500 MB by default. Undo states store tile deltas rather than whole canvas copies. To customize, adjust memory preferences in settings.
- **Hardware Tier Acceleration**: Automatically leverages Direct3D 12 Feature Level 11_0+. On multi-GPU laptops, ensure Compositor is assigned to the "High Performance GPU" in Windows Display Settings > Graphics.
