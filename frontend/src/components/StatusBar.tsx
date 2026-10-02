import React from 'react';

interface StatusBarProps {
  width: number;
  height: number;
  zoom: number;
  cursorX?: number;
  cursorY?: number;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  width,
  height,
  zoom,
  cursorX = 0,
  cursorY = 0,
}) => {
  return (
    <div
      style={{
        height: 22,
        backgroundColor: 'var(--bg-panel-header)',
        borderTop: '1px solid var(--border-color)',
        display: 'flex',
        alignItems: 'center',
        padding: '0 12px',
        fontSize: 11,
        color: 'var(--text-muted)',
        gap: 20,
      }}
    >
      <span>{Math.round(zoom * 100)}%</span>
      <span>Doc: {width} × {height} px (sRGB)</span>
      <span>X: {cursorX} px  Y: {cursorY} px</span>
      <div style={{ flex: 1 }} />
      <span style={{ color: 'var(--text-main)' }}>D3D12 GPU Active | 60 FPS</span>
    </div>
  );
};
