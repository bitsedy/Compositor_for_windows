import React, { useState } from 'react';

interface FilterModalProps {
  filterKind: string | null;
  onClose: () => void;
  onApply: (kind: string, settings: any) => void;
}

export const FilterModal: React.FC<FilterModalProps> = ({ filterKind, onClose, onApply }) => {
  const [radius, setRadius] = useState(10);
  const [angle, setAngle] = useState(0);
  const [distance, setDistance] = useState(15);
  const [amount, setAmount] = useState(25);
  const [gaussian, setGaussian] = useState(false);
  const [preview, setPreview] = useState(true);

  if (!filterKind) return null;

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
          }}
        >
          {filterKind}
        </div>

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
          {filterKind === 'Gaussian Blur' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>Radius:</span>
                <span>{radius} px</span>
              </div>
              <input
                type="range"
                min="0.1"
                max="100"
                step="0.5"
                value={radius}
                onChange={(e) => setRadius(Number(e.target.value))}
              />
            </div>
          )}

          {filterKind === 'Motion Blur' && (
            <>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span>Angle:</span>
                  <span>{angle}°</span>
                </div>
                <input
                  type="range"
                  min="-90"
                  max="90"
                  value={angle}
                  onChange={(e) => setAngle(Number(e.target.value))}
                />
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span>Distance:</span>
                  <span>{distance} px</span>
                </div>
                <input
                  type="range"
                  min="1"
                  max="200"
                  value={distance}
                  onChange={(e) => setDistance(Number(e.target.value))}
                />
              </div>
            </>
          )}

          {filterKind === 'Add Noise' && (
            <>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span>Amount:</span>
                  <span>{amount}%</span>
                </div>
                <input
                  type="range"
                  min="0.1"
                  max="100"
                  value={amount}
                  onChange={(e) => setAmount(Number(e.target.value))}
                />
              </div>

              <label style={{ display: 'flex', alignItems: 'center', gap: 6, cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={gaussian}
                  onChange={(e) => setGaussian(e.target.checked)}
                />
                <span>Gaussian Distribution</span>
              </label>
            </>
          )}

          <label style={{ display: 'flex', alignItems: 'center', gap: 6, cursor: 'pointer', marginTop: 8 }}>
            <input
              type="checkbox"
              checked={preview}
              onChange={(e) => setPreview(e.target.checked)}
            />
            <span>Preview</span>
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
              onApply(filterKind, { radius, angle, distance, amount, gaussian });
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
