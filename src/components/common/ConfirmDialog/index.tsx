import React from 'react';
import './style.css';

interface ConfirmDialogProps {
    open: boolean;
    title: string;
    message: string;
    confirmText: string;
    cancelText: string;
    onConfirm: () => void;
    onCancel: () => void;
    variant?: 'warning' | 'danger';
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
    open,
    title,
    message,
    confirmText,
    cancelText,
    onConfirm,
    onCancel,
    variant = 'warning',
}) => {
    if (!open) return null;

    const confirmButtonClass = variant === 'danger' ? 'btn-danger' : 'btn-warning';

    return (
        <div className="confirm-dialog-overlay">
            <div className="confirm-dialog-content">
                {/* Title */}
                <h3 className="confirm-dialog-title">
                    {title}
                </h3>

                {/* Message */}
                <p className="confirm-dialog-message">
                    {message}
                </p>

                {/* Buttons */}
                <div className="confirm-dialog-actions">
                    <button
                        onClick={onCancel}
                        className="btn-base btn-cancel"
                    >
                        {cancelText}
                    </button>
                    <button
                        onClick={onConfirm}
                        className={`btn-base ${confirmButtonClass}`}
                    >
                        {confirmText}
                    </button>
                </div>
            </div>
        </div>
    );
};
