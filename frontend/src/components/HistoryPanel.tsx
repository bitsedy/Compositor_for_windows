import React from 'react';
import { History as HistoryIcon, RotateCcw } from 'lucide-react';

interface HistoryPanelProps {
  undoSteps: string[];
  redoSteps: string[];
  onJumpToState: (index: number) => void;
}

export const HistoryPanel: React.FC<HistoryPanelProps> = ({
  undoSteps,
  redoSteps,
  onJumpToState,
}) => {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: 160,
        backgroundColor: 'var(--bg-panel)',
        borderTop: '1px solid var(--border-color)',
      }}
    >
      <div
        style={{
          padding: '6px 10px',
          fontWeight: 'bold',
          fontSize: 11,
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}
      >
        <HistoryIcon size={13} />
        <span>HISTORY</span>
      </div>

      <div style={{ flex: 1, overflowY: 'auto' }}>
        {undoSteps.map((step, idx) => (
          <div
            key={`undo-${idx}`}
            onClick={() => onJumpToState(idx)}
            style={{
              padding: '4px 10px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              backgroundColor: idx === undoSteps.length - 1 ? 'var(--bg-active)' : 'transparent',
              color: idx === undoSteps.length - 1 ? 'var(--text-bright)' : 'var(--text-main)',
            }}
          >
            <RotateCcw size={12} />
            <span>{step}</span>
          </div>
        ))}

        {redoSteps.map((step, idx) => (
          <div
            key={`redo-${idx}`}
            style={{
              padding: '4px 10px',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              color: 'var(--text-muted)',
              fontStyle: 'italic',
            }}
          >
            <RotateCcw size={12} style={{ opacity: 0.5 }} />
            <span>{step}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
