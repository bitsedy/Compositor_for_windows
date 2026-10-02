import React, { useState } from 'react';

interface LevelsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onApply: (data: any) => void;
}

export const LevelsModal: React.FC<LevelsModalProps> = ({ isOpen, onClose, onApply }) => {
  const [channel, setChannel] = useState<'RGB' | 'Red' | 'Green' | 'Blue'>('RGB');
  const [black, setBlack] = useState(0);
  const [gamma, setGamma] = useState(1.0);
  const [white, setWhite] = useState(255);
  const [outBlack, setOutBlack] = useState(0);
  const [outWhite, setOutWhite] = useState(255);

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
          width: 380,
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
          <span>Levels</span>
          <select
            value={channel}
            onChange={(e) => setChannel(e.target.value as any)}
            style={{ width: 90 }}
          >
            <option value="RGB">RGB</option>
            <option value="Red">Red</option>
            <option value="Green">Green</option>
            <option value="Blue">Blue</option>
          </select>
        </div>

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
          {/* Histogram Placeholder */}
          <div
            style={{
              height: 120,
              backgroundColor: '#1b1b1b',
              border: '1px solid var(--border-color)',
              position: 'relative',
              display: 'flex',
              alignItems: 'flex-end',
              padding: '0 2px',
            }}
          >
            {/* Simulated histogram bars */}
            {Array.from({ length: 64 }).map((_, i) => {
              const h = Math.sin(i / 10) * 40 + Math.random() * 50 + 20;
              return (
                <div
                  key={i}
                  style={{
                    flex: 1,
                    height: `${Math.min(100, h)}%`,
                    backgroundColor: '#666',
                    marginRight: 1,
                  }}
                />
              );
            })}
          </div>

          {/* Input Levels */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            <span style={{ fontSize: 11, color: 'var(--text-muted)' }}>Input Levels:</span>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: 8 }}>
              <input
                type="number"
                value={black}
                onChange={(e) => setBlack(Number(e.target.value))}
                min="0"
                max="254"
                style={{ width: 50 }}
              />
              <input
                type="number"
                value={gamma}
                step="0.05"
                onChange={(e) => setGamma(Number(e.target.value))}
                min="0.1"
                max="9.9"
                style={{ width: 60 }}
              />
              <input
                type="number"
                value={white}
                onChange={(e) => setWhite(Number(e.target.value))}
                min="1"
                max="255"
                style={{ width: 50 }}
              />
            </div>
          </div>

          {/* Output Levels */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            <span style={{ fontSize: 11, color: 'var(--text-muted)' }}>Output Levels:</span>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: 8 }}>
              <input
                type="number"
                value={outBlack}
                onChange={(e) => setOutBlack(Number(e.target.value))}
                min="0"
                max="255"
                style={{ width: 50 }}
              />
              <input
                type="number"
                value={outWhite}
                onChange={(e) => setOutWhite(Number(e.target.value))}
                min="0"
                max="255"
                style={{ width: 50 }}
              />
            </div>
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
              onApply({ channel, black, gamma, white, outBlack, outWhite });
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
