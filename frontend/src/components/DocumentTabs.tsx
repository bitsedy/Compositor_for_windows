import React from 'react';
import { X } from 'lucide-react';

interface TabItem {
  id: string;
  name: string;
  isDirty?: boolean;
}

interface DocumentTabsProps {
  tabs: TabItem[];
  activeTabId: string;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
}

export const DocumentTabs: React.FC<DocumentTabsProps> = ({
  tabs,
  activeTabId,
  onSelectTab,
  onCloseTab,
}) => {
  return (
    <div
      style={{
        height: 28,
        backgroundColor: 'var(--bg-panel)',
        borderBottom: '1px solid var(--border-color)',
        display: 'flex',
        alignItems: 'center',
        paddingLeft: 6,
        gap: 2,
        overflowX: 'auto',
      }}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            onClick={() => onSelectTab(tab.id)}
            style={{
              height: 26,
              display: 'flex',
              alignItems: 'center',
              padding: '0 10px',
              backgroundColor: isActive ? 'var(--bg-app)' : 'var(--bg-panel-header)',
              color: isActive ? 'var(--text-bright)' : 'var(--text-muted)',
              borderTopLeftRadius: 4,
              borderTopRightRadius: 4,
              border: '1px solid var(--border-color)',
              borderBottom: isActive ? 'none' : '1px solid var(--border-color)',
              cursor: 'pointer',
              gap: 8,
              fontSize: 11,
              maxWidth: 160,
            }}
          >
            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {tab.name}
              {tab.isDirty && ' *'}
            </span>

            <button
              onClick={(e) => {
                e.stopPropagation();
                onCloseTab(tab.id);
              }}
              style={{
                width: 14,
                height: 14,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: '50%',
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLElement).style.backgroundColor = 'rgba(255,255,255,0.2)';
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
              }}
            >
              <X size={10} />
            </button>
          </div>
        );
      })}
    </div>
  );
};
