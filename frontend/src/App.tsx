import React, { useState, useEffect } from 'react';
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

import { ToolKind, LayerItem, BrushSettings, PaletteColor } from './types';

export const App: React.FC = () => {
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

  // Documents & Tabs
  const [tabs, setTabs] = useState<{ id: string; name: string; isDirty?: boolean }[]>([
    { id: 'doc-1', name: 'Untitled-1', isDirty: false },
  ]);
  const [activeTabId, setActiveTabId] = useState('doc-1');
  const [docWidth, setDocWidth] = useState(1920);
  const [docHeight, setDocHeight] = useState(1080);
  const [zoom, setZoom] = useState(1.0);

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
    {
      id: 'layer-2',
      name: 'Layer 1',
      isVisible: true,
      isGroup: false,
      opacity: 1.0,
      blendMode: 'Normal',
      hasMask: false,
      maskEnabled: false,
      hasEffects: false,
    },
  ]);
  const [activeLayerId, setActiveLayerId] = useState('layer-2');

  // History State
  const [undoSteps, setUndoSteps] = useState<string[]>(['Open Document', 'New Layer']);
  const [redoSteps, setRedoSteps] = useState<string[]>([]);

  // Modals Open State
  const [isCurvesOpen, setCurvesOpen] = useState(false);
  const [isLevelsOpen, setLevelsOpen] = useState(false);
  const [isHueSatOpen, setHueSatOpen] = useState(false);
  const [activeFilterKind, setActiveFilterKind] = useState<string | null>(null);
  const [isCameraRawOpen, setCameraRawOpen] = useState(false);
  const [isNewDocOpen, setNewDocOpen] = useState(false);
  const [isExportOpen, setExportOpen] = useState(false);

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
      } else if (ctrlOrCmd && e.key.toLowerCase() === 's') {
        handleSave();
        e.preventDefault();
      } else if (ctrlOrCmd && e.key.toLowerCase() === 'n') {
        setNewDocOpen(true);
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
  }, [undoSteps, redoSteps]);

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

  const handleUndo = () => {
    if (undoSteps.length <= 1) return;
    const last = undoSteps[undoSteps.length - 1];
    setUndoSteps(undoSteps.slice(0, -1));
    setRedoSteps([last, ...redoSteps]);
    invokeTauri('undo');
  };

  const handleRedo = () => {
    if (redoSteps.length === 0) return;
    const next = redoSteps[0];
    setRedoSteps(redoSteps.slice(1));
    setUndoSteps([...undoSteps, next]);
    invokeTauri('redo');
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

  const handleOpenFilterOrAdj = (kind: string) => {
    if (kind === 'Curves') setCurvesOpen(true);
    else if (kind === 'Levels') setLevelsOpen(true);
    else if (kind === 'Hue/Saturation') setHueSatOpen(true);
    else if (kind === 'Camera Raw') setCameraRawOpen(true);
    else if (kind === 'Invert') {
      invokeTauri('apply_filter', { kind: 'Invert', settings: {} });
      setUndoSteps([...undoSteps, 'Invert']);
    } else {
      setActiveFilterKind(kind);
    }
  };

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
      {/* 1. Menu Bar */}
      <MenuBar
        onNew={() => setNewDocOpen(true)}
        onOpen={() => invokeTauri('open_project')}
        onSave={handleSave}
        onExport={() => setExportOpen(true)}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onCut={() => invokeTauri('clipboard_cut')}
        onCopy={() => invokeTauri('clipboard_copy')}
        onPaste={() => invokeTauri('clipboard_paste')}
        onSelectAll={() => invokeTauri('select_all')}
        onDeselect={() => invokeTauri('deselect')}
        onInvertSelection={() => invokeTauri('invert_selection')}
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
          zoom={zoom}
          onZoomChange={setZoom}
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
                setLayers(
                  layers.map((l) => (l.id === id ? { ...l, isVisible: !l.isVisible } : l))
                );
                invokeTauri('toggle_layer_visibility', { id });
              }}
              onChangeBlendMode={(id, blendMode) => {
                setLayers(layers.map((l) => (l.id === id ? { ...l, blendMode } : l)));
                invokeTauri('set_layer_blend_mode', { id, blendMode });
              }}
              onChangeOpacity={(id, opacity) => {
                setLayers(layers.map((l) => (l.id === id ? { ...l, opacity } : l)));
                invokeTauri('set_layer_opacity', { id, opacity });
              }}
              onAddLayer={() => {
                const newId = `layer-${Date.now()}`;
                const newLayer: LayerItem = {
                  id: newId,
                  name: `Layer ${layers.length}`,
                  isVisible: true,
                  isGroup: false,
                  opacity: 1.0,
                  blendMode: 'Normal',
                  hasMask: false,
                  maskEnabled: false,
                  hasEffects: false,
                };
                setLayers([newLayer, ...layers]);
                setActiveLayerId(newId);
                setUndoSteps([...undoSteps, 'New Layer']);
                invokeTauri('add_layer');
              }}
              onAddGroup={() => {
                const newId = `group-${Date.now()}`;
                const newGroup: LayerItem = {
                  id: newId,
                  name: `Group ${layers.length}`,
                  isVisible: true,
                  isGroup: true,
                  opacity: 1.0,
                  blendMode: 'Normal',
                  hasMask: false,
                  maskEnabled: false,
                  hasEffects: false,
                };
                setLayers([newGroup, ...layers]);
                setActiveLayerId(newId);
                setUndoSteps([...undoSteps, 'New Group']);
                invokeTauri('add_group');
              }}
              onDeleteLayer={(id) => {
                if (layers.length <= 1) return;
                setLayers(layers.filter((l) => l.id !== id));
                setActiveLayerId(layers.find((l) => l.id !== id)?.id || '');
                setUndoSteps([...undoSteps, 'Delete Layer']);
                invokeTauri('delete_layer', { id });
              }}
            />
          </div>

          {/* History Panel */}
          <HistoryPanel
            undoSteps={undoSteps}
            redoSteps={redoSteps}
            onJumpToState={(idx) => {
              const diff = undoSteps.length - 1 - idx;
              for (let i = 0; i < diff; i++) handleUndo();
            }}
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
          invokeTauri('apply_filter', { kind: 'Curves', settings: data });
          setUndoSteps([...undoSteps, 'Curves']);
        }}
      />

      <LevelsModal
        isOpen={isLevelsOpen}
        onClose={() => setLevelsOpen(false)}
        onApply={(data) => {
          invokeTauri('apply_filter', { kind: 'Levels', settings: data });
          setUndoSteps([...undoSteps, 'Levels']);
        }}
      />

      <HueSaturationModal
        isOpen={isHueSatOpen}
        onClose={() => setHueSatOpen(false)}
        onApply={(data) => {
          invokeTauri('apply_filter', { kind: 'Hue/Saturation', settings: data });
          setUndoSteps([...undoSteps, 'Hue/Saturation']);
        }}
      />

      <FilterModal
        filterKind={activeFilterKind}
        onClose={() => setActiveFilterKind(null)}
        onApply={(kind, settings) => {
          invokeTauri('apply_filter', { kind, settings });
          setUndoSteps([...undoSteps, kind]);
        }}
      />

      <CameraRawModal
        isOpen={isCameraRawOpen}
        onClose={() => setCameraRawOpen(false)}
        onApply={(settings) => {
          invokeTauri('apply_filter', { kind: 'Camera Raw', settings });
          setUndoSteps([...undoSteps, 'Camera Raw']);
        }}
      />

      <NewDocModal
        isOpen={isNewDocOpen}
        onClose={() => setNewDocOpen(false)}
        onCreate={(name, w, h) => {
          setDocWidth(w);
          setDocHeight(h);
          setTabs([{ id: `doc-${Date.now()}`, name, isDirty: false }, ...tabs]);
          invokeTauri('new_document', { name, width: w, height: h });
        }}
      />

      <ExportModal
        isOpen={isExportOpen}
        onClose={() => setExportOpen(false)}
        onExport={(format, quality) => {
          invokeTauri('export_image', { format, quality });
        }}
      />
    </div>
  );
};
