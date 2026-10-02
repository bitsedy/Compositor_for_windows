import React, { useState, useEffect, useRef } from 'react';

interface MenuBarProps {
  onNew: () => void;
  onOpen: () => void;
  onSave: () => void;
  onExport: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onCut: () => void;
  onCopy: () => void;
  onPaste: () => void;
  onSelectAll: () => void;
  onDeselect: () => void;
  onInvertSelection: () => void;
  onOpenFilter: (kind: string) => void;
  onOpenAdjustment: (kind: string) => void;
}

export const MenuBar: React.FC<MenuBarProps> = ({
  onNew,
  onOpen,
  onSave,
  onExport,
  onUndo,
  onRedo,
  onCut,
  onCopy,
  onPaste,
  onSelectAll,
  onDeselect,
  onInvertSelection,
  onOpenFilter,
  onOpenAdjustment,
}) => {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setActiveMenu(null);
      }
    };
    window.addEventListener('mousedown', handleClickOutside);
    return () => window.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const menus: Record<string, { label: string; shortcut?: string; action?: () => void; divider?: boolean }[]> = {
    File: [
      { label: 'New...', shortcut: 'Ctrl+N', action: onNew },
      { label: 'Open...', shortcut: 'Ctrl+O', action: onOpen },
      { label: 'Save', shortcut: 'Ctrl+S', action: onSave },
      { label: 'Export as JPEG...', shortcut: 'Ctrl+Shift+E', action: onExport },
      { divider: true, label: '' },
      { label: 'Close', shortcut: 'Ctrl+W' },
    ],
    Edit: [
      { label: 'Undo', shortcut: 'Ctrl+Z', action: onUndo },
      { label: 'Redo', shortcut: 'Ctrl+Y', action: onRedo },
      { divider: true, label: '' },
      { label: 'Cut', shortcut: 'Ctrl+X', action: onCut },
      { label: 'Copy', shortcut: 'Ctrl+C', action: onCopy },
      { label: 'Copy Merged', shortcut: 'Ctrl+Shift+C' },
      { label: 'Paste', shortcut: 'Ctrl+V', action: onPaste },
      { divider: true, label: '' },
      { label: 'Content-Aware Fill...', action: () => onOpenFilter('Content-Aware Fill') },
      { label: 'Free Transform', shortcut: 'Ctrl+T' },
    ],
    Image: [
      { label: 'Curves...', shortcut: 'Ctrl+M', action: () => onOpenAdjustment('Curves') },
      { label: 'Levels...', shortcut: 'Ctrl+L', action: () => onOpenAdjustment('Levels') },
      { label: 'Hue/Saturation...', shortcut: 'Ctrl+U', action: () => onOpenAdjustment('Hue/Saturation') },
      { label: 'Exposure...', action: () => onOpenAdjustment('Exposure') },
      { label: 'Gradient Map...', action: () => onOpenAdjustment('Gradient Map') },
      { label: 'Black & White...', shortcut: 'Alt+Shift+Ctrl+B', action: () => onOpenAdjustment('Black & White') },
      { label: 'Color Balance...', shortcut: 'Ctrl+B', action: () => onOpenAdjustment('Color Balance') },
      { label: 'Invert', shortcut: 'Ctrl+I', action: () => onOpenAdjustment('Invert') },
      { divider: true, label: '' },
      { label: 'Canvas Size...' },
      { label: 'Image Size...' },
      { label: 'Trim...' },
    ],
    Layer: [
      { label: 'New Layer', shortcut: 'Ctrl+Shift+N' },
      { label: 'New Group', shortcut: 'Ctrl+G' },
      { label: 'Duplicate Layer', shortcut: 'Ctrl+J' },
      { label: 'Delete Layer' },
      { divider: true, label: '' },
      { label: 'Create Clipping Mask', shortcut: 'Ctrl+Alt+G' },
      { label: 'Add Layer Mask' },
      { label: 'Layer Effects...' },
      { divider: true, label: '' },
      { label: 'Merge Down', shortcut: 'Ctrl+E' },
      { label: 'Flatten Image' },
    ],
    Select: [
      { label: 'All', shortcut: 'Ctrl+A', action: onSelectAll },
      { label: 'Deselect', shortcut: 'Ctrl+D', action: onDeselect },
      { label: 'Reselect', shortcut: 'Ctrl+Shift+D' },
      { label: 'Inverse', shortcut: 'Ctrl+Shift+I', action: onInvertSelection },
      { divider: true, label: '' },
      { label: 'Select Subject', action: () => onOpenFilter('Select Subject') },
      { label: 'Color Range...', action: () => onOpenFilter('Color Range') },
      { divider: true, label: '' },
      { label: 'Modify > Expand...' },
      { label: 'Modify > Contract...' },
      { label: 'Modify > Feather...' },
    ],
    Filter: [
      { label: 'Gaussian Blur...', action: () => onOpenFilter('Gaussian Blur') },
      { label: 'Motion Blur...', action: () => onOpenFilter('Motion Blur') },
      { label: 'Add Noise...', action: () => onOpenFilter('Add Noise') },
      { label: 'Vignette...', action: () => onOpenFilter('Vignette') },
      { label: 'Bloom / Glow...', action: () => onOpenFilter('Bloom / Glow') },
      { label: 'Dither...', action: () => onOpenFilter('Dither') },
      { label: 'Tonal Contrast...', action: () => onOpenFilter('Tonal Contrast') },
      { label: 'Lens Correction...', action: () => onOpenFilter('Lens Correction') },
      { divider: true, label: '' },
      { label: 'Camera Raw Filter...', shortcut: 'Ctrl+Shift+A', action: () => onOpenFilter('Camera Raw') },
    ],
    View: [
      { label: 'Zoom In', shortcut: 'Ctrl+=' },
      { label: 'Zoom Out', shortcut: 'Ctrl+-' },
      { label: 'Fit on Screen', shortcut: 'Ctrl+0' },
      { label: 'Actual Pixels (100%)', shortcut: 'Ctrl+1' },
      { divider: true, label: '' },
      { label: 'Rulers', shortcut: 'Ctrl+R' },
      { label: 'Grid', shortcut: 'Ctrl+\'' },
      { label: 'Snap', shortcut: 'Ctrl+Shift+;' },
    ],
    Help: [
      { label: 'Keyboard Shortcuts' },
      { label: 'About Compositor' },
    ],
  };

  return (
    <div
      ref={menuRef}
      style={{
        display: 'flex',
        alignItems: 'center',
        height: 28,
        backgroundColor: 'var(--bg-panel-header)',
        borderBottom: '1px solid var(--border-color)',
        paddingLeft: 8,
        zIndex: 100,
        position: 'relative',
      }}
    >
      <div style={{ fontWeight: 'bold', marginRight: 14, color: 'var(--text-bright)' }}>
        Compositor
      </div>

      {Object.entries(menus).map(([name, items]) => (
        <div key={name} style={{ position: 'relative' }}>
          <button
            onClick={() => setActiveMenu(activeMenu === name ? null : name)}
            onMouseEnter={() => activeMenu && setActiveMenu(name)}
            style={{
              padding: '4px 10px',
              color: activeMenu === name ? 'var(--text-bright)' : 'var(--text-main)',
              backgroundColor: activeMenu === name ? 'var(--bg-active)' : 'transparent',
              borderRadius: 3,
            }}
          >
            {name}
          </button>

          {activeMenu === name && (
            <div
              style={{
                position: 'absolute',
                top: 28,
                left: 0,
                backgroundColor: 'var(--bg-panel)',
                border: '1px solid var(--border-color)',
                boxShadow: '0 4px 12px rgba(0,0,0,0.5)',
                minWidth: 200,
                padding: '4px 0',
                borderRadius: 4,
                zIndex: 200,
              }}
            >
              {items.map((item, idx) =>
                item.divider ? (
                  <div
                    key={idx}
                    style={{
                      height: 1,
                      backgroundColor: 'var(--border-color)',
                      margin: '4px 0',
                    }}
                  />
                ) : (
                  <button
                    key={idx}
                    onClick={() => {
                      setActiveMenu(null);
                      item.action?.();
                    }}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      width: '100%',
                      padding: '5px 16px',
                      textAlign: 'left',
                    }}
                    onMouseEnter={(e) => {
                      (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-active)';
                      (e.currentTarget as HTMLElement).style.color = 'var(--text-bright)';
                    }}
                    onMouseLeave={(e) => {
                      (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
                      (e.currentTarget as HTMLElement).style.color = 'var(--text-main)';
                    }}
                  >
                    <span>{item.label}</span>
                    {item.shortcut && (
                      <span style={{ color: 'var(--text-muted)', marginLeft: 16 }}>
                        {item.shortcut}
                      </span>
                    )}
                  </button>
                )
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};
