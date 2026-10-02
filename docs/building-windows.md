# Building Compositor on Windows

This guide explains how to configure the development environment, compile from source, run the full test suite and performance benchmarks, and produce production installers for **Compositor on Windows 10 (22H2) and Windows 11** (x64 and ARM64).

---

## 1. Prerequisites & Toolchain Setup

### A. Rust Toolchain
Install Rust 1.80 or newer via [rustup](https://rustup.rs):
```powershell
rustup default stable-x86_64-pc-windows-gnu
# Or for MSVC toolchain:
# rustup default stable-x86_64-pc-windows-msvc
```

### B. C/C++ Compiler for Pixel Kernels
Compositor incorporates 9 performance-critical C kernels (`AdjustPixels.c`, `BrushPixels.c`, `ContentFill.c`, `DitherPixels.c`, `HealPixels.c`, `LensPixels.c`, `LevelsPixels.c`, `NoisePixels.c`, `WandPixels.c`).

Ensure `clang` or `gcc` is available on your `PATH`. For example, with LLVM-MinGW:
```powershell
$env:PATH = "C:\Users\sedya\llvm-mingw-20260616-ucrt-x86_64\bin;$env:PATH"
clang --version
```

### C. Node.js & npm (Frontend Shell)
Node.js 18+ is required to bundle the React/TypeScript frontend:
```powershell
node --version
npm --version
```

### D. Direct3D 12 GPU & Drivers
Direct3D 12 support (Feature Level 11_0 or higher) with current GPU drivers. DirectML is included standard in Windows 10 (build 19041+) and Windows 11.

---

## 2. Step-by-Step Build Instructions

### Step 1: Build Frontend Assets
Compile the React 18 / Vite 6 frontend into `frontend/dist`:
```powershell
cd frontend
npm install
npm run build
cd ..
```
Verify that `frontend/dist/index.html` exists.

### Step 2: Run All Workspace Tests
Execute the comprehensive test suite across all crates (`compositor-core`, `compositor-pixel`, `compositor-gpu`, `compositor-io`, `compositor-desktop`, `spikes`):
```powershell
$env:PATH = "C:\Users\sedya\llvm-mingw-20260616-ucrt-x86_64\bin;$env:PATH"
cargo test --workspace
```
All 43 unit and integration tests must pass, confirming:
- Exact mathematical formulas for all 25 Photoshop blend modes.
- Monotone cubic Hermite spline interpolation for Curves.
- Bounded memory history manager undo/redo operations.
- All C pixel kernels (Wand, Content Fill, Spot Healing, Dither, Levels, Lens Distort, etc.).
- Bidirectional `.comp` manifest (versions 1 through 11) serialization and atomic saving.
- Real-time `ProjectWatcher` event coalescing.

### Step 3: Run Performance Spikes & Verification Harness
Run the three performance benchmarks in release mode to measure GPU and stylus metrics:
```powershell
# Spike 1: 24 MP Pan/Zoom at 60+ FPS
cargo run --bin spike1_wgpu_pan_zoom --release

# Spike 2: Windows Ink 800px Brush Latency (< 8ms)
cargo run --bin spike2_ink_brush_latency --release

# Spike 3: DirectML Salient Object Segmentation (< 500ms)
cargo run --bin spike3_onnx_directml --release
```

### Step 4: Run Compositor Desktop in Development Mode
To launch the full desktop application:
```powershell
cargo run -p compositor-desktop
```

---

## 3. Producing Production Installers

Compositor supports both WiX-based MSI installers and lightweight NSIS setup executables via Tauri's bundler.

### Build Release Binaries and Packages
```powershell
# Build optimized release binary
cargo build -p compositor-desktop --release
```

Using the Tauri CLI:
```powershell
cargo tauri build
```
This generates:
- Standalone executable: `target/release/compositor-desktop.exe`
- NSIS installer: `target/release/bundle/nsis/Compositor_1.0.0_x64-setup.exe`
- MSI installer: `target/release/bundle/msi/Compositor_1.0.0_x64_en-US.msi`

---

## 4. Shell Integration & File Association

To register `.comp` project folder associations in Windows Explorer manually:
```powershell
reg import packaging\windows\register_comp_association.reg
```
This registers:
- `ProgID`: `Compositor.Project`
- File extension: `.comp`
- Context menu: "Open with Compositor"
- Directory context menu for directories ending in `.comp`.

---

## 5. WinGet Distribution

To publish or update the Windows Package Manager manifest:
1. Manifests are located in `packaging/winget/`:
   - `compositor.version.yaml`
   - `compositor.locale.en-US.yaml`
   - `compositor.installer.yaml`
2. Update the `InstallerSha256` hash in `compositor.installer.yaml` using:
   ```powershell
   Get-FileHash target\release\bundle\nsis\Compositor_1.0.0_x64-setup.exe -Algorithm SHA256
   ```
3. Validate manifests:
   ```powershell
   winget validate packaging\winget\
   ```
