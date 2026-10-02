import React from 'react';
import { LayerItem, BlendMode } from '../types';
import {
  Eye,
  EyeOff,
  Folder,
  Layers,
  Sliders,
  Sparkles,
  Plus,
  Trash2,
  Lock,
} from 'lucide-react';

interface LayersPanelProps {
  layers: LayerItem[];
  activeLayerId: string;
  onSelectLayer: (id: string) => void;
  onToggleVisibility: (id: string) => void;
  onChangeBlendMode: (id: string, mode: BlendMode) => void;
  onChangeOpacity: (id: string, opacity: number) => void;
  onAddLayer: () => void;
  onAddGroup: () => void;
  onDeleteLayer: (id: string) => void;
}

export const BLEND_MODES: BlendMode[] = [
  'Normal',
  'Dissolve',
  'Darken',
  'Multiply',
  'Color Burn',
  'Linear Burn',
  'Darker Color',
  'Lighten',
  'Screen',
  'Color Dodge',
  'Linear Dodge (Add)',
  'Lighter Color',
  'Overlay',
  'Soft Light',
  'Hard Light',
  'Vivid Light',
  'Linear Light',
  'Pin Light',
  'Hard Mix',
  'Difference',
  'Exclusion',
  'Subtract',
  'Divide',
  'Hue',
  'Saturation',
  'Color',
  'Luminosity',
];

export const LayersPanel: React.FC<LayersPanelProps> = ({
  layers,
  activeLayerId,
  onSelectLayer,
  onToggleVisibility,
  onChangeBlendMode,
  onChangeOpacity,
  onAddLayer,
  onAddGroup,
  onDeleteLayer,
}) => {
  const activeLayer = layers.find((l) => l.id === activeLayerId) || layers[0];

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: 'var(--bg-panel)',
        color: 'var(--text-main)',
      }}
    >
      {/* Panel Header */}
      <div
        style={{
          padding: '6px 10px',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          flexDirection: 'column',
          gap: 8,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontWeight: 'bold', fontSize: 11 }}>LAYERS</span>
          <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
            <Lock size={12} color="var(--text-muted)" />
          </div>
        </div>

        {/* Blend Mode & Opacity */}
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <select
            value={activeLayer?.blendMode || 'Normal'}
            onChange={(e) =>
              activeLayer && onChangeBlendMode(activeLayer.id, e.target.value as BlendMode)
            }
            style={{ flex: 1 }}
          >
            {BLEND_MODES.map((mode) => (
              <option key={mode} value={mode}>
                {mode}
              </option>
            ))}
          </select>

          <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
            <span>Opacity:</span>
            <input
              type="number"
              min="0"
              max="100"
              value={Math.round((activeLayer?.opacity ?? 1.0) * 100)}
              onChange={(e) =>
                activeLayer && onChangeOpacity(activeLayer.id, Number(e.target.value) / 100)
              }
              style={{ width: 44 }}
            />
            <span>%</span>
          </div>
        </div>
      </div>

      {/* Layer List */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px 0' }}>
        {layers.map((layer) => {
          const isSelected = layer.id === activeLayerId;
          return (
            <div
              key={layer.id}
              onClick={() => onSelectLayer(layer.id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                padding: '4px 8px',
                backgroundColor: isSelected ? 'var(--bg-active)' : 'transparent',
                color: isSelected ? 'var(--text-bright)' : 'var(--text-main)',
                cursor: 'pointer',
                borderBottom: '1px solid #2d2d2d',
                gap: 8,
              }}
              onMouseEnter={(e) => {
                if (!isSelected) (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-hover)';
              }}
              onMouseLeave={(e) => {
                if (!isSelected) (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
              }}
            >
              {/* Visibility Toggle */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleVisibility(layer.id);
                }}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  color: layer.isVisible ? 'inherit' : 'var(--text-muted)',
                }}
              >
                {layer.isVisible ? <Eye size={14} /> : <EyeOff size={14} />}
              </button>

              {/* Layer Thumbnail / Icon */}
              <div
                style={{
                  width: 24,
                  height: 24,
                  backgroundColor: '#1a1a1a',
                  border: '1px solid var(--border-color)',
                  borderRadius: 2,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                {layer.isGroup ? (
                  <Folder size={14} color="#e0af68" />
                ) : layer.adjustmentKind ? (
                  <Sliders size={14} color="#7aa2f7" />
                ) : (
                  <Layers size={14} color="#9ece6a" />
                )}
              </div>

              {/* Layer Title */}
              <div style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {layer.name}
              </div>

              {/* Badges for Mask & Effects */}
              {layer.hasEffects && (
                <span title="Layer Effects" style={{ color: '#bb9af7', display: 'flex' }}>
                  <Sparkles size={12} />
                </span>
              )}
            </div>
          );
        })}
      </div>

      {/* Footer Actions */}
      <div
        style={{
          height: 30,
          borderTop: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-end',
          padding: '0 8px',
          gap: 6,
        }}
      >
        <button
          onClick={onAddGroup}
          title="New Group (Ctrl+G)"
          style={{ padding: 4, borderRadius: 3 }}
        >
          <Folder size={14} />
        </button>
        <button
          onClick={onAddLayer}
          title="New Layer (Ctrl+Shift+N)"
          style={{ padding: 4, borderRadius: 3 }}
        >
          <Plus size={14} />
        </button>
        <button
          onClick={() => activeLayer && onDeleteLayer(activeLayer.id)}
          title="Delete Layer"
          style={{ padding: 4, borderRadius: 3 }}
        >
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );
};
