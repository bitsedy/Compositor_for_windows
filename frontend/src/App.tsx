import React, { useState, useEffect, useRef, useCallback } from 'react';
import { MenuBar } from './components/MenuBar';
import { ToolOptionsBar } from './components/ToolOptionsBar';
import { DocumentTabs } from './components/DocumentTabs';
import { ToolsPalette } from './components/ToolsPalette';
import { CanvasContainer } from './components/CanvasContainer';
import { LayersPanel } from './components/LayersPanel';
import { AdjustmentsPanel } from './components/AdjustmentsPanel';
import { HistoryPanel } from './components/HistoryPanel';
import { StatusBar } from './components/StatusBar';

import { CurvesModal } from './components/modals/CurvesModal';
import { LevelsModal } from './components/modals/LevelsModal';
import { HueSaturationModal } from './components/modals/HueSaturationModal';
import { FilterModal } from './components/modals/FilterModal';
import { CameraRawModal } from './components/modals/CameraRawModal';
import { NewDocModal } from './components/modals/NewDocModal';
import { ExportModal } from './components/modals/ExportModal';

import { ToolKind, LayerItem, BrushSettings, PaletteColor, BlendMode } from './types';

// ---------------------------------------------------------------------------
// History snapshot type
// ---------------------------------------------------------------------------
interface HistoryEntry {
  name: string;
  docWidth: number;
  docHeight: number;
  layers: LayerItem[];
  activeLayerId: string;
  canvases: Map<string, HTMLCanvasElement>;
}

// ---------------------------------------------------------------------------
// Deep-clone a single canvas (preserves pixel data)
// ---------------------------------------------------------------------------
const cloneCanvas = (src: HTMLCanvasElement): HTMLCanvasElement => {
  const copy = document.createElement('canvas');
  copy.width = Math.max(1, src.width || 1);
  copy.height = Math.max(1, src.height || 1);
  const ctx = copy.getContext('2d');
  if (ctx && src.width > 0 && src.height > 0) {
    try {
      ctx.drawImage(src, 0, 0);
    } catch (_) {}
  }
  return copy;
};

