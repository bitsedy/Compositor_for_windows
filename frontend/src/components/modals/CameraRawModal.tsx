import React, { useState } from 'react';

interface CameraRawModalProps {
  isOpen: boolean;
  onClose: () => void;
  onApply: (settings: any) => void;
}

export const CameraRawModal: React.FC<CameraRawModalProps> = ({ isOpen, onClose, onApply }) => {
  const [exposure, setExposure] = useState(0.0);
  const [contrast, setContrast] = useState(0);
  const [highlights, setHighlights] = useState(0);
  const [shadows, setShadows] = useState(0);
  const [whites, setWhites] = useState(0);
  const [blacks, setBlacks] = useState(0);
  const [temperature, setTemperature] = useState(0);
  const [tint, setTint] = useState(0);
  const [vibrance, setVibrance] = useState(0);
  const [saturation, setSaturation] = useState(0);
  const [texture, setTexture] = useState(0);
  const [clarity, setClarity] = useState(0);
  const [dehaze, setDehaze] = useState(0);
  const [vignette, setVignette] = useState(0);

  if (!isOpen) return null;

  const sliders = [
    { label: 'Exposure', value: exposure, setter: setExposure, min: -5, max: 5, step: 0.1 },
    { label: 'Contrast', value: contrast, setter: setContrast, min: -100, max: 100, step: 1 },
    { label: 'Highlights', value: highlights, setter: setHighlights, min: -100, max: 100, step: 1 },
    { label: 'Shadows', value: shadows, setter: setShadows, min: -100, max: 100, step: 1 },
    { label: 'Whites', value: whites, setter: setWhites, min: -100, max: 100, step: 1 },
    { label: 'Blacks', value: blacks, setter: setBlacks, min: -100, max: 100, step: 1 },
    { label: 'Temp', value: temperature, setter: setTemperature, min: -100, max: 100, step: 1 },
    { label: 'Tint', value: tint, setter: setTint, min: -100, max: 100, step: 1 },
    { label: 'Vibrance', value: vibrance, setter: setVibrance, min: -100, max: 100, step: 1 },
    { label: 'Saturation', value: saturation, setter: setSaturation, min: -100, max: 100, step: 1 },
    { label: 'Texture', value: texture, setter: setTexture, min: -100, max: 100, step: 1 },
    { label: 'Clarity', value: clarity, setter: setClarity, min: -100, max: 100, step: 1 },
    { label: 'Dehaze', value: dehaze, setter: setDehaze, min: -100, max: 100, step: 1 },
    { label: 'Vignette', value: vignette, setter: setVignette, min: -100, max: 100, step: 1 },
  ];

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
          width: 440,
          maxHeight: '90vh',
          backgroundColor: 'var(--bg-panel)',
          border: '1px solid var(--border-color)',
          borderRadius: 6,
          boxShadow: '0 8px 24px rgba(0,0,0,0.7)',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
      >
        <div
          style={{
            padding: '8px 14px',
            backgroundColor: 'var(--bg-panel-header)',
            borderBottom: '1px solid var(--border-color)',
            fontWeight: 'bold',
          }}
        >
          Camera Raw Filter
        </div>

        <div style={{ flex: 1, overflowY: 'auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
          {sliders.map((s) => (
            <div key={s.label} style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11 }}>
                <span>{s.label}:</span>
                <span>{s.value}</span>
              </div>
              <input
                type="range"
                min={s.min}
                max={s.max}
                step={s.step}
                value={s.value}
                onChange={(e) => s.setter(Number(e.target.value))}
              />
            </div>
          ))}
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
              onApply({
                exposure,
                contrast,
                highlights,
                shadows,
                whites,
                blacks,
                temperature,
                tint,
                vibrance,
                saturation,
                texture,
                clarity,
                dehaze,
                vignette,
              });
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
