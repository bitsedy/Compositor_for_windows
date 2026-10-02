import React, { useEffect, useRef } from 'react';

interface CanvasContainerProps {
  documentName: string;
  zoom: number;
  onZoomChange: (zoom: number) => void;
}

export const CanvasContainer: React.FC<CanvasContainerProps> = () => {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const updateViewport = () => {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      // Notify Tauri backend of the native child window surface positioning
      if ((window as any).__TAURI_INTERNALS__) {
        try {
          const { invoke } = (window as any).__TAURI_INTERNALS__;
          invoke('set_canvas_viewport', {
            x: Math.round(rect.left),
            y: Math.round(rect.top),
            width: Math.round(rect.width),
            height: Math.round(rect.height),
          }).catch(() => {});
        } catch (_) {}
      }
    };

    updateViewport();
    window.addEventListener('resize', updateViewport);
    return () => window.removeEventListener('resize', updateViewport);
  }, []);

  return (
    <div
      style={{
        flex: 1,
        position: 'relative',
        backgroundColor: '#181818',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Horizontal Top Ruler */}
      <div
        style={{
          height: 18,
          backgroundColor: 'var(--bg-panel-header)',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          fontSize: 9,
          color: 'var(--text-muted)',
          paddingLeft: 22,
          position: 'relative',
        }}
      >
        <span>0</span>
        <span style={{ marginLeft: 90 }}>100</span>
        <span style={{ marginLeft: 90 }}>200</span>
        <span style={{ marginLeft: 90 }}>300</span>
        <span style={{ marginLeft: 90 }}>400</span>
        <span style={{ marginLeft: 90 }}>500</span>
        <span style={{ marginLeft: 90 }}>600</span>
        <span style={{ marginLeft: 90 }}>700</span>
        <span style={{ marginLeft: 90 }}>800</span>
      </div>

      <div style={{ flex: 1, display: 'flex', position: 'relative' }}>
        {/* Vertical Left Ruler */}
        <div
          style={{
            width: 18,
            backgroundColor: 'var(--bg-panel-header)',
            borderRight: '1px solid var(--border-color)',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            fontSize: 9,
            color: 'var(--text-muted)',
            paddingTop: 4,
          }}
        >
          <span>0</span>
          <span style={{ marginTop: 90 }}>100</span>
          <span style={{ marginTop: 90 }}>200</span>
          <span style={{ marginTop: 90 }}>300</span>
          <span style={{ marginTop: 90 }}>400</span>
          <span style={{ marginTop: 90 }}>500</span>
          <span style={{ marginTop: 90 }}>600</span>
        </div>

        {/* Native wgpu Surface Host Placeholder */}
        <div
          id="canvas-surface-host"
          ref={containerRef}
          style={{
            flex: 1,
            height: '100%',
            position: 'relative',
            background:
              'repeating-conic-gradient(#2a2a2a 0% 25%, #222222 0% 50%) 50% / 20px 20px',
          }}
        >
          {/* Transparent click/pointer shield overlay allowing direct pointer pass-through */}
        </div>
      </div>
    </div>
  );
};
