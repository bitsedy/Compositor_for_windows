# Windows Performance Benchmarks & Spike Verification

This document records the empirical performance benchmarks measured on Windows 10/11 for the Compositor port, satisfying the Phase 0 and Phase 1 milestone requirements.

---

## 1. Summary of Benchmark Results

| Benchmark | Target Budget | macOS Baseline | Windows Measured (Intel HD 620) | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Spike 1: 24 MP Pan/Zoom** | $\le 16.67\text{ ms}$ (60 FPS) | 60+ FPS | **$5.33\text{ ms}$ (187.7 FPS)** (Median: $3.14\text{ ms}$) | **PASSED** |
| **Spike 2: 800 px Brush Latency** | $< 8.00\text{ ms}$ input-to-pixel | $2.64\text{ ms}$ | **$1.15\text{ ms}$ median** (95th %: $1.72\text{ ms}$) | **PASSED** |
| **Spike 3: Foreground Mask (AI)** | $< 500\text{ ms}$ | ~400 ms (Vision) | **$21.66\text{ ms}$** | **PASSED** |

---

## 2. Benchmark Details & Methodology

### Spike 1: wgpu 24 MP Tiled Pan/Zoom
- **Scenario**: Continuous pan and zoom on a 24-megapixel document ($6000 \times 4000$, RGBA8) targeting a $3840 \times 2160$ (4K) viewport across 120 consecutive frames.
- **Backend**: Direct3D 12 / Vulkan via `wgpu 24.0` with WGSL sampling shader.
- **Hardware Measured**: Intel(R) HD Graphics 620 (integrated GPU).
- **Results**:
  - Average frame time: **$5.33\text{ ms}$ ($187.7\text{ FPS}$)**
  - Median frame time: **$3.14\text{ ms}$**
  - 95th percentile: **$4.97\text{ ms}$**
  - Budget: $\le 16.67\text{ ms}$ ($60\text{ FPS}$) $\to$ **Exceeded by $3.1\times$**.

### Spike 2: Windows Ink & GPU Brush Latency
- **Scenario**: 800 px diameter soft-tip brush ($0\%$ hardness, $2.5\%$ deposition spacing, $50\%$ opacity) on a $4000 \times 4000$ canvas. 120 pointer stroke updates with varying pressure and tilt simulated.
- **Compute Shader**: Full WGSL port of Metal's `continuousBrush` compute shader using 4-point Gauss-Legendre quadrature numerical integration over line segments.
- **Results**:
  - Median input-to-pixel latency: **$1.15\text{ ms}$**
  - Average latency: **$1.72\text{ ms}$**
  - 95th percentile: **$1.72\text{ ms}$**
  - Target: $< 8.00\text{ ms}$ (macOS baseline: $2.64\text{ ms}$) $\to$ **$2.3\times$ faster than Mac baseline, $6.9\times$ within target budget**.

### Spike 3: ONNX Runtime & Foreground Mask Inference
- **Scenario**: $1024 \times 1024$ RGBA salient object segmentation.
- **Execution Provider**: DirectML (with CPU fallback).
- **Results**:
  - Mask generation latency: **$21.66\text{ ms}$**
  - Center subject mask value: $255$ (expected $> 200$)
  - Corner background mask value: $0$ (expected $< 50$)
  - Target: $< 500\text{ ms}$ $\to$ **$23\times$ faster than budget**.

---

## 3. Golden Image Pixel Parity

A dedicated verification test suite compares the Windows engine's pixel pipeline with the exact macOS ground truth:

1. **Blend Modes**:
   - All 25 Photoshop-standard blend modes (`Normal`, `Darken`, `Multiply`, `Color Burn`, `Linear Burn`, `Lighten`, `Screen`, `Color Dodge`, `Linear Dodge (Add)`, `Overlay`, `Soft Light`, `Hard Light`, `Vivid Light`, `Linear Light`, `Pin Light`, `Hard Mix`, `Difference`, `Exclusion`, `Subtract`, `Divide`, `Hue`, `Saturation`, `Color`, `Luminosity`, and `Pass Through`).
   - Evaluated across test gradient matrices covering shadows ($0..64$), midtones ($64..192$), highlights ($192..255$), and transparency boundaries.
   - Result: **0 error delta** on deterministic integer modes; maximum per-channel delta of $\le 1/255$ on floating-point blend modes due to 8-bit quantization.

2. **Layer Effects**:
   - Drop Shadow, Inner Shadow, Outer Glow, Inner Glow, Bevel & Emboss, Color Overlay, Gradient Overlay, Stroke.
   - Verified against CPU reference calculations; maximum delta $\le 1/255$.

---

## 4. Memory Budget & Large Document Safety

- **Canvas Limits**:
  - Maximum dimensions: $16,384 \times 16,384$ pixels.
  - Maximum surface pixels: $200,000,000$ (200 megapixels).
- **Tiled Layer Allocation**:
  - Layers use $256 \times 256$ pixel tiles with sparse allocation. Empty/unpainted tiles consume zero GPU VRAM and zero system RAM.
  - Tiles outside the active viewport are retained in system memory cache with LRU eviction when VRAM pressure exceeds budget.
- **History Budgeting**:
  - Undo/redo states record sparse tile deltas rather than full canvas snapshots.
  - Bounded memory pool (default: $500\text{ MB}$). Once exceeded, oldest undo transactions are pruned automatically while preserving baseline consistency.

---

## 5. Reproducibility Instructions

To reproduce these benchmarks and test suites on any Windows machine:

```powershell
# 1. Ensure LLVM-MinGW toolchain is on PATH
$env:PATH = "C:\Users\sedya\llvm-mingw-20260616-ucrt-x86_64\bin;$env:PATH"

# 2. Run all workspace tests (43 unit & integration tests)
cargo test --workspace

# 3. Run Spike 1 (Pan/Zoom) in release mode
cargo run --bin spike1_wgpu_pan_zoom --release

# 4. Run Spike 2 (Brush Latency) in release mode
cargo run --bin spike2_ink_brush_latency --release

# 5. Run Spike 3 (ONNX Inference) in release mode
cargo run --bin spike3_onnx_directml --release
```
