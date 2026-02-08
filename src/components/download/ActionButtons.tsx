import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfirmDialog } from '../common/ConfirmDialog';

interface ActionButtonsProps {
    downloadId: string;
    state: 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';
    onStateChange?: (newState: string) => void;
}

export const ActionButtons: React.FC<ActionButtonsProps> = ({
    downloadId,
    state,
    onStateChange,
}) => {
    const [loading, setLoading] = useState<string | null>(null);
    const [showCancelDialog, setShowCancelDialog] = useState(false);

    const handlePause = async () => {
        setLoading('pause');
        try {
            await invoke('pause_download', { downloadId });
            onStateChange?.('paused');
        } catch (error) {
            console.error('Failed to pause download:', error);
            alert(`Failed to pause: ${error}`);
        } finally {
            setLoading(null);
        }
    };

    const handleResume = async () => {
        setLoading('resume');
        try {
            await invoke('resume_download', { downloadId });
            onStateChange?.('active');
        } catch (error) {
            console.error('Failed to resume download:', error);
            alert(`Failed to resume: ${error}`);
        } finally {
            setLoading(null);
        }
    };

    const handleStop = async () => {
        setLoading('stop');
        try {
            await invoke('stop_download', { downloadId });
            onStateChange?.('stopped');
        } catch (error) {
            console.error('Failed to stop download:', error);
            alert(`Failed to stop: ${error}`);
        } finally {
            setLoading(null);
        }
    };

    const handleCancelClick = () => {
        setShowCancelDialog(true);
    };

    const handleCancelConfirm = async () => {
        setShowCancelDialog(false);
        setLoading('cancel');
        try {
            await invoke('cancel_download', { downloadId });
            onStateChange?.('cancelled');
        } catch (error) {
            console.error('Failed to cancel download:', error);
            alert(`Failed to cancel: ${error}`);
        } finally {
            setLoading(null);
        }
    };

    const handleCancelDialogClose = () => {
        setShowCancelDialog(false);
    };

    // Button component with loading state
    const Button: React.FC<{
        onClick: () => void;
        disabled: boolean;
        variant: 'primary' | 'success' | 'warning' | 'danger';
        children: React.ReactNode;
        loading?: boolean;
    }> = ({ onClick, disabled, variant, children, loading: isLoading }) => {
        const baseStyle = {
            padding: '8px 16px',
            borderRadius: '4px',
            border: 'none',
            fontSize: '14px',
            fontWeight: 500,
            cursor: disabled ? 'not-allowed' : 'pointer',
            opacity: disabled ? 0.5 : 1,
            transition: 'all 0.2s',
        };

        const variantStyles = {
            primary: { background: '#007acc', color: 'white' },
            success: { background: '#28a745', color: 'white' },
            warning: { background: '#ff9800', color: 'white' },
            danger: { background: '#dc3545', color: 'white' },
        };

        return (
            <button
                onClick={onClick}
                disabled={disabled}
                style={{ ...baseStyle, ...variantStyles[variant] }}
            >
                {isLoading ? '...' : children}
            </button>
        );
    };

    const isLoading = loading !== null;

    // Render buttons based on state
    if (state === 'active') {
        return (
            <>
                <div style={{ display: 'flex', gap: '8px' }}>
                    <Button
                        onClick={handlePause}
                        disabled={isLoading}
                        variant="primary"
                        loading={loading === 'pause'}
                    >
                        ⏸️ Pause
                    </Button>
                    <Button
                        onClick={handleStop}
                        disabled={isLoading}
                        variant="warning"
                        loading={loading === 'stop'}
                    >
                        ⏹️ Stop
                    </Button>
                    <Button
                        onClick={handleCancelClick}
                        disabled={isLoading}
                        variant="danger"
                        loading={loading === 'cancel'}
                    >
                        ❌ Cancel
                    </Button>
                </div>

                <ConfirmDialog
                    open={showCancelDialog}
                    title="Cancel Download?"
                    message="This will delete the partial file and cannot be undone. Are you sure you want to cancel this download?"
                    confirmText="Yes, Cancel Download"
                    cancelText="No, Keep Downloading"
                    onConfirm={handleCancelConfirm}
                    onCancel={handleCancelDialogClose}
                    variant="danger"
                />
            </>
        );
    }

    if (state === 'paused' || state === 'stopped') {
        return (
            <>
                <div style={{ display: 'flex', gap: '8px' }}>
                    <Button
                        onClick={handleResume}
                        disabled={isLoading}
                        variant="success"
                        loading={loading === 'resume'}
                    >
                        ▶️ Resume
                    </Button>
                    <Button
                        onClick={handleCancelClick}
                        disabled={isLoading}
                        variant="danger"
                        loading={loading === 'cancel'}
                    >
                        ❌ Cancel
                    </Button>
                </div>

                <ConfirmDialog
                    open={showCancelDialog}
                    title="Cancel Download?"
                    message="This will delete the partial file and cannot be undone. Are you sure you want to cancel this download?"
                    confirmText="Yes, Cancel Download"
                    cancelText="No, Keep Download"
                    onConfirm={handleCancelConfirm}
                    onCancel={handleCancelDialogClose}
                    variant="danger"
                />
            </>
        );
    }

    if (state === 'failed') {
        return (
            <>
                <div style={{ display: 'flex', gap: '8px' }}>
                    <Button
                        onClick={handleResume}
                        disabled={isLoading}
                        variant="primary"
                        loading={loading === 'resume'}
                    >
                        🔄 Retry
                    </Button>
                    <Button
                        onClick={handleCancelClick}
                        disabled={isLoading}
                        variant="danger"
                        loading={loading === 'cancel'}
                    >
                        ❌ Cancel
                    </Button>
                </div>

                <ConfirmDialog
                    open={showCancelDialog}
                    title="Cancel Download?"
                    message="This will delete the partial file. Are you sure you want to cancel this download?"
                    confirmText="Yes, Cancel Download"
                    cancelText="No, Keep Download"
                    onConfirm={handleCancelConfirm}
                    onCancel={handleCancelDialogClose}
                    variant="danger"
                />
            </>
        );
    }

    // No controls for completed or cancelled
    return null;
};
