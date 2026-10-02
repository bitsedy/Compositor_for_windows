import React, { useState } from 'react';

interface CurvesModalProps {
  isOpen: boolean;
  onClose: () => void;
  onApply: (data: any) => void;
}

export const CurvesModal: React.FC<CurvesModalProps> = ({ isOpen, onClose, onApply }) => {
  const [channel, setChannel] = useState<'RGB' | 'Red' | 'Green' | 'Blue'>('RGB');
  const [points] = useState<[number, number][]>([
    [0, 0],
    [255, 255],
  ]);

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
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>Curves</span>
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

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          {/* Curve Grid */}
          <div
            style={{
              width: 256,
              height: 256,
              backgroundColor: '#1b1b1b',
              border: '1px solid var(--border-color)',
              position: 'relative',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <svg width="256" height="256" style={{ position: 'absolute', inset: 0 }}>
              {/* Grid lines */}
              <line x1="64" y1="0" x2="64" y2="256" stroke="#333" strokeDasharray="2,2" />
              <line x1="128" y1="0" x2="128" y2="256" stroke="#333" strokeDasharray="2,2" />
              <line x1="192" y1="0" x2="192" y2="256" stroke="#333" strokeDasharray="2,2" />
              <line x1="0" y1="64" x2="256" y2="64" stroke="#333" strokeDasharray="2,2" />
              <line x1="0" y1="128" x2="256" y2="128" stroke="#333" strokeDasharray="2,2" />
              <line x1="0" y1="192" x2="256" y2="192" stroke="#333" strokeDasharray="2,2" />

              {/* Curve Line */}
              <line
                x1={points[0][0]}
                y1={256 - points[0][1]}
                x2={points[1][0]}
                y2={256 - points[1][1]}
                stroke={
                  channel === 'Red'
                    ? '#f7768e'
                    : channel === 'Green'
                    ? '#9ece6a'
                    : channel === 'Blue'
                    ? '#7aa2f7'
                    : '#ffffff'
                }
                strokeWidth="2"
              />

              {/* Control points */}
              {points.map(([px, py], i) => (
                <circle
                  key={i}
                  cx={px}
                  cy={256 - py}
                  r="5"
                  fill="#007acc"
                  stroke="#fff"
                  strokeWidth="1.5"
                />
              ))}
            </svg>
          </div>

          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              width: '100%',
              marginTop: 12,
              fontSize: 11,
              color: 'var(--text-muted)',
            }}
          >
            <span>Input: 0</span>
            <span>Output: 0</span>
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
          <button
            onClick={onClose}
            style={{
              padding: '4px 12px',
              backgroundColor: 'var(--bg-input)',
              borderRadius: 3,
            }}
          >
            Cancel
          </button>
          <button
            onClick={() => {
              onApply({ channel, points });
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