// ---------------------------------------------------------------------------
// Deep-clone an entire layer canvas map
// ---------------------------------------------------------------------------
const cloneCanvasesMap = (map: Map<string, HTMLCanvasElement>): Map<string, HTMLCanvasElement> => {
  const copy = new Map<string, HTMLCanvasElement>();
  map.forEach((c, id) => copy.set(id, cloneCanvas(c)));
  return copy;
};

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------
export const App: React.FC = () => {
  const fileInputRef = useRef<HTMLInputElement>(null);

  // ── Tool & Brush ──────────────────────────────────────────────────────────
  const [activeTool, setActiveTool] = useState<ToolKind>('brush');
  const [brushSettings, setBrushSettings] = useState<BrushSettings>({
    size: 30,
    hardness: 80,
    opacity: 100,
    flow: 100,
    spacing: 25,
    smoothing: 0,
    pressureSize: true,
    pressureOpacity: false,
  });

  // ── Colors ────────────────────────────────────────────────────────────────
  const [foregroundColor, setForegroundColor] = useState<PaletteColor>({ red: 0, green: 0, blue: 0, alpha: 1 });
  const [backgroundColor, setBackgroundColor] = useState<PaletteColor>({ red: 1, green: 1, blue: 1, alpha: 1 });

  // ── Document ──────────────────────────────────────────────────────────────
  const [tabs, setTabs] = useState<{ id: string; name: string; isDirty?: boolean }[]>([
    { id: 'doc-1', name: 'Untitled-1', isDirty: false },
  ]);
  const [activeTabId, setActiveTabId] = useState('doc-1');
  const [docWidth, setDocWidth] = useState(1920);
  const [docHeight, setDocHeight] = useState(1080);
  const [zoom, setZoom] = useState(0.5);
  const [panX, setPanX] = useState(80);
  const [panY, setPanY] = useState(40);

  // ── Selection ─────────────────────────────────────────────────────────────
  const [selection, setSelection] = useState<{ x: number; y: number; width: number; height: number } | null>(null);

  // ── Drag-and-drop: use a counter so nested enter/leave events don't flicker
  // counter > 0  → overlay visible
  const [dragCounter, setDragCounter] = useState(0);

  // ── Layers ────────────────────────────────────────────────────────────────
  const [layers, setLayers] = useState<LayerItem[]>([
    {
      id: 'layer-1',
      name: 'Background',
      isVisible: true,
      isGroup: false,
      opacity: 1.0,
      blendMode: 'Normal',
      hasMask: false,
      maskEnabled: false,
      hasEffects: false,
    },
  ]);
  const [activeLayerId, setActiveLayerId] = useState('layer-1');
  const [layerCanvasesMap, setLayerCanvasesMap] = useState<Map<string, HTMLCanvasElement>>(new Map());

  // ── History stack (immutable pixel snapshots) ─────────────────────────────
  // We store the stack in a ref so functional-updates inside setState can
  // always read the latest value without stale-closure issues.
  const [historyStack, setHistoryStack] = useState<HistoryEntry[]>([]);
  const [historyIndex, setHistoryIndex] = useState<number>(-1);
  const historyStackRef = useRef<HistoryEntry[]>([]);
  const historyIndexRef = useRef<number>(-1);

  // Keep refs in sync with state
  useEffect(() => { historyStackRef.current = historyStack; }, [historyStack]);
  useEffect(() => { historyIndexRef.current = historyIndex; }, [historyIndex]);

  // ── Modals ────────────────────────────────────────────────────────────────
  const [isCurvesOpen, setCurvesOpen] = useState(false);
  const [isLevelsOpen, setLevelsOpen] = useState(false);
  const [isHueSatOpen, setHueSatOpen] = useState(false);
  const [activeFilterKind, setActiveFilterKind] = useState<string | null>(null);
  const [isCameraRawOpen, setCameraRawOpen] = useState(false);
  const [isNewDocOpen, setNewDocOpen] = useState(false);
  const [isExportOpen, setExportOpen] = useState(false);

  // ── Tauri IPC ─────────────────────────────────────────────────────────────
  const invokeTauri = async (cmd: string, args?: any) => {
    if ((window as any).__TAURI_INTERNALS__) {
      try {
        const { invoke } = (window as any).__TAURI_INTERNALS__;
        return await invoke(cmd, args);
      } catch (err) {
        console.error(`Tauri invoke error [${cmd}]:`, err);
      }
    }
  };

  // ── Push history snapshot ─────────────────────────────────────────────────
  // CRITICAL: reads historyIndexRef.current (always fresh) to avoid stale closure.
  const pushHistorySnapshot = useCallback(
    (
      name: string,
      overrideMap?: Map<string, HTMLCanvasElement>,
      overrideLayers?: LayerItem[],
      overrideWidth?: number,
      overrideHeight?: number,
      overrideActiveLayerId?: string,
    ) => {
      // Snapshot the currently-active state (or overrides for atomic compound ops)
      // We read from the ref so we always get the latest index even in batched calls.
      const currentIndex = historyIndexRef.current;

      setHistoryStack((prev) => {
        // Trim any future (redo) states
        const trimmed = prev.slice(0, currentIndex + 1);

        // Build the snapshot from overrides or current state; we deliberately
        // capture the React state values via closure at call-time, which is correct.
        const entry: HistoryEntry = {
          name,
          docWidth: overrideWidth ?? docWidth,
          docHeight: overrideHeight ?? docHeight,
          layers: (overrideLayers ?? layers).map((l) => ({ ...l })),
          activeLayerId: overrideActiveLayerId ?? activeLayerId,
          canvases: cloneCanvasesMap(overrideMap ?? layerCanvasesMap),
        };

        const next = [...trimmed, entry];
        if (next.length > 50) next.shift(); // cap at 50 steps
        historyStackRef.current = next;
        return next;
      });

      const nextIndex = Math.min(currentIndex + 1, 49);
      historyIndexRef.current = nextIndex;
      setHistoryIndex(nextIndex);
    },
    // Deps: all current state values that get snapshot'd by default
    [docWidth, docHeight, layers, activeLayerId, layerCanvasesMap],
  );

  // ── Initialize default canvas on app startup ──────────────────────────────
  useEffect(() => {
    const bgCanvas = document.createElement('canvas');
    bgCanvas.width = 1920;
    bgCanvas.height = 1080;
    const ctx = bgCanvas.getContext('2d');
    if (ctx) {
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(0, 0, 1920, 1080);
    }
    const map = new Map<string, HTMLCanvasElement>();
    map.set('layer-1', bgCanvas);
    setLayerCanvasesMap(map);

    const availW = window.innerWidth - 360;
    const availH = window.innerHeight - 150;
    const fitZ = Math.min(availW / 1920, availH / 1080, 0.6);
    setZoom(fitZ);
    setPanX(Math.round((availW - 1920 * fitZ) / 2));
    setPanY(Math.round((availH - 1080 * fitZ) / 2));

    const initialEntry: HistoryEntry = {
      name: 'New Document',
      docWidth: 1920,
      docHeight: 1080,
      layers: [
        {
          id: 'layer-1',
          name: 'Background',
          isVisible: true,
          isGroup: false,
          opacity: 1.0,
          blendMode: 'Normal',
          hasMask: false,
          maskEnabled: false,
          hasEffects: false,
        },
      ],
      activeLayerId: 'layer-1',
      canvases: cloneCanvasesMap(map),
    };
    historyStackRef.current = [initialEntry];
    historyIndexRef.current = 0;
    setHistoryStack([initialEntry]);
    setHistoryIndex(0);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ── Open / load image file ────────────────────────────────────────────────
  const openImageFile = useCallback((file: File) => {
    const reader = new FileReader();
    reader.onerror = () => {
      console.error('Failed to read file:', file.name);
    };
    reader.onload = (e) => {
      const dataUrl = e.target?.result as string;
      if (!dataUrl) return;

      const img = new Image();
      img.onerror = () => {
        console.error('Failed to decode image file:', file.name);
      };
      img.onload = () => {
        const w = img.width;
        const h = img.height;

        setDocWidth(w);
        setDocHeight(h);

        const lCanvas = document.createElement('canvas');
        lCanvas.width = w;
        lCanvas.height = h;
        const ctx = lCanvas.getContext('2d');
        if (ctx) ctx.drawImage(img, 0, 0);

        const newLayerId = `layer-${Date.now()}`;
        const newLayer: LayerItem = {
          id: newLayerId,
          name: file.name.replace(/\.[^/.]+$/, ''),
          isVisible: true,
          isGroup: false,
          opacity: 1.0,
          blendMode: 'Normal',
          hasMask: false,
          maskEnabled: false,
          hasEffects: false,
        };

        const newMap = new Map<string, HTMLCanvasElement>();
        newMap.set(newLayerId, lCanvas);

        // Sync state atomically
        setLayerCanvasesMap(newMap);
        setLayers([newLayer]);
        setActiveLayerId(newLayerId);

        // Auto-center and fit zoom
        const availW = window.innerWidth - 360;
        const availH = window.innerHeight - 150;
        const fitZ = Math.min(availW / w, availH / h, 1.0);
        setZoom(fitZ);
        setPanX(Math.round((availW - w * fitZ) / 2));
        setPanY(Math.round((availH - h * fitZ) / 2));

        const tabName = file.name;
        setTabs([{ id: `doc-${Date.now()}`, name: tabName, isDirty: false }]);
        setSelection(null);

        // Initialize a fresh history stack for this document
        const entry: HistoryEntry = {
          name: `Open ${file.name}`,
          docWidth: w,
          docHeight: h,
          layers: [newLayer],
          activeLayerId: newLayerId,
          canvases: cloneCanvasesMap(newMap),
        };
        historyStackRef.current = [entry];
        historyIndexRef.current = 0;
        setHistoryStack([entry]);
        setHistoryIndex(0);

        invokeTauri('open_image_file', { path: (file as any).path || file.name });
      };
      img.src = dataUrl;
    };
    reader.readAsDataURL(file);
  }, []); // stable – uses only setters and refs

  // ── File input change handler ─────────────────────────────────────────────
  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      openImageFile(e.target.files[0]);
      // Reset so the same file can be re-opened
      e.target.value = '';
    }
  };

  // ── Window-wide drag-and-drop ─────────────────────────────────────────────
  // Using a dragCounter prevents flickering when the mouse crosses child
  // element boundaries inside WebView2.  dragenter increments, dragleave decrements.
  // dragend and drop always reset counter to 0.
  useEffect(() => {
    const onDragEnter = (e: DragEvent) => {
      e.preventDefault();
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
      setDragCounter((c) => c + 1);
    };
    const onDragOver = (e: DragEvent) => {
      e.preventDefault();
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
    };
    const onDragLeave = (e: DragEvent) => {
      e.preventDefault();
      if (e.clientX <= 0 || e.clientY <= 0 || e.clientX >= window.innerWidth || e.clientY >= window.innerHeight) {
        setDragCounter(0);
      } else {
        setDragCounter((c) => Math.max(0, c - 1));
      }
    };
    const onDrop = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragCounter(0);
      if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
        openImageFile(e.dataTransfer.files[0]);
      }
    };
    const onDragEnd = () => {
      setDragCounter(0);
    };

    window.addEventListener('dragenter', onDragEnter);
    window.addEventListener('dragover', onDragOver);
    window.addEventListener('dragleave', onDragLeave);
    window.addEventListener('drop', onDrop);
    window.addEventListener('dragend', onDragEnd);
    return () => {
      window.removeEventListener('dragenter', onDragEnter);
      window.removeEventListener('dragover', onDragOver);
      window.removeEventListener('dragleave', onDragLeave);
      window.removeEventListener('drop', onDrop);
      window.removeEventListener('dragend', onDragEnd);
    };
  }, [openImageFile]);

  // ── Clipboard paste (image) ───────────────────────────────────────────────
  useEffect(() => {
    const handlePaste = (e: ClipboardEvent) => {
      if (e.clipboardData && e.clipboardData.items) {
        for (let i = 0; i < e.clipboardData.items.length; i++) {
          const item = e.clipboardData.items[i];
          if (item.type.indexOf('image') !== -1) {
            const file = item.getAsFile();
            if (file) { openImageFile(file); break; }
          }
        }
      }
    };
    window.addEventListener('paste', handlePaste);
    return () => window.removeEventListener('paste', handlePaste);
  }, [openImageFile]);

  // ── TRUE UNDO (restores physical pixel buffers) ───────────────────────────
  const handleUndo = useCallback(() => {
    const currentIdx = historyIndexRef.current;
    if (currentIdx <= 0) return;

    const targetIdx = currentIdx - 1;
    const targetState = historyStackRef.current[targetIdx];
    if (!targetState) return;

    // Restore all state from snapshot
    setDocWidth(targetState.docWidth);
    setDocHeight(targetState.docHeight);
    setLayers(targetState.layers.map((l) => ({ ...l })));
    setActiveLayerId(targetState.activeLayerId);
    // Clone so CanvasContainer gets a new Map reference and re-renders
    setLayerCanvasesMap(cloneCanvasesMap(targetState.canvases));

    historyIndexRef.current = targetIdx;
    setHistoryIndex(targetIdx);

    invokeTauri('undo');
  }, []); // stable – reads only refs

  // ── TRUE REDO ─────────────────────────────────────────────────────────────
  const handleRedo = useCallback(() => {
    const currentIdx = historyIndexRef.current;
    const stack = historyStackRef.current;
    if (currentIdx >= stack.length - 1) return;

    const targetIdx = currentIdx + 1;
    const targetState = stack[targetIdx];
    if (!targetState) return;

    setDocWidth(targetState.docWidth);
    setDocHeight(targetState.docHeight);
    setLayers(targetState.layers.map((l) => ({ ...l })));
    setActiveLayerId(targetState.activeLayerId);
    setLayerCanvasesMap(cloneCanvasesMap(targetState.canvases));

    historyIndexRef.current = targetIdx;
    setHistoryIndex(targetIdx);

    invokeTauri('redo');
  }, []); // stable – reads only refs

  // ── Jump to arbitrary history step (from HistoryPanel click) ─────────────
  // FIXED: panel index is relative to historyStack[0..historyStack.length-1],
  // but HistoryPanel shows undoSteps (0..historyIndex) with local idx 0-based.
  // We map: global stack index = idx (since undoSteps = historyStack.slice(0, historyIndex+1))
  const handleJumpToHistory = useCallback((panelIdx: number) => {
    // panelIdx is the index into undoSteps[], which maps 1:1 to historyStack[panelIdx]
    const stack = historyStackRef.current;
    if (panelIdx < 0 || panelIdx >= stack.length) return;
    const targetState = stack[panelIdx];
    if (!targetState) return;

    setDocWidth(targetState.docWidth);
    setDocHeight(targetState.docHeight);
    setLayers(targetState.layers.map((l) => ({ ...l })));
    setActiveLayerId(targetState.activeLayerId);
    setLayerCanvasesMap(cloneCanvasesMap(targetState.canvases));

    historyIndexRef.current = panelIdx;
    setHistoryIndex(panelIdx);
  }, []);

  // ── Color utilities ───────────────────────────────────────────────────────
  const handleSwapColors = useCallback(() => {
    const tmp = foregroundColor;
    setForegroundColor(backgroundColor);
    setBackgroundColor(tmp);
    invokeTauri('set_colors', { fg: backgroundColor, bg: tmp });
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [foregroundColor, backgroundColor]);

  const handleResetColors = useCallback(() => {
    const fg = { red: 0, green: 0, blue: 0, alpha: 1 };
    const bg = { red: 1, green: 1, blue: 1, alpha: 1 };
    setForegroundColor(fg);
    setBackgroundColor(bg);
    invokeTauri('set_colors', { fg, bg });
  }, []);

  const handleSelectTool = useCallback((tool: ToolKind) => {
    setActiveTool(tool);
    invokeTauri('set_active_tool', { tool });
  }, []);

  const handleSave = useCallback(() => {
    invokeTauri('save_project');
  }, []);

  // ── Filter / Adjustment application (direct pixel transforms) ─────────────
  const handleApplyFilter = useCallback((kind: string, settings?: any) => {
    const lCanvas = layerCanvasesMap.get(activeLayerId);
    if (!lCanvas) return;
    const ctx = lCanvas.getContext('2d');
    if (!ctx) return;

    const imgData = ctx.getImageData(0, 0, lCanvas.width, lCanvas.height);
    const data = imgData.data;

    if (kind === 'Invert') {
      for (let i = 0; i < data.length; i += 4) {
        data[i] = 255 - data[i];
        data[i + 1] = 255 - data[i + 1];
        data[i + 2] = 255 - data[i + 2];
      }
    } else if (kind === 'Black & White') {
      for (let i = 0; i < data.length; i += 4) {
        const gray = 0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2];
        data[i] = gray; data[i + 1] = gray; data[i + 2] = gray;
      }
    } else if (kind === 'Add Noise') {
      const amt = (settings?.amount || 15) * 2.55;
      for (let i = 0; i < data.length; i += 4) {
        const noise = (Math.random() - 0.5) * amt;
        data[i] = Math.min(255, Math.max(0, data[i] + noise));
        data[i + 1] = Math.min(255, Math.max(0, data[i + 1] + noise));
        data[i + 2] = Math.min(255, Math.max(0, data[i + 2] + noise));
      }
    } else if (kind === 'Exposure') {
      const exp = Math.pow(2, settings?.exposure || 0.5);
      for (let i = 0; i < data.length; i += 4) {
        data[i] = Math.min(255, Math.max(0, data[i] * exp));
        data[i + 1] = Math.min(255, Math.max(0, data[i + 1] * exp));
        data[i + 2] = Math.min(255, Math.max(0, data[i + 2] * exp));
      }
    } else if (kind === 'Hue/Saturation') {
      // Simple hue-rotation via HSL conversion
      const hueShift = (settings?.hue || 0) / 360;
      const satMult = 1 + (settings?.saturation || 0) / 100;
      const lightAdd = (settings?.lightness || 0) / 100;
      const rgbToHsl = (r: number, g: number, b: number) => {
        r /= 255; g /= 255; b /= 255;
        const max = Math.max(r, g, b), min = Math.min(r, g, b);
        let h = 0, s = 0, l = (max + min) / 2;
        if (max !== min) {
          const d = max - min;
          s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
          switch (max) {
            case r: h = (g - b) / d + (g < b ? 6 : 0); break;
            case g: h = (b - r) / d + 2; break;
            case b: h = (r - g) / d + 4; break;
          }
          h /= 6;
        }
        return [h, s, l];
      };
      const hue2rgb = (p: number, q: number, t: number) => {
        if (t < 0) t += 1; if (t > 1) t -= 1;
        if (t < 1/6) return p + (q - p) * 6 * t;
        if (t < 1/2) return q;
        if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
        return p;
      };
      const hslToRgb = (h: number, s: number, l: number) => {
        if (s === 0) { const v = Math.round(l * 255); return [v, v, v]; }
        const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
        const p = 2 * l - q;
        return [
          Math.round(hue2rgb(p, q, h + 1/3) * 255),
          Math.round(hue2rgb(p, q, h) * 255),
          Math.round(hue2rgb(p, q, h - 1/3) * 255),
        ];
      };
      for (let i = 0; i < data.length; i += 4) {
        let [h, s, l] = rgbToHsl(data[i], data[i + 1], data[i + 2]);
        h = (h + hueShift) % 1;
        s = Math.min(1, Math.max(0, s * satMult));
        l = Math.min(1, Math.max(0, l + lightAdd));
        const [r, g, b] = hslToRgb(h, s, l);
        data[i] = r; data[i + 1] = g; data[i + 2] = b;
      }
    } else if (kind === 'Levels') {
      const inBlack = settings?.inputBlack ?? 0;
      const inWhite = settings?.inputWhite ?? 255;
      const gamma = settings?.gamma ?? 1.0;
      const outBlack = settings?.outputBlack ?? 0;
      const outWhite = settings?.outputWhite ?? 255;
      const range = inWhite - inBlack || 1;
      for (let i = 0; i < data.length; i += 4) {
        for (let ch = 0; ch < 3; ch++) {
          let v = (Math.min(255, Math.max(0, data[i + ch] - inBlack)) / range);
          v = Math.pow(v, 1 / gamma);
          data[i + ch] = Math.min(255, Math.max(0, Math.round(outBlack + v * (outWhite - outBlack))));
        }
      }
    } else if (kind === 'Curves') {
      // Linearize the curve points array into a 256-entry LUT
      const pts: [number, number][] = settings?.points || [[0, 0], [255, 255]];
      const lut = new Uint8Array(256);
      for (let x = 0; x < 256; x++) {
        // Find surrounding control points and lerp
        let y = x;
        for (let j = 0; j < pts.length - 1; j++) {
          const [x0, y0] = pts[j];
          const [x1, y1] = pts[j + 1];
          if (x >= x0 && x <= x1) {
            const t = (x - x0) / (x1 - x0 || 1);
            y = Math.round(y0 + t * (y1 - y0));
            break;
          }
        }
        lut[x] = Math.min(255, Math.max(0, y));
      }
      for (let i = 0; i < data.length; i += 4) {
        data[i] = lut[data[i]];
        data[i + 1] = lut[data[i + 1]];
        data[i + 2] = lut[data[i + 2]];
      }
    }

    ctx.putImageData(imgData, 0, 0);

    const newMap = new Map(layerCanvasesMap);
    newMap.set(activeLayerId, lCanvas);
    setLayerCanvasesMap(newMap);

    pushHistorySnapshot(kind, newMap);
    invokeTauri('apply_filter_command', { kind, settings: settings || {} });
  }, [activeLayerId, layerCanvasesMap, pushHistorySnapshot]);

  const handleOpenFilterOrAdj = useCallback((kind: string) => {
    if (kind === 'Curves') setCurvesOpen(true);
    else if (kind === 'Levels') setLevelsOpen(true);
    else if (kind === 'Hue/Saturation') setHueSatOpen(true);
    else if (kind === 'Camera Raw') setCameraRawOpen(true);
    else if (kind === 'Invert' || kind === 'Black & White') handleApplyFilter(kind);
    else setActiveFilterKind(kind);
  }, [handleApplyFilter]);

  // ── Export full composite ─────────────────────────────────────────────────
  const handleExport = useCallback((format: string, quality: number) => {
    const exportCanvas = document.createElement('canvas');
    exportCanvas.width = docWidth;
    exportCanvas.height = docHeight;
    const ctx = exportCanvas.getContext('2d');
    if (!ctx) return;

    if (format === 'jpeg') { ctx.fillStyle = '#ffffff'; ctx.fillRect(0, 0, docWidth, docHeight); }

    for (const layer of layers) {
      if (!layer.isVisible) continue;
      const lCanvas = layerCanvasesMap.get(layer.id);
      if (!lCanvas) continue;
      ctx.save();
      ctx.globalAlpha = layer.opacity;
      // Draw at exact position (1:1 size – layer canvas already matches document)
      ctx.drawImage(lCanvas, 0, 0);
      ctx.restore();
    }

    const mime = format === 'jpeg' ? 'image/jpeg' : format === 'png' ? 'image/png' : 'image/webp';
    const dataUrl = exportCanvas.toDataURL(mime, quality / 100);
    const link = document.createElement('a');
    link.download = `${tabs[0]?.name || 'Artwork'}.${format === 'jpeg' ? 'jpg' : format}`;
    link.href = dataUrl;
    link.click();
    invokeTauri('export_image', { format, quality });
  }, [docWidth, docHeight, layers, layerCanvasesMap, tabs]);

  // ── Keyboard shortcuts ────────────────────────────────────────────────────
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement || e.target instanceof HTMLTextAreaElement) return;
      const ctrl = e.ctrlKey || e.metaKey;

      if (ctrl && e.key.toLowerCase() === 'z') {
        e.preventDefault();
        if (e.shiftKey) handleRedo(); else handleUndo();
      } else if (ctrl && e.key.toLowerCase() === 'y') {
        e.preventDefault(); handleRedo();
      } else if (ctrl && e.key.toLowerCase() === 'o') {
        e.preventDefault(); fileInputRef.current?.click();
      } else if (ctrl && e.key.toLowerCase() === 's') {
        e.preventDefault(); handleSave();
      } else if (ctrl && e.key.toLowerCase() === 'n') {
        e.preventDefault(); setNewDocOpen(true);
      } else if (ctrl && e.key.toLowerCase() === 'a') {
        e.preventDefault(); setSelection({ x: 0, y: 0, width: docWidth, height: docHeight });
      } else if (ctrl && e.key.toLowerCase() === 'd') {
        e.preventDefault(); setSelection(null);
      } else if (!ctrl) {
        switch (e.key.toLowerCase()) {
          case 'b': setActiveTool('brush'); break;
          case 'v': setActiveTool('move'); break;
          case 'm': setActiveTool('marquee_rect'); break;
          case 'l': setActiveTool('lasso'); break;
          case 'w': setActiveTool('wand'); break;
          case 'e': setActiveTool('eraser'); break;
          case 'i': setActiveTool('eyedropper'); break;
          case 'h': setActiveTool('hand'); break;
          case 'x': handleSwapColors(); break;
          case '[': setBrushSettings((s) => ({ ...s, size: Math.max(1, s.size - 5) })); break;
          case ']': setBrushSettings((s) => ({ ...s, size: Math.min(500, s.size + 5) })); break;
        }
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleUndo, handleRedo, handleSave, handleSwapColors, docWidth, docHeight]);

  // ── Layer canvas update (called from CanvasContainer on every brush dab) ──
  const handleLayerCanvasUpdate = useCallback((layerId: string, updatedCanvas: HTMLCanvasElement) => {
    setLayerCanvasesMap((prev) => {
      const next = new Map(prev);
      next.set(layerId, updatedCanvas);
      return next;
    });
  }, []);

  // ── Layer add / delete / group helpers ───────────────────────────────────
  const handleAddLayer = useCallback(() => {
    const newId = `layer-${Date.now()}`;
    const newLayer: LayerItem = {
      id: newId,
      name: `Layer ${layers.length + 1}`,
      isVisible: true,
      isGroup: false,
      opacity: 1.0,
      blendMode: 'Normal',
      hasMask: false,
      maskEnabled: false,
      hasEffects: false,
    };
    const newCanvas = document.createElement('canvas');
    newCanvas.width = docWidth;
    newCanvas.height = docHeight;
    const newMap = new Map(layerCanvasesMap);
    newMap.set(newId, newCanvas);
    setLayerCanvasesMap(newMap);
    const nextLayers = [newLayer, ...layers];
    setLayers(nextLayers);
    setActiveLayerId(newId);
    pushHistorySnapshot('New Layer', newMap, nextLayers, docWidth, docHeight, newId);
    invokeTauri('add_layer');
  }, [docWidth, docHeight, layers, layerCanvasesMap, pushHistorySnapshot]);

  const handleAddGroup = useCallback(() => {
    const newId = `group-${Date.now()}`;
    const newGroup: LayerItem = {
      id: newId,
      name: `Group ${layers.length + 1}`,
      isVisible: true,
      isGroup: true,
      opacity: 1.0,
      blendMode: 'Normal',
      hasMask: false,
      maskEnabled: false,
      hasEffects: false,
    };
    const nextLayers = [newGroup, ...layers];
    setLayers(nextLayers);
    setActiveLayerId(newId);
    pushHistorySnapshot('New Group', undefined, nextLayers, docWidth, docHeight, newId);
    invokeTauri('add_group');
  }, [docWidth, docHeight, layers, pushHistorySnapshot]);

  const handleDeleteLayer = useCallback((id: string) => {
    if (layers.length <= 1) return;
    const nextLayers = layers.filter((l) => l.id !== id);
    const nextActive = nextLayers[0]?.id || '';
    setLayers(nextLayers);
    setActiveLayerId(nextActive);
    const newMap = new Map(layerCanvasesMap);
    newMap.delete(id);
    setLayerCanvasesMap(newMap);
    pushHistorySnapshot('Delete Layer', newMap, nextLayers, docWidth, docHeight, nextActive);
    invokeTauri('delete_layer', { id });
  }, [docWidth, docHeight, layers, layerCanvasesMap, pushHistorySnapshot]);

  const handleToggleVisibility = useCallback((id: string) => {
    const nextLayers = layers.map((l) => (l.id === id ? { ...l, isVisible: !l.isVisible } : l));
    setLayers(nextLayers);
    pushHistorySnapshot('Toggle Layer Visibility', undefined, nextLayers);
    invokeTauri('toggle_layer_visibility', { id });
  }, [layers, pushHistorySnapshot]);

  const handleChangeBlendMode = useCallback((id: string, blendMode: BlendMode) => {
    const nextLayers = layers.map((l) => (l.id === id ? { ...l, blendMode } : l));
    setLayers(nextLayers);
    pushHistorySnapshot(`Blend Mode: ${blendMode}`, undefined, nextLayers);
    invokeTauri('set_layer_blend_mode', { id, blendMode });
  }, [layers, pushHistorySnapshot]);

  const handleChangeOpacity = useCallback((id: string, opacity: number) => {
    const nextLayers = layers.map((l) => (l.id === id ? { ...l, opacity } : l));
    setLayers(nextLayers);
    pushHistorySnapshot('Change Layer Opacity', undefined, nextLayers);
    invokeTauri('set_layer_opacity', { id, opacity });
  }, [layers, pushHistorySnapshot]);

  // ── Derive undo / redo step lists for HistoryPanel ───────────────────────
  // undoSteps[i] maps to historyStack[i] – same index, no offset needed.
  const undoSteps = historyStack.slice(0, historyIndex + 1).map((e) => e.name);
  const redoSteps = historyStack.slice(historyIndex + 1).map((e) => e.name);

  const isWindowDragging = dragCounter > 0;

  // ── Render ────────────────────────────────────────────────────────────────
  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100vw', height: '100vh', overflow: 'hidden' }}>
      {/* Hidden native file picker */}
      <input
        type="file"
        ref={fileInputRef}
        accept="image/*,.comp,.psd,.psb,.tiff,.tif,.svg,.raw,.heic,.heif"
        style={{ display: 'none' }}
        onChange={handleFileInputChange}
      />

      {/* 1. Menu Bar */}
      <MenuBar
        onNew={() => setNewDocOpen(true)}
        onOpen={() => fileInputRef.current?.click()}
        onSave={handleSave}
        onExport={() => setExportOpen(true)}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onCut={() => invokeTauri('clipboard_cut')}
        onCopy={() => invokeTauri('clipboard_copy')}
        onPaste={() => invokeTauri('clipboard_paste')}
        onSelectAll={() => setSelection({ x: 0, y: 0, width: docWidth, height: docHeight })}
        onDeselect={() => setSelection(null)}
        onInvertSelection={() => setSelection({ x: 0, y: 0, width: docWidth, height: docHeight })}
        onOpenFilter={handleOpenFilterOrAdj}
        onOpenAdjustment={handleOpenFilterOrAdj}
      />

      {/* 2. Tool Options Bar */}
      <ToolOptionsBar
        activeTool={activeTool}
        brushSettings={brushSettings}
        onUpdateBrush={(s) => {
          setBrushSettings((prev) => ({ ...prev, ...s }));
          invokeTauri('set_brush_settings', { settings: { ...brushSettings, ...s } });
        }}
      />

      {/* 3. Document Tabs */}
      <DocumentTabs
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={setActiveTabId}
        onCloseTab={(id) => {
          if (tabs.length > 1) {
            setTabs(tabs.filter((t) => t.id !== id));
            if (activeTabId === id) setActiveTabId(tabs[0].id);
          }
        }}
      />

      {/* 4. Main Work Area */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden', position: 'relative' }}>
        {/* Left Tools Palette */}
        <ToolsPalette
          activeTool={activeTool}
          onSelectTool={handleSelectTool}
          foregroundColor={foregroundColor}
          backgroundColor={backgroundColor}
          onSwapColors={handleSwapColors}
          onResetColors={handleResetColors}
          onOpenColorPicker={() => {}}
        />

        {/* Center Canvas */}
        <CanvasContainer
          documentName={tabs.find((t) => t.id === activeTabId)?.name || 'Untitled'}
          docWidth={docWidth}
          docHeight={docHeight}
          layers={layers}
          activeLayerId={activeLayerId}
          activeTool={activeTool}
          brushSettings={brushSettings}
          foregroundColor={foregroundColor}
          backgroundColor={backgroundColor}
          zoom={zoom}
          onZoomChange={setZoom}
          panX={panX}
          panY={panY}
          onPanChange={(px, py) => { setPanX(px); setPanY(py); }}
          layerCanvases={layerCanvasesMap}
          onLayerCanvasUpdate={handleLayerCanvasUpdate}
          onColorSample={setForegroundColor}
          selection={selection}
          onSelectionChange={setSelection}
          onDropFiles={(files) => { if (files.length > 0) openImageFile(files[0]); }}
          onAddHistoryStep={(name) => pushHistorySnapshot(name)}
        />

        {/* Right Sidebar */}
        <div
          style={{
            width: 280,
            display: 'flex',
            flexDirection: 'column',
            borderLeft: '1px solid var(--border-color)',
            backgroundColor: 'var(--bg-panel)',
          }}
        >
          <AdjustmentsPanel onAddAdjustment={handleOpenFilterOrAdj} />

          <div style={{ flex: 1, overflow: 'hidden' }}>
            <LayersPanel
              layers={layers}
              activeLayerId={activeLayerId}
              onSelectLayer={setActiveLayerId}
              onToggleVisibility={handleToggleVisibility}
              onChangeBlendMode={handleChangeBlendMode}
              onChangeOpacity={handleChangeOpacity}
              onAddLayer={handleAddLayer}
              onAddGroup={handleAddGroup}
              onDeleteLayer={handleDeleteLayer}
            />
          </div>

          <HistoryPanel
            undoSteps={undoSteps}
            redoSteps={redoSteps}
            onJumpToState={handleJumpToHistory}
          />
        </div>
      </div>

      {/* 5. Status Bar */}
      <StatusBar width={docWidth} height={docHeight} zoom={zoom} />

      {/* Modals */}
      <CurvesModal
        isOpen={isCurvesOpen}
        onClose={() => setCurvesOpen(false)}
        onApply={(data) => handleApplyFilter('Curves', data)}
      />
      <LevelsModal
        isOpen={isLevelsOpen}
        onClose={() => setLevelsOpen(false)}
        onApply={(data) => handleApplyFilter('Levels', data)}
      />
      <HueSaturationModal
        isOpen={isHueSatOpen}
        onClose={() => setHueSatOpen(false)}
        onApply={(data) => handleApplyFilter('Hue/Saturation', data)}
      />
      <FilterModal
        filterKind={activeFilterKind}
        onClose={() => setActiveFilterKind(null)}
        onApply={(kind, settings) => handleApplyFilter(kind, settings)}
      />
      <CameraRawModal
        isOpen={isCameraRawOpen}
        onClose={() => setCameraRawOpen(false)}
        onApply={(settings) => handleApplyFilter('Camera Raw', settings)}
      />
      <NewDocModal
        isOpen={isNewDocOpen}
        onClose={() => setNewDocOpen(false)}
        onCreate={(name, w, h) => {
          setDocWidth(w);
          setDocHeight(h);

          const newId = `layer-${Date.now()}`;
          const newBg: LayerItem = {
            id: newId,
            name: 'Background',
            isVisible: true,
            isGroup: false,
            opacity: 1.0,
            blendMode: 'Normal',
            hasMask: false,
            maskEnabled: false,
            hasEffects: false,
          };
          const newCanvas = document.createElement('canvas');
          newCanvas.width = w;
          newCanvas.height = h;
          const ctx = newCanvas.getContext('2d');
          if (ctx) { ctx.fillStyle = '#ffffff'; ctx.fillRect(0, 0, w, h); }

          const newMap = new Map<string, HTMLCanvasElement>();
          newMap.set(newId, newCanvas);
          setLayerCanvasesMap(newMap);
          setLayers([newBg]);
          setActiveLayerId(newId);

          const availW = window.innerWidth - 360;
          const availH = window.innerHeight - 150;
          const fitZ = Math.min(availW / w, availH / h, 0.8);
          setZoom(fitZ);
          setPanX(Math.round((availW - w * fitZ) / 2));
          setPanY(Math.round((availH - h * fitZ) / 2));

          setTabs([{ id: `doc-${Date.now()}`, name, isDirty: false }, ...tabs]);
          setSelection(null);

          const entry: HistoryEntry = {
            name: 'New Document',
            docWidth: w,
            docHeight: h,
            layers: [newBg],
            activeLayerId: newId,
            canvases: cloneCanvasesMap(newMap),
          };
          historyStackRef.current = [entry];
          historyIndexRef.current = 0;
          setHistoryStack([entry]);
          setHistoryIndex(0);
          invokeTauri('new_document', { name, width: w, height: h });
        }}
      />
      <ExportModal
        isOpen={isExportOpen}
        onClose={() => setExportOpen(false)}
        onExport={handleExport}
      />

      {/* Full-screen drag-and-drop overlay (counter-based, no flicker) */}
      {isWindowDragging && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'rgba(0, 120, 215, 0.45)',
            border: '4px dashed #0078d7',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#ffffff',
            fontSize: 22,
            fontWeight: 'bold',
            zIndex: 99999,
            pointerEvents: 'none',
            backdropFilter: 'blur(4px)',
          }}
        >
          <div
            style={{
              padding: '24px 48px',
              backgroundColor: 'rgba(20, 20, 20, 0.95)',
              borderRadius: 12,
              boxShadow: '0 12px 40px rgba(0,0,0,0.8)',
            }}
          >
            📁 Drop image anywhere to open in Compositor
          </div>
        </div>
      )}
    </div>
  );
};
