import React from 'react';
import { ToolKind, PaletteColor } from '../types';
import {
  Move,
  Square,
  Circle,
  Lasso,
  Wand2,
  Crop,
  Pipette,
  Bandage,
  Paintbrush,
  Stamp,
  History,
  Eraser,
  Blend,
  Droplet,
  Sun,
  Type,
  Shapes,
  Hand,
  ZoomIn,
  ArrowUpDown,
} from 'lucide-react';

interface ToolsPaletteProps {
  activeTool: ToolKind;
  onSelectTool: (tool: ToolKind) => void;
  foregroundColor: PaletteColor;
  backgroundColor: PaletteColor;
  onSwapColors: () => void;
  onResetColors: () => void;
  onOpenColorPicker: (which: 'fg' | 'bg') => void;
}

export const ToolsPalette: React.FC<ToolsPaletteProps> = ({
  activeTool,
  onSelectTool,
  foregroundColor,
  backgroundColor,
  onSwapColors,
  onResetColors,
  onOpenColorPicker,
}) => {
  const tools: { id: ToolKind; label: string; icon: React.ReactNode; shortcut: string }[] = [
    { id: 'move', label: 'Move Tool', icon: <Move size={16} />, shortcut: 'V' },
    { id: 'marquee_rect', label: 'Rectangular Marquee', icon: <Square size={16} />, shortcut: 'M' },
    { id: 'marquee_ellipse', label: 'Elliptical Marquee', icon: <Circle size={16} />, shortcut: 'Shift+M' },
    { id: 'lasso', label: 'Lasso Tool', icon: <Lasso size={16} />, shortcut: 'L' },
    { id: 'wand', label: 'Magic Wand Tool', icon: <Wand2 size={16} />, shortcut: 'W' },
    { id: 'crop', label: 'Crop Tool', icon: <Crop size={16} />, shortcut: 'C' },
    { id: 'eyedropper', label: 'Eyedropper Tool', icon: <Pipette size={16} />, shortcut: 'I' },
    { id: 'spot_heal', label: 'Spot Healing Brush', icon: <Bandage size={16} />, shortcut: 'J' },
    { id: 'brush', label: 'Brush Tool', icon: <Paintbrush size={16} />, shortcut: 'B' },
    { id: 'clone_stamp', label: 'Clone Stamp Tool', icon: <Stamp size={16} />, shortcut: 'S' },
    { id: 'history_brush', label: 'History Brush Tool', icon: <History size={16} />, shortcut: 'Y' },
    { id: 'eraser', label: 'Eraser Tool', icon: <Eraser size={16} />, shortcut: 'E' },
    { id: 'gradient', label: 'Gradient Tool', icon: <Blend size={16} />, shortcut: 'G' },
    { id: 'blur', label: 'Blur / Sharpen / Smudge', icon: <Droplet size={16} />, shortcut: 'R' },
    { id: 'dodge', label: 'Dodge / Burn / Sponge', icon: <Sun size={16} />, shortcut: 'O' },
    { id: 'type', label: 'Type Tool', icon: <Type size={16} />, shortcut: 'T' },
    { id: 'shape', label: 'Shape Tool', icon: <Shapes size={16} />, shortcut: 'U' },
    { id: 'hand', label: 'Hand Tool', icon: <Hand size={16} />, shortcut: 'H' },
    { id: 'zoom', label: 'Zoom Tool', icon: <ZoomIn size={16} />, shortcut: 'Z' },
  ];

  const fgHex = `rgb(${Math.round(foregroundColor.red * 255)}, ${Math.round(
    foregroundColor.green * 255
  )}, ${Math.round(foregroundColor.blue * 255)})`;
  const bgHex = `rgb(${Math.round(backgroundColor.red * 255)}, ${Math.round(
    backgroundColor.green * 255
  )}, ${Math.round(backgroundColor.blue * 255)})`;

  return (
    <div
      style={{
        width: 44,
        backgroundColor: 'var(--bg-panel)',
        borderRight: '1px solid var(--border-color)',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: '6px 0',
        zIndex: 50,
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        {tools.map((t) => {
          const isActive = activeTool === t.id;
          return (
            <button
              key={t.id}
              onClick={() => onSelectTool(t.id)}
              title={`${t.label} (${t.shortcut})`}
              style={{
                width: 32,
                height: 32,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: 4,
                backgroundColor: isActive ? 'var(--bg-active)' : 'transparent',
                color: isActive ? 'var(--text-bright)' : 'var(--text-main)',
              }}
              onMouseEnter={(e) => {
                if (!isActive) (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-hover)';
              }}
              onMouseLeave={(e) => {
                if (!isActive) (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
              }}
            >
              {t.icon}
            </button>
          );
        })}
      </div>

      <div style={{ flex: 1 }} />

      {/* Color Swatches */}
      <div style={{ position: 'relative', width: 34, height: 34, marginBottom: 8 }}>
        {/* Background Swatch */}
        <div
          onClick={() => onOpenColorPicker('bg')}
          title="Background Color"
          style={{
            position: 'absolute',
            bottom: 0,
            right: 0,
            width: 20,
            height: 20,
            backgroundColor: bgHex,
            border: '1px solid var(--border-color)',
            boxShadow: '0 1px 3px rgba(0,0,0,0.4)',
            cursor: 'pointer',
          }}
        />
        {/* Foreground Swatch */}
        <div
          onClick={() => onOpenColorPicker('fg')}
          title="Foreground Color"
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: 20,
            height: 20,
            backgroundColor: fgHex,
            border: '1px solid var(--border-color)',
            boxShadow: '0 1px 3px rgba(0,0,0,0.4)',
            cursor: 'pointer',
            zIndex: 2,
          }}
        />
        {/* Swap Colors Button */}
        <button
          onClick={onSwapColors}
          title="Switch Foreground and Background Colors (X)"
          style={{
            position: 'absolute',
            top: -2,
            right: -2,
            width: 12,
            height: 12,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'var(--text-muted)',
            cursor: 'pointer',
            zIndex: 3,
          }}
        >
          <ArrowUpDown size={10} />
        </button>
        {/* Reset Colors Button */}
        <button
          onClick={onResetColors}
          title="Default Foreground/Background Colors (D)"
          style={{
            position: 'absolute',
            bottom: -2,
            left: -2,
            width: 10,
            height: 10,
            backgroundColor: '#000',
            border: '1px solid #fff',
            cursor: 'pointer',
            zIndex: 3,
          }}
        />
      </div>
    </div>
  );
};
