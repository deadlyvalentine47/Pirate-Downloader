import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfirmDialog } from '../../common/ConfirmDialog';
import './style.css';

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



    const isLoading = loading !== null;

    // Render buttons based on state
    if (state === 'active') {
        return (
            <>
                <div className="action-buttons-container">
                    <button
                        className="action-btn btn-pause"
                        onClick={handlePause}
                        disabled={isLoading}
                    >
                        {loading === 'pause' ? '...' : '⏸️ Pause'}
                    </button>
                    <button
                        className="action-btn btn-stop"
                        onClick={handleStop}
                        disabled={isLoading}
                    >
                        {loading === 'stop' ? '...' : '⏹️ Stop'}
                    </button>
                    <button
                        className="action-btn btn-cancel"
                        onClick={handleCancelClick}
                        disabled={isLoading}
                    >
                        {loading === 'cancel' ? '...' : '❌ Cancel'}
                    </button>
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
                <div className="action-buttons-container">
                    <button
                        className="action-btn btn-resume"
                        onClick={handleResume}
                        disabled={isLoading}
                    >
                        {loading === 'resume' ? '...' : '▶️ Resume'}
                    </button>
                    <button
                        className="action-btn btn-cancel"
                        onClick={handleCancelClick}
                        disabled={isLoading}
                    >
                        {loading === 'cancel' ? '...' : '❌ Cancel'}
                    </button>
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
                <div className="action-buttons-container">
                    <button
                        className="action-btn btn-resume"
                        onClick={handleResume}
                        disabled={isLoading}
                    >
                        {loading === 'resume' ? '...' : '🔄 Retry'}
                    </button>
                    <button
                        className="action-btn btn-cancel"
                        onClick={handleCancelClick}
                        disabled={isLoading}
                    >
                        {loading === 'cancel' ? '...' : '❌ Cancel'}
                    </button>
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
