#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex, OnceLock};
use std::ptr::null_mut;
use tauri::{Manager, State};
use uuid::Uuid;
use glam::Vec2;

use compositor_core::{
    CanvasDocument, DocumentHistory, DocumentSelection, ImageLayer, LayerBlendMode,
    LayerId, PaletteColor, invert_selection as core_invert_selection,
    filter::{FilterParameters, GaussianBlurSettings, MotionBlurSettings, NoiseSettings},
};
use compositor_gpu::{GpuContext, TiledLayerRenderer, BrushCoveragePipeline};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassExW, SetWindowPos,
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, HMENU, SWP_NOACTIVATE, SWP_NOZORDER,
    WINDOW_EX_STYLE, WNDCLASSEXW, WS_CHILD, WS_VISIBLE, WS_CLIPSIBLINGS,
};
use windows::Win32::UI::Input::Pointer::{
    GetPointerInfo, GetPointerPenInfo, POINTER_INFO, POINTER_PEN_INFO,
};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BrushSettings {
    pub size: f32,
    pub hardness: f32,
    pub opacity: f32,
    pub flow: f32,
    pub spacing: f32,
    pub smoothing: f32,
    pub pressure_size: bool,
    pub pressure_opacity: bool,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 30.0,
            hardness: 0.8,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.25,
            smoothing: 0.0,
            pressure_size: true,
            pressure_opacity: false,
        }
    }
}

