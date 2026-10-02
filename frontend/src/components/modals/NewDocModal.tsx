import React, { useState } from 'react';

interface NewDocModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (name: string, width: number, height: number) => void;
}

export const NewDocModal: React.FC<NewDocModalProps> = ({ isOpen, onClose, onCreate }) => {
  const [name, setName] = useState('Untitled-1');
  const [width, setWidth] = useState(1920);
  const [height, setHeight] = useState(1080);

  if (!isOpen) return null;

  const presets = [
    { label: 'Full HD (1920 × 1080)', w: 1920, h: 1080 },
    { label: '4K UHD (3840 × 2160)', w: 3840, h: 2160 },
    { label: 'Square (2048 × 2048)', w: 2048, h: 2048 },
    { label: 'Instagram Portrait (1080 × 1350)', w: 1080, h: 1350 },
    { label: 'Photo (3000 × 2000)', w: 3000, h: 2000 },
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
          width: 400,
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
          }}
        >
          New Document
        </div>

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <span>Name:</span>
            <input type="text" value={name} onChange={(e) => setName(e.target.value)} />
          </div>

          <div style={{ display: 'flex', gap: 12 }}>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 4 }}>
              <span>Width (px):</span>
              <input
                type="number"
                value={width}
                onChange={(e) => setWidth(Number(e.target.value))}
                min="1"
                max="65535"
              />
            </div>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 4 }}>
              <span>Height (px):</span>
              <input
                type="number"
                value={height}
                onChange={(e) => setHeight(Number(e.target.value))}
                min="1"
                max="65535"
              />
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <span>Preset:</span>
            <select
              onChange={(e) => {
                const p = presets[Number(e.target.value)];
                if (p) {
                  setWidth(p.w);
                  setHeight(p.h);
                }
              }}
            >
              {presets.map((p, idx) => (
                <option key={idx} value={idx}>
                  {p.label}
                </option>
              ))}
            </select>
          </div>
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
              onCreate(name, width, height);
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
            Create
          </button>
        </div>
      </div>
    </div>
  );
};
