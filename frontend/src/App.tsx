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

interface HistoryEntry {
  name: string;
  docWidth: number;
  docHeight: number;
  layers: LayerItem[];
  activeLayerId: string;
  canvases: Map<string, HTMLCanvasElement>;
}

// Helpers to clone canvases for immutable history snapshots
const cloneCanvas = (src: HTMLCanvasElement): HTMLCanvasElement => {
  const copy = document.createElement('canvas');
  copy.width = src.width;
  copy.height = src.height;
  const ctx = copy.getContext('2d');
  if (ctx) {
    ctx.drawImage(src, 0, 0);
  }
  return copy;
};

const cloneCanvasesMap = (map: Map<string, HTMLCanvasElement>): Map<string, HTMLCanvasElement> => {
  const copy = new Map<string, HTMLCanvasElement>();
  map.forEach((c, id) => {
    copy.set(id, cloneCanvas(c));
  });
  return copy;
};

export const App: React.FC = () => {
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Active Tool & Settings
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

  // Color Swatches
  const [foregroundColor, setForegroundColor] = useState<PaletteColor>({
    red: 0,
    green: 0,
    blue: 0,
    alpha: 1,
  });
  const [backgroundColor, setBackgroundColor] = useState<PaletteColor>({
    red: 1,
    green: 1,
    blue: 1,
    alpha: 1,
  });

  // Documents & Dimensions
  const [tabs, setTabs] = useState<{ id: string; name: string; isDirty?: boolean }[]>([
    { id: 'doc-1', name: 'Untitled-1', isDirty: false },
  ]);
  const [activeTabId, setActiveTabId] = useState('doc-1');
  const [docWidth, setDocWidth] = useState(1920);
  const [docHeight, setDocHeight] = useState(1080);
  const [zoom, setZoom] = useState(0.5);
  const [panX, setPanX] = useState(80);
  const [panY, setPanY] = useState(40);

  // Selection
  const [selection, setSelection] = useState<{ x: number; y: number; width: number; height: number } | null>(null);

  // Drag and drop state
  const [isWindowDragging, setIsWindowDragging] = useState(false);

  // Layers State
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

  // Layer Bitmaps
  const [layerCanvasesMap, setLayerCanvasesMap] = useState<Map<string, HTMLCanvasElement>>(new Map());

  // Real Historical Snapshots for True Undo / Redo
  const [historyStack, setHistoryStack] = useState<HistoryEntry[]>([]);
  const [historyIndex, setHistoryIndex] = useState<number>(-1);

  // Modals Open State
  const [isCurvesOpen, setCurvesOpen] = useState(false);
  const [isLevelsOpen, setLevelsOpen] = useState(false);
  const [isHueSatOpen, setHueSatOpen] = useState(false);
  const [activeFilterKind, setActiveFilterKind] = useState<string | null>(null);
  const [isCameraRawOpen, setCameraRawOpen] = useState(false);
  const [isNewDocOpen, setNewDocOpen] = useState(false);
  const [isExportOpen, setExportOpen] = useState(false);

  // IPC helpers with Tauri
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

  // Push a state snapshot to the history stack
  const pushHistorySnapshot = useCallback(
    (name: string, customMap?: Map<string, HTMLCanvasElement>, customLayers?: LayerItem[]) => {
      const mapToSave = customMap || layerCanvasesMap;
      const layersToSave = customLayers || layers;
      const entry: HistoryEntry = {
        name,
        docWidth,
        docHeight,
        layers: layersToSave.map((l) => ({ ...l })),
        activeLayerId,
        canvases: cloneCanvasesMap(mapToSave),
      };

      setHistoryStack((prev) => {
        const trimmed = prev.slice(0, historyIndex + 1);
        const next = [...trimmed, entry];
        if (next.length > 50) next.shift();
        return next;
      });
      setHistoryIndex((prev) => Math.min(prev + 1, 49));
    },
    [docWidth, docHeight, layers, activeLayerId, layerCanvasesMap, historyIndex]
  );

  // Initialize canvas on app startup
  useEffect(() => {
    const bgCanvas = document.createElement('canvas');
    bgCanvas.width = 1920;
    bgCanvas.height = 1080;
    const ctx = bgCanvas.getContext('2d');
    if (ctx) {
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(0, 0, 1920, 1080);
    }
    const map = new Map();
    map.set('layer-1', bgCanvas);
    setLayerCanvasesMap(map);

    // Initial center
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
    setHistoryStack([initialEntry]);
    setHistoryIndex(0);
  }, []);

  // Image Loading Function (Core Solution for Bringing in Images)
  const openImageFile = useCallback((file: File) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const dataUrl = e.target?.result as string;
      if (!dataUrl) return;

      const img = new Image();
      img.onload = () => {
        const w = img.width;
        const h = img.height;

        setDocWidth(w);
        setDocHeight(h);

        // Create new layer canvas with image pixels
        const lCanvas = document.createElement('canvas');
        lCanvas.width = w;
        lCanvas.height = h;
        const ctx = lCanvas.getContext('2d');
        if (ctx) {
          ctx.drawImage(img, 0, 0);
        }

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

        setTabs([{ id: `doc-${Date.now()}`, name: file.name, isDirty: false }]);
        setSelection(null);

        // Initialize history stack with this new image
        const entry: HistoryEntry = {
          name: `Open ${file.name}`,
          docWidth: w,
          docHeight: h,
          layers: [newLayer],
          activeLayerId: newLayerId,
          canvases: cloneCanvasesMap(newMap),
        };
        setHistoryStack([entry]);
        setHistoryIndex(0);

        // Synchronize with Rust engine
        invokeTauri('open_image_file', { path: (file as any).path || file.name });
      };
      img.src = dataUrl;
    };
    reader.readAsDataURL(file);
  }, []);

  // Handle file input change
  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      openImageFile(e.target.files[0]);
    }
  };

  // Window-Wide Drag and Drop Listener (Automatic Drag and Drop)
  useEffect(() => {
    const handleDragOver = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'copy';
      }
      setIsWindowDragging(true);
    };

    const handleDragEnter = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'copy';
      }
      setIsWindowDragging(true);
    };

    const handleDragLeave = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.clientX <= 0 || e.clientY <= 0 || e.clientX >= window.innerWidth || e.clientY >= window.innerHeight) {
        setIsWindowDragging(false);
      }
    };

    const handleDrop = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsWindowDragging(false);

      if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
        const file = e.dataTransfer.files[0];
        openImageFile(file);
      }
    };

    window.addEventListener('dragover', handleDragOver);
    window.addEventListener('dragenter', handleDragEnter);
    window.addEventListener('dragleave', handleDragLeave);
    window.addEventListener('drop', handleDrop);

    return () => {
      window.removeEventListener('dragover', handleDragOver);
      window.removeEventListener('dragenter', handleDragEnter);
      window.removeEventListener('dragleave', handleDragLeave);
      window.removeEventListener('drop', handleDrop);
    };
  }, [openImageFile]);

  // Global Paste listener (paste image from clipboard)
  useEffect(() => {
    const handlePaste = (e: ClipboardEvent) => {
      if (e.clipboardData && e.clipboardData.items) {
        for (let i = 0; i < e.clipboardData.items.length; i++) {
          const item = e.clipboardData.items[i];
          if (item.type.indexOf('image') !== -1) {
            const file = item.getAsFile();
            if (file) {
              openImageFile(file);
              break;
            }
          }
        }
      }
    };
    window.addEventListener('paste', handlePaste);
    return () => window.removeEventListener('paste', handlePaste);
  }, [openImageFile]);

  // TRUE UNDO / REDO: Restores physical canvas pixels and layer states
  const handleUndo = useCallback(() => {
    if (historyIndex <= 0) return;
    const targetIdx = historyIndex - 1;
    const targetState = historyStack[targetIdx];
    if (!targetState) return;

    setDocWidth(targetState.docWidth);
    setDocHeight(targetState.docHeight);
    setLayers(targetState.layers.map((l) => ({ ...l })));
    setActiveLayerId(targetState.activeLayerId);
    setLayerCanvasesMap(cloneCanvasesMap(targetState.canvases));
    setHistoryIndex(targetIdx);

    invokeTauri('undo');
  }, [historyIndex, historyStack]);

  const handleRedo = useCallback(() => {
    if (historyIndex >= historyStack.length - 1) return;
    const targetIdx = historyIndex + 1;
    const targetState = historyStack[targetIdx];
    if (!targetState) return;

    setDocWidth(targetState.docWidth);
    setDocHeight(targetState.docHeight);
    setLayers(targetState.layers.map((l) => ({ ...l })));
    setActiveLayerId(targetState.activeLayerId);
    setLayerCanvasesMap(cloneCanvasesMap(targetState.canvases));
    setHistoryIndex(targetIdx);

    invokeTauri('redo');
  }, [historyIndex, historyStack]);

  const handleJumpToHistory = (idx: number) => {
    if (idx < 0 || idx >= historyStack.length) return;
    const targetState = historyStack[idx];
    if (!targetState) return;

    setDocWidth(targetState.docWidth);
    setDocHeight(targetState.docHeight);
    setLayers(targetState.layers.map((l) => ({ ...l })));
    setActiveLayerId(targetState.activeLayerId);
    setLayerCanvasesMap(cloneCanvasesMap(targetState.canvases));
    setHistoryIndex(idx);
  };

  const handleSave = () => {
    invokeTauri('save_project');
  };

  const handleSwapColors = () => {
    const tmp = foregroundColor;
    setForegroundColor(backgroundColor);
    setBackgroundColor(tmp);
    invokeTauri('set_colors', { fg: backgroundColor, bg: tmp });
  };

  const handleResetColors = () => {
    const fg = { red: 0, green: 0, blue: 0, alpha: 1 };
    const bg = { red: 1, green: 1, blue: 1, alpha: 1 };
    setForegroundColor(fg);
    setBackgroundColor(bg);
    invokeTauri('set_colors', { fg, bg });
  };

  const handleSelectTool = (tool: ToolKind) => {
    setActiveTool(tool);
    invokeTauri('set_active_tool', { tool });
  };

  // Filter and Adjustment application directly on pixel buffer
  const handleApplyFilter = (kind: string, settings?: any) => {
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
        data[i] = gray;
        data[i + 1] = gray;
        data[i + 2] = gray;
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
    }

    ctx.putImageData(imgData, 0, 0);

    const newMap = new Map(layerCanvasesMap);
    newMap.set(activeLayerId, lCanvas);
    setLayerCanvasesMap(newMap);

    pushHistorySnapshot(kind, newMap);
    invokeTauri('apply_filter_command', { kind, settings: settings || {} });
  };

  const handleOpenFilterOrAdj = (kind: string) => {
    if (kind === 'Curves') setCurvesOpen(true);
    else if (kind === 'Levels') setLevelsOpen(true);
    else if (kind === 'Hue/Saturation') setHueSatOpen(true);
    else if (kind === 'Camera Raw') setCameraRawOpen(true);
    else if (kind === 'Invert' || kind === 'Black & White') {
      handleApplyFilter(kind);
    } else {
      setActiveFilterKind(kind);
    }
  };

  // Export full composite image
  const handleExport = (format: string, quality: number) => {
    const exportCanvas = document.createElement('canvas');
    exportCanvas.width = docWidth;
    exportCanvas.height = docHeight;
    const ctx = exportCanvas.getContext('2d');
    if (!ctx) return;

    if (format === 'jpeg') {
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(0, 0, docWidth, docHeight);
    }

    // Composite all visible layers in stack order
    for (const layer of layers) {
      if (!layer.isVisible) continue;
      const lCanvas = layerCanvasesMap.get(layer.id);
      if (!lCanvas) continue;
      ctx.globalAlpha = layer.opacity;
      ctx.drawImage(lCanvas, 0, 0, docWidth, docHeight);
    }

    const mime = format === 'png' ? 'image/png' : 'image/jpeg';
    const dataUrl = exportCanvas.toDataURL(mime, quality / 100);

    const link = document.createElement('a');
    link.download = `${tabs[0]?.name || 'Artwork'}.${format === 'jpeg' ? 'jpg' : format}`;
    link.href = dataUrl;
    link.click();

    invokeTauri('export_image', { format, quality });
  };

  // Keyboard Shortcuts (Photoshop parity)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) return;

      const ctrlOrCmd = e.ctrlKey || e.metaKey;

      if (ctrlOrCmd && e.key.toLowerCase() === 'z') {
        if (e.shiftKey) handleRedo();
        else handleUndo();
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 'y') {
        handleRedo();
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 'o') {
        fileInputRef.current?.click();
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 's') {
        handleSave();
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 'n') {
        setNewDocOpen(true);
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 'a') {
        setSelection({ x: 0, y: 0, width: docWidth, height: docHeight });
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 'd') {
        setSelection(null);
        e.preventDefault();
      } else if (e.key.toLowerCase() === 'b') {
        setActiveTool('brush');
      } else if (e.key.toLowerCase() === 'v') {
        setActiveTool('move');
      } else if (e.key.toLowerCase() === 'm') {
        setActiveTool('marquee_rect');
      } else if (e.key.toLowerCase() === 'l') {
        setActiveTool('lasso');
      } else if (e.key.toLowerCase() === 'w') {
        setActiveTool('wand');
      } else if (e.key.toLowerCase() === 'e') {
        setActiveTool('eraser');
      } else if (e.key.toLowerCase() === 'i') {
        setActiveTool('eyedropper');
      } else if (e.key.toLowerCase() === 'x') {
        handleSwapColors();
      } else if (e.key.toLowerCase() === 'd') {
        handleResetColors();
      } else if (e.key === '[') {
        setBrushSettings((s) => ({ ...s, size: Math.max(1, s.size - 5) }));
      } else if (e.key === ']') {
        setBrushSettings((s) => ({ ...s, size: Math.min(500, s.size + 5) }));
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleUndo, handleRedo, docWidth, docHeight]);

  const handleLayerCanvasUpdate = (layerId: string, updatedCanvas: HTMLCanvasElement) => {
    const newMap = new Map(layerCanvasesMap);
    newMap.set(layerId, updatedCanvas);
    setLayerCanvasesMap(newMap);
  };

  // Derive undo and redo lists for HistoryPanel
  const undoSteps = historyStack.slice(0, historyIndex + 1).map((e) => e.name);
  const redoSteps = historyStack.slice(historyIndex + 1).map((e) => e.name);

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        width: '100vw',
        height: '100vh',
        overflow: 'hidden',
      }}
    >
      {/* Hidden File Input for Native Open Dialog */}
      <input
        type="file"
        ref={fileInputRef}
        accept="image/*,.comp,.psd,.psb,.tiff,.tif,.svg,.raw"
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
            if (activeTabId === id) {
              setActiveTabId(tabs[0].id);
            }
          }
        }}
      />

      {/* 4. Main Work Area (Tools + Canvas + Right Sidebar) */}
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
          onPanChange={(px, py) => {
            setPanX(px);
            setPanY(py);
          }}
          layerCanvases={layerCanvasesMap}
          onLayerCanvasUpdate={handleLayerCanvasUpdate}
          onColorSample={setForegroundColor}
          selection={selection}
          onSelectionChange={setSelection}
          onDropFiles={(files) => {
            if (files.length > 0) openImageFile(files[0]);
          }}
          onAddHistoryStep={(name) => {
            pushHistorySnapshot(name);
          }}
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
          {/* Adjustments Presets */}
          <AdjustmentsPanel onAddAdjustment={handleOpenFilterOrAdj} />

          {/* Layers Panel */}
          <div style={{ flex: 1, overflow: 'hidden' }}>
            <LayersPanel
              layers={layers}
              activeLayerId={activeLayerId}
              onSelectLayer={setActiveLayerId}
              onToggleVisibility={(id) => {
                const nextLayers = layers.map((l) => (l.id === id ? { ...l, isVisible: !l.isVisible } : l));
                setLayers(nextLayers);
                pushHistorySnapshot('Toggle Layer Visibility', undefined, nextLayers);
                invokeTauri('toggle_layer_visibility', { id });
              }}
              onChangeBlendMode={(id, blendMode: BlendMode) => {
                const nextLayers = layers.map((l) => (l.id === id ? { ...l, blendMode } : l));
                setLayers(nextLayers);
                pushHistorySnapshot(`Blend Mode: ${blendMode}`, undefined, nextLayers);
                invokeTauri('set_layer_blend_mode', { id, blendMode });
              }}
              onChangeOpacity={(id, opacity) => {
                const nextLayers = layers.map((l) => (l.id === id ? { ...l, opacity } : l));
                setLayers(nextLayers);
                pushHistorySnapshot('Change Layer Opacity', undefined, nextLayers);
                invokeTauri('set_layer_opacity', { id, opacity });
              }}
              onAddLayer={() => {
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

                pushHistorySnapshot('New Layer', newMap, nextLayers);
                invokeTauri('add_layer');
              }}
              onAddGroup={() => {
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
                pushHistorySnapshot('New Group', undefined, nextLayers);
                invokeTauri('add_group');
              }}
              onDeleteLayer={(id) => {
                if (layers.length <= 1) return;
                const nextLayers = layers.filter((l) => l.id !== id);
                setLayers(nextLayers);
                setActiveLayerId(nextLayers[0]?.id || '');

                const newMap = new Map(layerCanvasesMap);
                newMap.delete(id);
                setLayerCanvasesMap(newMap);

                pushHistorySnapshot('Delete Layer', newMap, nextLayers);
                invokeTauri('delete_layer', { id });
              }}
            />
          </div>

          {/* History Panel */}
          <HistoryPanel
            undoSteps={undoSteps}
            redoSteps={redoSteps}
            onJumpToState={handleJumpToHistory}
          />
        </div>
      </div>

      {/* 5. Status Bar */}
      <StatusBar width={docWidth} height={docHeight} zoom={zoom} />

      {/* Modals & Dialogs */}
      <CurvesModal
        isOpen={isCurvesOpen}
        onClose={() => setCurvesOpen(false)}
        onApply={(data) => {
          handleApplyFilter('Curves', data);
        }}
      />

      <LevelsModal
        isOpen={isLevelsOpen}
        onClose={() => setLevelsOpen(false)}
        onApply={(data) => {
          handleApplyFilter('Levels', data);
        }}
      />

      <HueSaturationModal
        isOpen={isHueSatOpen}
        onClose={() => setHueSatOpen(false)}
        onApply={(data) => {
          handleApplyFilter('Hue/Saturation', data);
        }}
      />

      <FilterModal
        filterKind={activeFilterKind}
        onClose={() => setActiveFilterKind(null)}
        onApply={(kind, settings) => {
          handleApplyFilter(kind, settings);
        }}
      />

      <CameraRawModal
        isOpen={isCameraRawOpen}
        onClose={() => setCameraRawOpen(false)}
        onApply={(settings) => {
          handleApplyFilter('Camera Raw', settings);
        }}
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
          if (ctx) {
            ctx.fillStyle = '#ffffff';
            ctx.fillRect(0, 0, w, h);
          }

          const newMap = new Map();
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

      {/* Full-Screen Window Drag & Drop Overlay */}
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
