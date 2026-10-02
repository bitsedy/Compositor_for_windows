import React, { useEffect, useRef, useState, useCallback } from 'react';
import { ToolKind, LayerItem, BrushSettings, PaletteColor } from '../types';

interface CanvasContainerProps {
  documentName: string;
  docWidth: number;
  docHeight: number;
  layers: LayerItem[];
  activeLayerId: string;
  activeTool: ToolKind;
  brushSettings: BrushSettings;
  foregroundColor: PaletteColor;
  backgroundColor: PaletteColor;
  zoom: number;
  onZoomChange: (zoom: number) => void;
  panX: number;
  panY: number;
  onPanChange: (x: number, y: number) => void;
  layerCanvases: Map<string, HTMLCanvasElement>;
  onLayerCanvasUpdate: (layerId: string, canvas: HTMLCanvasElement) => void;
  onColorSample: (color: PaletteColor) => void;
  selection: { x: number; y: number; width: number; height: number } | null;
  onSelectionChange: (sel: { x: number; y: number; width: number; height: number } | null) => void;
  onDropFiles: (files: FileList) => void;
  onAddHistoryStep: (name: string) => void;
}

export const CanvasContainer: React.FC<CanvasContainerProps> = ({
  docWidth,
  docHeight,
  layers,
  activeLayerId,
  activeTool,
  brushSettings,
  foregroundColor,
  zoom,
  onZoomChange,
  panX,
  panY,
  onPanChange,
  layerCanvases,
  onLayerCanvasUpdate,
  onColorSample,
  selection,
  onSelectionChange,
  onDropFiles,
  onAddHistoryStep,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const displayCanvasRef = useRef<HTMLCanvasElement>(null);
  const [isDraggingFile, setIsDraggingFile] = useState(false);
  const [cursorPos, setCursorPos] = useState<{ x: number; y: number } | null>(null);
  const [isPointerDown, setIsPointerDown] = useState(false);
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState<{ x: number; y: number }>({ x: 0, y: 0 });
  const [isSpacePressed, setIsSpacePressed] = useState(false);
  const [lastPoint, setLastPoint] = useState<{ x: number; y: number } | null>(null);
  const [selectionStart, setSelectionStart] = useState<{ x: number; y: number } | null>(null);
  const [antsOffset, setAntsOffset] = useState(0);

  // Marching ants animation
  useEffect(() => {
    if (!selection) return;
    let animId: number;
    const animate = () => {
      setAntsOffset((prev) => (prev + 0.5) % 8);
      animId = requestAnimationFrame(animate);
    };
    animId = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animId);
  }, [selection]);

  // Spacebar pan detection
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Space' && !e.repeat && !(e.target instanceof HTMLInputElement)) {
        setIsSpacePressed(true);
      }
    };
    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        setIsSpacePressed(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, []);

  // Map blend mode to CanvasRenderingContext2D globalCompositeOperation
  const getCompositeOp = (mode: string): GlobalCompositeOperation => {
    switch (mode) {
      case 'Multiply': return 'multiply';
      case 'Screen': return 'screen';
      case 'Overlay': return 'overlay';
      case 'Darken': return 'darken';
      case 'Lighten': return 'lighten';
      case 'Color Dodge': return 'color-dodge';
      case 'Color Burn': return 'color-burn';
      case 'Hard Light': return 'hard-light';
      case 'Soft Light': return 'soft-light';
      case 'Difference': return 'difference';
      case 'Exclusion': return 'exclusion';
      case 'Hue': return 'hue';
      case 'Saturation': return 'saturation';
      case 'Color': return 'color';
      case 'Luminosity': return 'luminosity';
      default: return 'source-over';
    }
  };

  // Main Render Pass: Composite all layers onto display canvas
  const renderComposite = useCallback(() => {
    const canvas = displayCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // 1. Draw Checkerboard background
    const tileSize = 16;
    for (let y = 0; y < canvas.height; y += tileSize) {
      for (let x = 0; x < canvas.width; x += tileSize) {
        ctx.fillStyle = (Math.floor(x / tileSize) + Math.floor(y / tileSize)) % 2 === 0 ? '#383838' : '#282828';
        ctx.fillRect(x, y, tileSize, tileSize);
      }
    }

    // 2. Render each visible layer from bottom to top.
    // FIXED: draw at native 1:1 pixel size (not stretched to canvas.width/height).
    // The CSS zoom/width/height on the wrapping <canvas> element handles visual scaling.
    for (const layer of layers) {
      if (!layer.isVisible) continue;
      const lCanvas = layerCanvases.get(layer.id);
      if (!lCanvas) continue;

      ctx.save();
      ctx.globalAlpha = Math.max(0, Math.min(1, layer.opacity));
      ctx.globalCompositeOperation = getCompositeOp(layer.blendMode);
      ctx.drawImage(lCanvas, 0, 0);
      ctx.restore();
    }

    // 3. Render Marching Ants Selection
    if (selection) {
      ctx.save();
      ctx.lineWidth = 1;

      // Black line
      ctx.strokeStyle = '#000000';
      ctx.setLineDash([4, 4]);
      ctx.lineDashOffset = -antsOffset;
      ctx.strokeRect(selection.x + 0.5, selection.y + 0.5, selection.width, selection.height);

      // White line
      ctx.strokeStyle = '#ffffff';
      ctx.setLineDash([4, 4]);
      ctx.lineDashOffset = -antsOffset + 4;
      ctx.strokeRect(selection.x + 0.5, selection.y + 0.5, selection.width, selection.height);

      ctx.restore();
    }
  }, [layers, layerCanvases, selection, antsOffset]);

  // Re-render when layers, layerCanvases, or selection changes
  useEffect(() => {
    renderComposite();
  }, [renderComposite]);

  // Convert client viewport coordinates to document coordinates
  const clientToDoc = (clientX: number, clientY: number) => {
    if (!containerRef.current) return { x: 0, y: 0 };
    const rect = containerRef.current.getBoundingClientRect();
    const docX = (clientX - rect.left - panX) / zoom;
    const docY = (clientY - rect.top - panY) / zoom;
    return { x: docX, y: docY };
  };

  // Draw brush dab or stroke on the active layer
  const paintBrushDab = (x1: number, y1: number, x2: number, y2: number, pressure: number) => {
    const lCanvas = layerCanvases.get(activeLayerId);
    if (!lCanvas) return;
    const ctx = lCanvas.getContext('2d');
    if (!ctx) return;

    ctx.save();
    const isEraser = activeTool === 'eraser';
    if (isEraser) {
      ctx.globalCompositeOperation = 'destination-out';
    } else {
      ctx.globalCompositeOperation = 'source-over';
    }

    const currentRadius = Math.max(1, (brushSettings.size * (brushSettings.pressureSize ? Math.max(0.1, pressure) : 1)) / 2);
    const alpha = (brushSettings.opacity / 100) * (brushSettings.flow / 100);

    const dist = Math.hypot(x2 - x1, y2 - y1);
    const steps = Math.max(1, Math.ceil(dist / (currentRadius * (brushSettings.spacing / 100))));

    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      const curX = x1 + (x2 - x1) * t;
      const curY = y1 + (y2 - y1) * t;

      const grad = ctx.createRadialGradient(curX, curY, currentRadius * (brushSettings.hardness / 100), curX, curY, currentRadius);
      const r = Math.round(foregroundColor.red * 255);
      const g = Math.round(foregroundColor.green * 255);
      const b = Math.round(foregroundColor.blue * 255);

      if (isEraser) {
        grad.addColorStop(0, `rgba(0, 0, 0, ${alpha})`);
        grad.addColorStop(1, 'rgba(0, 0, 0, 0)');
      } else {
        grad.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${alpha})`);
        grad.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0)`);
      }

      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(curX, curY, currentRadius, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.restore();
    onLayerCanvasUpdate(activeLayerId, lCanvas);
  };

  // Pointer Event Handlers
  const handlePointerDown = (e: React.PointerEvent) => {
    (e.target as HTMLElement).setPointerCapture(e.pointerId);

    // Pan with spacebar or hand tool or middle click
    if (isSpacePressed || activeTool === 'hand' || e.button === 1) {
      setIsPanning(true);
      setPanStart({ x: e.clientX - panX, y: e.clientY - panY });
      return;
    }

    if (e.button !== 0) return; // Left click only for drawing

    const { x, y } = clientToDoc(e.clientX, e.clientY);
    setIsPointerDown(true);
    setLastPoint({ x, y });

    // Eyedropper tool or Alt+Click
    if (activeTool === 'eyedropper' || e.altKey) {
      const dCanvas = displayCanvasRef.current;
      if (dCanvas) {
        const dCtx = dCanvas.getContext('2d');
        if (dCtx && x >= 0 && x < dCanvas.width && y >= 0 && y < dCanvas.height) {
          const pixel = dCtx.getImageData(Math.floor(x), Math.floor(y), 1, 1).data;
          onColorSample({
            red: pixel[0] / 255,
            green: pixel[1] / 255,
            blue: pixel[2] / 255,
            alpha: pixel[3] / 255,
          });
        }
      }
      return;
    }

    // Brush or Eraser
    if (activeTool === 'brush' || activeTool === 'eraser') {
      const pressure = e.pressure && e.pressure > 0 ? e.pressure : 1.0;
      paintBrushDab(x, y, x, y, pressure);
    }

    // Marquee Selection
    if (activeTool === 'marquee_rect') {
      setSelectionStart({ x, y });
      onSelectionChange({ x, y, width: 0, height: 0 });
    }
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    setCursorPos({ x: e.clientX, y: e.clientY });

    // Panning
    if (isPanning) {
      onPanChange(e.clientX - panStart.x, e.clientY - panStart.y);
      return;
    }

    if (!isPointerDown) return;

    const { x, y } = clientToDoc(e.clientX, e.clientY);
    const pressure = e.pressure && e.pressure > 0 ? e.pressure : 1.0;

    // Brush or Eraser
    if ((activeTool === 'brush' || activeTool === 'eraser') && lastPoint) {
      paintBrushDab(lastPoint.x, lastPoint.y, x, y, pressure);
      setLastPoint({ x, y });
    }

    // Marquee Selection
    if (activeTool === 'marquee_rect' && selectionStart) {
      const minX = Math.min(selectionStart.x, x);
      const minY = Math.min(selectionStart.y, y);
      const width = Math.abs(x - selectionStart.x);
      const height = Math.abs(y - selectionStart.y);
      onSelectionChange({ x: minX, y: minY, width, height });
    }
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    try {
      (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    } catch (_) {}

    if (isPanning) {
      setIsPanning(false);
      return;
    }

    if (isPointerDown) {
      setIsPointerDown(false);
      if (activeTool === 'brush') {
        onAddHistoryStep('Brush Stroke');
      } else if (activeTool === 'eraser') {
        onAddHistoryStep('Eraser');
      }
    }
    setLastPoint(null);
    setSelectionStart(null);
  };

  // Mouse-wheel zoom centered on cursor.
  // FIXED: Must be registered imperatively with { passive: false } so that
  // e.preventDefault() actually suppresses browser scroll / pinch-zoom.
  // React's synthetic onWheel prop cannot call preventDefault reliably.
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
      const rect = container.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;
      const factor = e.deltaY < 0 ? 1.15 : 0.85;
      const newZoom = Math.min(32.0, Math.max(0.05, zoom * factor));
      const newPanX = mouseX - (mouseX - panX) * (newZoom / zoom);
      const newPanY = mouseY - (mouseY - panY) * (newZoom / zoom);
      onZoomChange(newZoom);
      onPanChange(newPanX, newPanY);
    };

    container.addEventListener('wheel', handleWheel, { passive: false });
    return () => container.removeEventListener('wheel', handleWheel);
  }, [zoom, panX, panY, onZoomChange, onPanChange]);

  // Drag and Drop Handling
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingFile(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingFile(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingFile(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      onDropFiles(e.dataTransfer.files);
    }
  };

  // Cursor style
  const getCursorStyle = () => {
    if (isSpacePressed || isPanning || activeTool === 'hand') return 'grab';
    if (activeTool === 'eyedropper') return 'crosshair';
    if (activeTool === 'marquee_rect' || activeTool === 'lasso') return 'crosshair';
    if (activeTool === 'move') return 'move';
    if (activeTool === 'brush' || activeTool === 'eraser') return 'none';
    return 'default';
  };

  const cursorRadius = (brushSettings.size * zoom) / 2;

  return (
    <div
      style={{
        flex: 1,
        position: 'relative',
        backgroundColor: '#181818',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        userSelect: 'none',
      }}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* 1. Horizontal Top Ruler */}
      <div
        style={{
          height: 18,
          backgroundColor: 'var(--bg-panel-header)',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          fontSize: 9,
          color: 'var(--text-muted)',
          paddingLeft: 18,
          position: 'relative',
          overflow: 'hidden',
        }}
      >
        {Array.from({ length: 30 }).map((_, i) => {
          const val = i * 100;
          const pos = 18 + panX + val * zoom;
          if (pos < 0 || pos > 2500) return null;
          return (
            <span
              key={i}
              style={{
                position: 'absolute',
                left: pos,
                transform: 'translateX(-50%)',
              }}
            >
              {val}
            </span>
          );
        })}
      </div>

      <div style={{ flex: 1, display: 'flex', position: 'relative' }}>
        {/* 2. Vertical Left Ruler */}
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
            position: 'relative',
            overflow: 'hidden',
          }}
        >
          {Array.from({ length: 30 }).map((_, i) => {
            const val = i * 100;
            const pos = panY + val * zoom;
            if (pos < 0 || pos > 2000) return null;
            return (
              <span
                key={i}
                style={{
                  position: 'absolute',
                  top: pos,
                  transform: 'translateY(-50%)',
                }}
              >
                {val}
              </span>
            );
          })}
        </div>

        {/* 3. Main Canvas Interactive Viewport */}
        <div
          ref={containerRef}
          style={{
            flex: 1,
            height: '100%',
            position: 'relative',
            cursor: getCursorStyle(),
            overflow: 'hidden',
          }}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
        >
          {/* Document Canvas Sheet with Photoshop Drop Shadow */}
          <div
            style={{
              position: 'absolute',
              left: panX,
              top: panY,
              width: docWidth * zoom,
              height: docHeight * zoom,
              boxShadow: '0 8px 30px rgba(0,0,0,0.85), 0 0 1px rgba(255,255,255,0.2)',
              pointerEvents: 'none',
            }}
          >
            <canvas
              ref={displayCanvasRef}
              width={docWidth}
              height={docHeight}
              style={{
                width: '100%',
                height: '100%',
                display: 'block',
              }}
            />
          </div>

          {/* Custom Brush Reticle Outline Cursor */}
          {(activeTool === 'brush' || activeTool === 'eraser') && cursorPos && (
            <div
              style={{
                position: 'fixed',
                left: cursorPos.x - cursorRadius,
                top: cursorPos.y - cursorRadius,
                width: cursorRadius * 2,
                height: cursorRadius * 2,
                borderRadius: '50%',
                border: '1px solid rgba(255, 255, 255, 0.85)',
                boxShadow: '0 0 1px rgba(0, 0, 0, 0.9)',
                pointerEvents: 'none',
                zIndex: 9999,
              }}
            />
          )}

          {/* Drag & Drop Visual Overlay */}
          {isDraggingFile && (
            <div
              style={{
                position: 'absolute',
                inset: 0,
                backgroundColor: 'rgba(0, 120, 215, 0.3)',
                border: '3px dashed #0078d7',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                color: '#ffffff',
                fontSize: 18,
                fontWeight: 'bold',
                zIndex: 100,
                pointerEvents: 'none',
              }}
            >
              <div style={{ padding: '16px 32px', backgroundColor: 'rgba(20, 20, 20, 0.9)', borderRadius: 8 }}>
                Drop image to open in Compositor
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
