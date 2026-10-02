# Architecture & Design Decisions Record (ADR)

## DECISION-001: Target Technology Stack

- **Context**: Compositor is ~33.7k lines of Swift and ~2.5k lines of C designed exclusively for macOS (AppKit, SwiftUI, Metal, Core Image, Core Graphics, Vision). Windows requires a complete re-platforming with strict performance budgets (<8ms brush latency on 4K, 60fps pan/zoom on 24MP 50-layer document).
- **Decision**:
  - **Core Engine**: Rust 2021/2024. Completely headless, strictly separated from any UI logic. Document models, layer hierarchy, masks, history, selections, PSD/PSB parser, .comp I/O, filters, brush engine.
  - **GPU Subsystem**: `wgpu` 24.0 on Direct3D 12 (primary Windows backend) with Vulkan fallback. Tiled layer compositing, GPU layer effects, GPU warp/liquify, GPU noise, GPU continuous brush coverage. MSL shaders translated 1:1 to WGSL.
  - **Desktop Shell & UI**: Tauri 2 with React 18, TypeScript, Tailwind CSS, and Radix UI primitives.
  - **Canvas Surface**: Native Win32 child window hosting a Direct3D 12 / wgpu swapchain surface. Pointer events (`WM_POINTERUPDATE`, `WM_POINTERDOWN`, `WM_POINTERUP`) and pen pressure/tilt from Windows Ink are captured directly in the Win32 window procedure in Rust and fed straight to the GPU pipeline without webview IPC hops.
- **Consequences**: Avoids any DOM/IPC bottlenecks for painting and rendering while leveraging modern web technologies for complex floating sheets, nested layer panels, and inspector controls.

---

## DECISION-002: C Kernels Re-use and Compilation Strategy

- **Context**: The existing macOS codebase relies on ~2.5k lines of highly optimized C kernels (`AdjustPixels.c`, `BrushPixels.c`, `ContentFill.c`, `DitherPixels.c`, `HealPixels.c`, `LensPixels.c`, `LevelsPixels.c`, `NoisePixels.c`, `WandPixels.c`).
- **Decision**: Compile the portable C kernels directly using MSVC/Clang via Cargo's `cc` crate into `compositor-pixel`, exposing safe Rust wrapper abstractions. Where profiling and benchmarks demonstrate a bottleneck, port critical inner loops to Rust with explicit SIMD (`std::simd` / `wide`).
- **Consequences**: Guarantees bit-exact pixel output matching macOS reference implementations while preserving proven numerical stability and algorithms.

---

## DECISION-003: Golden-Image Test Harness Strategy

- **Context**: Running original macOS XCTest binaries is not possible on Windows. We must ensure pixel-level parity (tolerance: max per-channel delta of 1/255 for deterministic ops).
- **Decision**:
  - Extract known test fixtures, expected pixel values, and synthetic image generators from `CompositorTests/` (e.g. `PSDFixture.swift`, `BrushIntersectionTests.swift`, `CameraRawTests.swift`, `LayerAppearanceTests.swift`).
  - Implement ground-truth mathematical reference implementations in Rust for all 24 blend modes based on the Adobe Photoshop / PDF 1.7 blending specifications.
  - Save reference golden RGBA buffers in `tests/golden/` as PNG fixtures.
  - Build a Rust golden test harness comparing engine output against golden images with per-channel delta thresholds and diff image export on failure.
- **Consequences**: Enables automated, deterministic regression testing in continuous integration on both x64 and ARM64.

---

## DECISION-004: DirectML for Foreground Mask and Object Selection

- **Context**: macOS Compositor uses Apple's Vision framework (`VNGenerateForegroundInstanceMaskRequest`) for Select Subject, Remove Background, and Object Selection.
- **Decision**: Use ONNX Runtime (`ort` crate) configured with the DirectML Execution Provider (D3D12 GPU acceleration) on Windows, with a multi-threaded CPU fallback. Use a permissively licensed (MIT / Apache-2.0) salient object detection model (BiRefNet / U2-Net).
- **Consequences**: Leverages any DirectX 12 capable GPU (Intel, AMD, NVIDIA, Qualcomm Adreno) without requiring proprietary CUDA runtimes.

---

## DECISION-005: Workspace Layout and Crate Partitioning

- **Context**: The project must build cleanly, support headless testing, benchmark critical paths, and separate responsibilities.
- **Decision**: Structure the Cargo workspace into modular crates:
  1. `compositor-core`: Headless document model, layers, masks, history, selection, geometry, limits.
  2. `compositor-pixel`: C kernels, safe wrappers, blend modes, adjustments, CPU filters, SIMD math.
  3. `compositor-gpu`: wgpu D3D12/Vulkan renderer, tiled compositor, WGSL shaders, texture cache.
  4. `compositor-io`: Image codecs (PNG, JPEG, TIFF, SVG, HEIC, RAW), PSD/PSB parser/builder, .comp package store & watcher.
  5. `compositor-ai`: ONNX Runtime + DirectML salient object segmentation and background removal.
  6. `compositor-desktop`: Tauri 2 desktop shell, Win32 child window wgpu canvas, Windows Ink input handler, system integration.
- **Consequences**: Headless crates can be developed and tested in parallel with zero dependency on UI or windowing libraries.
