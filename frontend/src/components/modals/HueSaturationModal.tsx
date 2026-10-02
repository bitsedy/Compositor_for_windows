import React, { useState } from 'react';

interface HueSaturationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onApply: (data: any) => void;
}

export const HueSaturationModal: React.FC<HueSaturationModalProps> = ({
  isOpen,
  onClose,
  onApply,
}) => {
  const [range, setRange] = useState('Master');
  const [hue, setHue] = useState(0);
  const [saturation, setSaturation] = useState(0);
  const [lightness, setLightness] = useState(0);
  const [colorize, setColorize] = useState(false);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0,0,0,0.6)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
    >
      <div
        style={{
          width: 360,
          backgroundColor: 'var(--bg-panel)',
          border: '1px solid var(--border-color)',
          borderRadius: 6,
          boxShadow: '0 8px 24px rgba(0,0,0,0.7)',
          overflow: 'hidden',
        }}
      >
        <div
          style={{
            padding: '8px 14px',
            backgroundColor: 'var(--bg-panel-header)',
            borderBottom: '1px solid var(--border-color)',
            fontWeight: 'bold',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>Hue/Saturation</span>
          <select value={range} onChange={(e) => setRange(e.target.value)} style={{ width: 100 }}>
            <option value="Master">Master</option>
            <option value="Reds">Reds</option>
            <option value="Yellows">Yellows</option>
            <option value="Greens">Greens</option>
            <option value="Cyans">Cyans</option>
            <option value="Blues">Blues</option>
            <option value="Magentas">Magentas</option>
          </select>
        </div>

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
          {/* Hue */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span>Hue:</span>
              <span>{hue}°</span>
            </div>
            <input
              type="range"
              min="-180"
              max="180"
              value={hue}
              onChange={(e) => setHue(Number(e.target.value))}
            />
          </div>

          {/* Saturation */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span>Saturation:</span>
              <span>{saturation}</span>
            </div>
            <input
              type="range"
              min="-100"
              max="100"
              value={saturation}
              onChange={(e) => setSaturation(Number(e.target.value))}
            />
          </div>

          {/* Lightness */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span>Lightness:</span>
              <span>{lightness}</span>
            </div>
            <input
              type="range"
              min="-100"
              max="100"
              value={lightness}
              onChange={(e) => setLightness(Number(e.target.value))}
            />
          </div>

          <label style={{ display: 'flex', alignItems: 'center', gap: 6, cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={colorize}
              onChange={(e) => setColorize(e.target.checked)}
            />
            <span>Colorize</span>
          </label>
        </div>

        <div
          style={{
            padding: '10px 14px',
            backgroundColor: 'var(--bg-panel-header)',
            borderTop: '1px solid var(--border-color)',
            display: 'flex',
            justifyContent: 'flex-end',
            gap: 8,
          }}
        >
          <button onClick={onClose} style={{ padding: '4px 12px', backgroundColor: 'var(--bg-input)', borderRadius: 3 }}>
            Cancel
          </button>
          <button
            onClick={() => {
              onApply({ range, hue, saturation, lightness, colorize });
              onClose();
            }}
            style={{
              padding: '4px 16px',
              backgroundColor: 'var(--accent)',
              color: '#fff',
              borderRadius: 3,
              fontWeight: 'bold',
            }}
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
};
