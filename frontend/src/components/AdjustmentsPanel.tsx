import React from 'react';
import {
  Sliders,
  TrendingUp,
  Activity,
  Sun,
  Palette,
  Scale,
  Contrast,
  CircleDot,
  Blend,
  Sparkles,
} from 'lucide-react';

interface AdjustmentsPanelProps {
  onAddAdjustment: (kind: string) => void;
}

export const AdjustmentsPanel: React.FC<AdjustmentsPanelProps> = ({ onAddAdjustment }) => {
  const adjustments = [
    { id: 'Levels', label: 'Levels', icon: <Activity size={16} /> },
    { id: 'Curves', label: 'Curves', icon: <TrendingUp size={16} /> },
    { id: 'Exposure', label: 'Exposure', icon: <Sun size={16} /> },
    { id: 'Hue/Saturation', label: 'Hue/Saturation', icon: <Palette size={16} /> },
    { id: 'Color Balance', label: 'Color Balance', icon: <Scale size={16} /> },
    { id: 'Black & White', label: 'Black & White', icon: <Contrast size={16} /> },
    { id: 'Gradient Map', label: 'Gradient Map', icon: <Blend size={16} /> },
    { id: 'Invert', label: 'Invert', icon: <CircleDot size={16} /> },
    { id: 'Grain', label: 'Grain', icon: <Sparkles size={16} /> },
    { id: 'Gaussian Blur', label: 'Gaussian Blur', icon: <Sliders size={16} /> },
  ];

  return (
    <div
      style={{
        padding: 10,
        backgroundColor: 'var(--bg-panel)',
        borderBottom: '1px solid var(--border-color)',
      }}
    >
      <div style={{ fontWeight: 'bold', fontSize: 11, marginBottom: 8 }}>ADJUSTMENTS</div>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(5, 1fr)',
          gap: 6,
        }}
      >
        {adjustments.map((adj) => (
          <button
            key={adj.id}
            onClick={() => onAddAdjustment(adj.id)}
            title={`Add ${adj.label} Adjustment Layer`}
            style={{
              height: 32,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              backgroundColor: 'var(--bg-input)',
              border: '1px solid var(--border-color)',
              borderRadius: 4,
              color: 'var(--text-bright)',
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLElement).style.borderColor = 'var(--accent)';
              (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-hover)';
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLElement).style.borderColor = 'var(--border-color)';
              (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-input)';
            }}
          >
            {adj.icon}
          </button>
        ))}
      </div>
    </div>
  );
};
