import React from 'react';
import { ToolKind, BrushSettings } from '../types';

interface ToolOptionsBarProps {
  activeTool: ToolKind;
  brushSettings: BrushSettings;
  onUpdateBrush: (settings: Partial<BrushSettings>) => void;
}

export const ToolOptionsBar: React.FC<ToolOptionsBarProps> = ({
  activeTool,
  brushSettings,
  onUpdateBrush,
}) => {
  return (
    <div
      style={{
        height: 32,
        backgroundColor: 'var(--bg-panel)',
        borderBottom: '1px solid var(--border-color)',
        display: 'flex',
        alignItems: 'center',
        padding: '0 12px',
        gap: 16,
        fontSize: 11,
      }}
    >
      <span style={{ fontWeight: 'bold', color: 'var(--text-bright)', textTransform: 'uppercase' }}>
        {activeTool.replace('_', ' ')}
      </span>

      {(activeTool === 'brush' ||
        activeTool === 'eraser' ||
        activeTool === 'spot_heal' ||
        activeTool === 'clone_stamp') && (
        <>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>Size:</span>
            <input
              type="range"
              min="1"
              max="500"
              value={brushSettings.size}
              onChange={(e) => onUpdateBrush({ size: Number(e.target.value) })}
              style={{ width: 80 }}
            />
            <span>{brushSettings.size} px</span>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>Hardness:</span>
            <input
              type="range"
              min="0"
              max="100"
              value={brushSettings.hardness}
              onChange={(e) => onUpdateBrush({ hardness: Number(e.target.value) })}
              style={{ width: 80 }}
            />
            <span>{brushSettings.hardness}%</span>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>Opacity:</span>
            <input
              type="range"
              min="1"
              max="100"
              value={brushSettings.opacity}
              onChange={(e) => onUpdateBrush({ opacity: Number(e.target.value) })}
              style={{ width: 80 }}
            />
            <span>{brushSettings.opacity}%</span>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>Flow:</span>
            <input
              type="range"
              min="1"
              max="100"
              value={brushSettings.flow}
              onChange={(e) => onUpdateBrush({ flow: Number(e.target.value) })}
              style={{ width: 80 }}
            />
            <span>{brushSettings.flow}%</span>
          </div>

          <label style={{ display: 'flex', alignItems: 'center', gap: 4, cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={brushSettings.pressureSize}
              onChange={(e) => onUpdateBrush({ pressureSize: e.target.checked })}
            />
            <span>Pressure Size</span>
          </label>
        </>
      )}

      {activeTool === 'wand' && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>Tolerance:</span>
            <input type="number" defaultValue="32" min="0" max="255" style={{ width: 50 }} />
          </div>
          <label style={{ display: 'flex', alignItems: 'center', gap: 4, cursor: 'pointer' }}>
            <input type="checkbox" defaultChecked />
            <span>Anti-alias</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: 4, cursor: 'pointer' }}>
            <input type="checkbox" defaultChecked />
            <span>Contiguous</span>
          </label>
        </div>
      )}

      {activeTool === 'crop' && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <span>Aspect Ratio:</span>
          <select defaultValue="unconstrained">
            <option value="unconstrained">Unconstrained</option>
            <option value="original">Original Ratio</option>
            <option value="1:1">1:1 (Square)</option>
            <option value="16:9">16:9 (Widescreen)</option>
            <option value="4:3">4:3</option>
          </select>
          <button
            style={{
              padding: '2px 8px',
              backgroundColor: 'var(--bg-active)',
              color: 'var(--text-bright)',
              borderRadius: 3,
            }}
          >
            Apply Crop
          </button>
        </div>
      )}
    </div>
  );
};
