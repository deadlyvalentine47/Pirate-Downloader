import React from 'react';

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

    const confirmButtonClass = variant === 'danger'
        ? 'bg-red-600 hover:bg-red-700 text-white'
        : 'bg-orange-600 hover:bg-orange-700 text-white';

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
            <div className="bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 p-6">
                {/* Title */}
                <h3 className="text-xl font-semibold text-white mb-4">
                    {title}
                </h3>

                {/* Message */}
                <p className="text-gray-300 mb-6">
                    {message}
                </p>

                {/* Buttons */}
                <div className="flex gap-3 justify-end">
                    <button
                        onClick={onCancel}
                        className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md transition-colors"
                    >
                        {cancelText}
                    </button>
                    <button
                        onClick={onConfirm}
                        className={`px-4 py-2 rounded-md transition-colors ${confirmButtonClass}`}
                    >
                        {confirmText}
                    </button>
                </div>
            </div>
        </div>
    );
};
