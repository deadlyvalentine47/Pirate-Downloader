import { useEffect, useRef } from 'react';
import './ContextMenu.css';

export interface ContextMenuItem {
    label: string;
    icon: string;
    onClick: () => void;
    danger?: boolean;
    disabled?: boolean;
    divider?: boolean;
}

interface ContextMenuProps {
    x: number;
    y: number;
    items: ContextMenuItem[];
    onClose: () => void;
}

export const ContextMenu = ({ x, y, items, onClose }: ContextMenuProps) => {
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClick = (e: MouseEvent) => {
            if (ref.current && !ref.current.contains(e.target as Node)) {
                onClose();
            }
        };
        const handleKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        document.addEventListener('mousedown', handleClick);
        document.addEventListener('keydown', handleKey);
        return () => {
            document.removeEventListener('mousedown', handleClick);
            document.removeEventListener('keydown', handleKey);
        };
    }, [onClose]);

    // Clamp to viewport
    const style: React.CSSProperties = {
        position: 'fixed',
        left: Math.min(x, window.innerWidth - 180),
        top: Math.min(y, window.innerHeight - items.length * 36 - 16),
        zIndex: 9999,
    };

    return (
        <div ref={ref} className="ctx-menu" style={style}>
            {items.map((item, i) =>
                item.divider ? (
                    <div key={i} className="ctx-divider" />
                ) : (
                    <button
                        key={i}
                        className={`ctx-item ${item.danger ? 'ctx-item-danger' : ''}`}
                        onClick={() => { item.onClick(); onClose(); }}
                        disabled={item.disabled}
                    >
                        <span className="ctx-icon">{item.icon}</span>
                        <span>{item.label}</span>
                    </button>
                )
            )}
        </div>
    );
};
