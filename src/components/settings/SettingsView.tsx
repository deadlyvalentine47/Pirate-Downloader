import { useState } from 'react';
import { useDownloadStore } from '../../stores/downloadStore';
import './SettingsView.css';

export const SettingsView = () => {
    const { threads, setThreads } = useDownloadStore();
    const [defaultPath, setDefaultPath] = useState('');
    const [maxParallel, setMaxParallel] = useState(3);

    return (
        <div className="settings-view">
            <div className="settings-section">
                <h2 className="settings-section-title">General</h2>
                <div className="settings-card">
                    <SettingRow
                        label="Default Download Folder"
                        description="Where new downloads are saved by default"
                    >
                        <input
                            className="settings-input"
                            value={defaultPath}
                            onChange={e => setDefaultPath(e.target.value)}
                            placeholder="~/Downloads"
                        />
                    </SettingRow>
                </div>
            </div>

            <div className="settings-section">
                <h2 className="settings-section-title">Network</h2>
                <div className="settings-card">
                    <SettingRow
                        label="Max Parallel Downloads"
                        description="How many downloads can run simultaneously"
                    >
                        <div className="settings-number-row">
                            <input
                                type="range"
                                min={1}
                                max={10}
                                value={maxParallel}
                                onChange={e => setMaxParallel(Number(e.target.value))}
                                className="settings-slider"
                            />
                            <span className="settings-value-badge">{maxParallel}</span>
                        </div>
                    </SettingRow>

                    <div className="settings-divider" />

                    <SettingRow
                        label="Default Thread Count"
                        description="Threads per download for HTTP chunked downloads"
                    >
                        <div className="settings-number-row">
                            <input
                                type="range"
                                min={1}
                                max={32}
                                value={threads}
                                onChange={e => setThreads(Number(e.target.value))}
                                className="settings-slider"
                            />
                            <span className="settings-value-badge">{threads}</span>
                        </div>
                    </SettingRow>
                </div>
            </div>

            <div className="settings-section">
                <h2 className="settings-section-title">Advanced</h2>
                <div className="settings-card">
                    <SettingRow
                        label="HLS / DASH Stream Engine"
                        description="Use native stream downloader for .m3u8 / .mpd URLs"
                    >
                        <ToggleSwitch checked={true} onChange={() => { }} />
                    </SettingRow>
                </div>
            </div>
        </div>
    );
};

const SettingRow = ({ label, description, children }: {
    label: string;
    description: string;
    children: React.ReactNode;
}) => (
    <div className="setting-row">
        <div className="setting-meta">
            <span className="setting-label">{label}</span>
            <span className="setting-desc">{description}</span>
        </div>
        <div className="setting-control">{children}</div>
    </div>
);

const ToggleSwitch = ({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) => (
    <button
        className={`toggle-switch ${checked ? 'toggle-on' : ''}`}
        onClick={() => onChange(!checked)}
        aria-pressed={checked}
    >
        <span className="toggle-thumb" />
    </button>
);
