import React, { useState } from 'react';

interface ExportModalProps {
  isOpen: boolean;
  onClose: () => void;
  onExport: (format: string, quality: number) => void;
}

export const ExportModal: React.FC<ExportModalProps> = ({ isOpen, onClose, onExport }) => {
  const [format, setFormat] = useState('jpeg');
  const [quality, setQuality] = useState(90);

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
          }}
        >
          Export Image
        </div>

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <span>Format:</span>
            <select value={format} onChange={(e) => setFormat(e.target.value)}>
              <option value="jpeg">JPEG (.jpg)</option>
              <option value="png">PNG (.png)</option>
              <option value="tiff">TIFF (.tif)</option>
              <option value="webp">WebP (.webp)</option>
            </select>
          </div>

          {format === 'jpeg' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>Quality:</span>
                <span>{quality}%</span>
              </div>
              <input
                type="range"
                min="10"
                max="100"
                value={quality}
                onChange={(e) => setQuality(Number(e.target.value))}
              />
            </div>
          )}

          <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>
            <span>Estimated output size: ~{(quality * 18.5).toFixed(0)} KB</span>
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
              onExport(format, quality);
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
            Export
          </button>
        </div>
      </div>
    </div>
  );
};