pub struct AppState {
    pub document: Option<CanvasDocument>,
    pub history: DocumentHistory,
    pub active_tool: String,
    pub active_layer_id: Option<LayerId>,
    pub brush_settings: BrushSettings,
    pub foreground_color: PaletteColor,
    pub background_color: PaletteColor,
    pub child_hwnd: Option<isize>,
    pub gpu: Option<Arc<GpuContext>>,
    pub renderer: Option<TiledLayerRenderer>,
    pub brush_pipeline: Option<BrushCoveragePipeline>,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Default for AppState {
    fn default() -> Self {
        let mut doc = CanvasDocument::new(1920, 1080);
        let bg_layer = ImageLayer::new_empty("Background", Vec2::new(1920.0, 1080.0));
        let bg_id = bg_layer.id;
        doc.layers.push(bg_layer);

        let mut history = DocumentHistory::new(100, 500 * 1024 * 1024);
        history.begin("Open Document", Some(&doc), Some(bg_id));
        history.end(Some(&doc), Some(bg_id));

        Self {
            document: Some(doc),
            history,
            active_tool: "brush".to_string(),
            active_layer_id: Some(bg_id),
            brush_settings: BrushSettings::default(),
            foreground_color: PaletteColor::black(),
            background_color: PaletteColor::white(),
            child_hwnd: None,
            gpu: None,
            renderer: None,
            brush_pipeline: None,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

static GLOBAL_STATE: OnceLock<Arc<Mutex<AppState>>> = OnceLock::new();

unsafe extern "system" fn canvas_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    const WM_POINTERDOWN: u32 = 0x0246;
    const WM_POINTERUPDATE: u32 = 0x0245;
    const WM_POINTERUP: u32 = 0x0247;
    const WM_MOUSEWHEEL: u32 = 0x020A;

    match msg {
        WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP => {
            let pointer_id = (wparam.0 & 0xFFFF) as u32;
            let mut pointer_info = POINTER_INFO::default();
            if GetPointerInfo(pointer_id, &mut pointer_info).is_ok() {
                let mut pen_info = POINTER_PEN_INFO::default();
                let pressure = if GetPointerPenInfo(pointer_id, &mut pen_info).is_ok() {
                    (pen_info.pressure as f32) / 1024.0
                } else {
                    1.0
                };

                let screen_x = pointer_info.ptPixelLocation.x as f32;
                let screen_y = pointer_info.ptPixelLocation.y as f32;

                if let Some(state_arc) = GLOBAL_STATE.get() {
                    if let Ok(mut state) = state_arc.try_lock() {
                        if state.active_tool == "brush" {
                            let size = if state.brush_settings.pressure_size {
                                state.brush_settings.size * pressure.max(0.05)
                            } else {
                                state.brush_settings.size
                            };

                            let pan_x = state.pan_x;
                            let pan_y = state.pan_y;
                            let zoom = state.zoom;
                            if let Some(renderer) = &mut state.renderer {
                                let doc_x = (screen_x - pan_x) / zoom;
                                let doc_y = (screen_y - pan_y) / zoom;
                                let rx = (doc_x - size).max(0.0) as i32;
                                let ry = (doc_y - size).max(0.0) as i32;
                                let rw = (size * 2.0) as i32;
                                let rh = (size * 2.0) as i32;
                                renderer.mark_dirty_rect(rx, ry, rw, rh);
                            }
                        }
                    }
                }
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) as i16) as f32 / 120.0;
            if let Some(state_arc) = GLOBAL_STATE.get() {
                if let Ok(mut state) = state_arc.try_lock() {
                    let factor = if delta > 0.0 { 1.15 } else { 0.85 };
                    state.zoom = (state.zoom * factor).clamp(0.05, 32.0);
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// -----------------------------------------------------------------------------
// Tauri IPC Commands
// -----------------------------------------------------------------------------

#[tauri::command]
fn new_document(
    state: State<Arc<Mutex<AppState>>>,
    name: String,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let mut doc = CanvasDocument::new(width, height);
    let bg_layer = ImageLayer::new_empty("Background", Vec2::new(width as f32, height as f32));
    let bg_id = bg_layer.id;
    doc.layers.push(bg_layer);

    state.history.begin(name, Some(&doc), Some(bg_id));
    state.history.end(Some(&doc), Some(bg_id));

    state.document = Some(doc);
    state.active_layer_id = Some(bg_id);
    Ok("OK".to_string())
}

#[tauri::command]
fn undo(state: State<Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let name = state.history.undo_name().to_string();

    if let Some(snapshot) = state.history.undo() {
        state.document = snapshot.document;
        state.active_layer_id = snapshot.active_layer_id;
        Ok(name)
    } else {
        Err("Nothing to undo".to_string())
    }
}

#[tauri::command]
fn redo(state: State<Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let name = state.history.redo_name().to_string();

    if let Some(snapshot) = state.history.redo() {
        state.document = snapshot.document;
        state.active_layer_id = snapshot.active_layer_id;
        Ok(name)
    } else {
        Err("Nothing to redo".to_string())
    }
}

#[tauri::command]
fn set_active_tool(state: State<Arc<Mutex<AppState>>>, tool: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.active_tool = tool;
    Ok(())
}

#[tauri::command]
fn set_brush_settings(
    state: State<Arc<Mutex<AppState>>>,
    settings: BrushSettings,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.brush_settings = settings;
    Ok(())
}

#[tauri::command]
fn set_colors(
    state: State<Arc<Mutex<AppState>>>,
    fg: PaletteColor,
    bg: PaletteColor,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.foreground_color = fg;
    state.background_color = bg;
    Ok(())
}

#[tauri::command]
fn select_all(state: State<Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(doc) = &mut state.document {
        let (w, h) = (doc.width as f32, doc.height as f32);
        let sel = DocumentSelection::from_rect(Vec2::new(0.0, 0.0), Vec2::new(w, h), true, 0.0);
        doc.selection = Some(sel);
    }
    Ok(())
}

#[tauri::command]
fn deselect(state: State<Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(doc) = &mut state.document {
        doc.selection = None;
    }
    Ok(())
}

#[tauri::command]
fn invert_selection(state: State<Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(doc) = &mut state.document {
        let inverted = core_invert_selection(doc.selection.as_ref(), doc.width, doc.height);
        doc.selection = inverted;
    }
    Ok(())
}

#[tauri::command]
fn add_layer(state: State<Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let doc_ref = state.document.clone();
    let active_id = state.active_layer_id;
    state.history.begin("New Layer", doc_ref.as_ref(), active_id);

    let id = if let Some(doc) = &mut state.document {
        let size = Vec2::new(doc.width as f32, doc.height as f32);
        let layer = ImageLayer::new_empty(format!("Layer {}", doc.layers.len() + 1), size);
        let lid = layer.id;
        doc.layers.push(layer);
        lid
    } else {
        return Err("No active document".to_string());
    };

    state.active_layer_id = Some(id);
    let current_doc = state.document.clone();
    state.history.end(current_doc.as_ref(), Some(id));
    Ok(id.to_string())
}

#[tauri::command]
fn add_group(state: State<Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let doc_ref = state.document.clone();
    let active_id = state.active_layer_id;
    state.history.begin("New Group", doc_ref.as_ref(), active_id);

    let id = if let Some(doc) = &mut state.document {
        let group = ImageLayer::new_group(format!("Group {}", doc.layers.len() + 1));
        let gid = group.id;
        doc.layers.push(group);
        gid
    } else {
        return Err("No active document".to_string());
    };

    state.active_layer_id = Some(id);
    let current_doc = state.document.clone();
    state.history.end(current_doc.as_ref(), Some(id));
    Ok(id.to_string())
}

#[tauri::command]
fn delete_layer(state: State<Arc<Mutex<AppState>>>, id: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let parsed_id = Uuid::parse_str(&id).map(LayerId).map_err(|e| e.to_string())?;
    let doc_ref = state.document.clone();
    let active_id = state.active_layer_id;
    state.history.begin("Delete Layer", doc_ref.as_ref(), active_id);

    if let Some(doc) = &mut state.document {
        if let Some(idx) = doc.layers.iter().position(|l| l.id == parsed_id) {
            doc.layers.remove(idx);
        }
        let new_active = doc.layers.last().map(|l| l.id);
        state.active_layer_id = new_active;
    }

    let current_doc = state.document.clone();
    let current_active = state.active_layer_id;
    state.history.end(current_doc.as_ref(), current_active);
    Ok(())
}

#[tauri::command]
fn toggle_layer_visibility(state: State<Arc<Mutex<AppState>>>, id: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let parsed_id = Uuid::parse_str(&id).map(LayerId).map_err(|e| e.to_string())?;
    if let Some(doc) = &mut state.document {
        if let Some(l) = doc.layer_mut(parsed_id) {
            l.is_visible = !l.is_visible;
        }
    }
    Ok(())
}

#[tauri::command]
fn set_layer_blend_mode(
    state: State<Arc<Mutex<AppState>>>,
    id: String,
    blend_mode: String,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let parsed_id = Uuid::parse_str(&id).map(LayerId).map_err(|e| e.to_string())?;
    let mode = match blend_mode.as_str() {
        "Multiply" => LayerBlendMode::Multiply,
        "Screen" => LayerBlendMode::Screen,
        "Overlay" => LayerBlendMode::Overlay,
        "Darken" => LayerBlendMode::Darken,
        "Lighten" => LayerBlendMode::Lighten,
        "Color Dodge" => LayerBlendMode::ColorDodge,
        "Color Burn" => LayerBlendMode::ColorBurn,
        "Hard Light" => LayerBlendMode::HardLight,
        "Soft Light" => LayerBlendMode::SoftLight,
        "Difference" => LayerBlendMode::Difference,
        "Exclusion" => LayerBlendMode::Exclusion,
        "Hue" => LayerBlendMode::Hue,
        "Saturation" => LayerBlendMode::Saturation,
        "Color" => LayerBlendMode::Color,
        "Luminosity" => LayerBlendMode::Luminosity,
        _ => LayerBlendMode::Normal,
    };
    if let Some(doc) = &mut state.document {
        if let Some(l) = doc.layer_mut(parsed_id) {
            l.blend_mode = mode;
        }
    }
    Ok(())
}

#[tauri::command]
fn set_layer_opacity(
    state: State<Arc<Mutex<AppState>>>,
    id: String,
    opacity: f32,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let parsed_id = Uuid::parse_str(&id).map(LayerId).map_err(|e| e.to_string())?;
    if let Some(doc) = &mut state.document {
        if let Some(l) = doc.layer_mut(parsed_id) {
            l.opacity = opacity.clamp(0.0, 1.0);
        }
    }
    Ok(())
}

#[tauri::command]
fn set_canvas_viewport(
    state: State<Arc<Mutex<AppState>>>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    if let Some(hwnd_val) = state.child_hwnd {
        let hwnd = HWND(hwnd_val as *mut _);
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                None,
                x,
                y,
                width as i32,
                height as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    Ok(())
}

#[tauri::command]
fn apply_filter_command(
    state: State<Arc<Mutex<AppState>>>,
    kind: String,
    settings: serde_json::Value,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let params = match kind.as_str() {
        "Gaussian Blur" => {
            let r = settings.get("radius").and_then(|v| v.as_f64()).unwrap_or(10.0);
            FilterParameters::GaussianBlur(GaussianBlurSettings { radius: r })
        }
        "Motion Blur" => {
            let a = settings.get("angle").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let d = settings.get("distance").and_then(|v| v.as_f64()).unwrap_or(10.0);
            FilterParameters::MotionBlur(MotionBlurSettings { angle: a, distance: d })
        }
        "Add Noise" => {
            let amt = settings.get("amount").and_then(|v| v.as_f64()).unwrap_or(10.0) as f32;
            let g = settings.get("gaussian").and_then(|v| v.as_bool()).unwrap_or(false);
            FilterParameters::Noise(NoiseSettings {
                amount: amt,
                gaussian: g,
                monochromatic: false,
                seed: 42,
            })
        }
        "Invert" => FilterParameters::Invert,
        _ => return Ok(()),
    };

    let doc_ref = state.document.clone();
    let active_id = state.active_layer_id;
    state.history.begin(kind, doc_ref.as_ref(), active_id);
    let current_doc = state.document.clone();
    state.history.end(current_doc.as_ref(), active_id);
    let _ = params;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OpenedImageDto {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub base64: String,
}

#[tauri::command]
fn open_image_file(state: State<Arc<Mutex<AppState>>>, path: String) -> Result<OpenedImageDto, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    let img = image::open(p).map_err(|e| format!("Failed to decode image: {}", e))?;
    let (w, h) = (img.width(), img.height());
    let rgba = img.to_rgba8();

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    use image::ImageEncoder;
    encoder.write_image(
        &rgba,
        w,
        h,
        image::ExtendedColorType::Rgba8,
    ).map_err(|e| e.to_string())?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("Imported Image").to_string();

    let mut state = state.lock().map_err(|e| e.to_string())?;
    let mut doc = CanvasDocument::new(w, h);
    let bg_layer = ImageLayer::new_empty(&name, Vec2::new(w as f32, h as f32));
    let bg_id = bg_layer.id;
    doc.layers.push(bg_layer);
    state.history.begin(format!("Open {}", name), Some(&doc), Some(bg_id));
    state.history.end(Some(&doc), Some(bg_id));
    state.document = Some(doc);
    state.active_layer_id = Some(bg_id);

    Ok(OpenedImageDto {
        name,
        width: w,
        height: h,
        base64: format!("data:image/png;base64,{}", b64),
    })
}

#[tauri::command]
fn import_image_data(
    state: State<Arc<Mutex<AppState>>>,
    name: String,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let doc_ref = state.document.clone();
    let active_id = state.active_layer_id;
    state.history.begin(format!("Import {}", name), doc_ref.as_ref(), active_id);

    let id = if let Some(doc) = &mut state.document {
        let layer = ImageLayer::new_empty(&name, Vec2::new(width as f32, height as f32));
        let lid = layer.id;
        doc.layers.push(layer);
        lid
    } else {
        let mut doc = CanvasDocument::new(width, height);
        let layer = ImageLayer::new_empty(&name, Vec2::new(width as f32, height as f32));
        let lid = layer.id;
        doc.layers.push(layer);
        state.document = Some(doc);
        lid
    };

    state.active_layer_id = Some(id);
    let cur_doc = state.document.clone();
    state.history.end(cur_doc.as_ref(), Some(id));
    Ok(id.to_string())
}

#[tauri::command]
fn export_image(
    _state: State<Arc<Mutex<AppState>>>,
    format: String,
    quality: f32,
) -> Result<String, String> {
    log::info!("Export image requested: format={}, quality={}", format, quality);
    Ok("OK".to_string())
}

#[tauri::command]
fn save_project(
    _state: State<Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    Ok("Saved".to_string())
}

#[tauri::command]
fn clipboard_copy() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn clipboard_paste() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn clipboard_cut() -> Result<(), String> {
    Ok(())
}

// -----------------------------------------------------------------------------
// App Initialization
// -----------------------------------------------------------------------------

fn main() {
    env_logger::init();

    let state = Arc::new(Mutex::new(AppState::default()));
    let _ = GLOBAL_STATE.set(state.clone());

    let state_for_setup = state.clone();

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();
            let parent_hwnd = window.hwnd().unwrap().0 as isize;

            let class_name: Vec<u16> = "CompositorCanvas\0".encode_utf16().collect();
            unsafe {
                let hinstance = GetModuleHandleW(None).unwrap();
                let wndclass = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
                    lpfnWndProc: Some(canvas_wnd_proc),
                    hInstance: hinstance.into(),
                    lpszClassName: PCWSTR(class_name.as_ptr()),
                    ..Default::default()
                };
                let _ = RegisterClassExW(&wndclass);

                let child = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(class_name.as_ptr()),
                    PCWSTR(null_mut()),
                    WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                    44,
                    60,
                    1100,
                    800,
                    HWND(parent_hwnd as *mut _),
                    HMENU(101 as *mut _),
                    hinstance,
                    None,
                );

                if let Ok(c) = child {
                    if let Ok(mut s) = state_for_setup.lock() {
                        s.child_hwnd = Some(c.0 as isize);
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            new_document,
            undo,
            redo,
            set_active_tool,
            set_brush_settings,
            set_colors,
            select_all,
            deselect,
            invert_selection,
            add_layer,
            add_group,
            delete_layer,
            toggle_layer_visibility,
            set_layer_blend_mode,
            set_layer_opacity,
            set_canvas_viewport,
            apply_filter_command,
            open_image_file,
            import_image_data,
            save_project,
            export_image,
            clipboard_copy,
            clipboard_paste,
            clipboard_cut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running compositor application");
}
