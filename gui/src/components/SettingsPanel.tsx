// SettingsPanel Component - Configuration management
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface Settings {
  version: number
  self_name: string
  mode: 'host' | 'agent'
  port: number
  host_address: string | null
  tls: {
    psk: string
  }
  edges: Record<string, string>
  hotkeys: {
    focus_return: string | null
    kill_switch: string | null
  } | null
  behavior: {
    edge_threshold_px: number | null
    friction_ms: number | null
    reconnect_delay_ms: number | null
  } | null
}

export default function SettingsPanel() {
  const [settings, setSettings] = useState<Settings>({
    version: 1,
    self_name: 'multishiva',
    mode: 'host',
    port: 53421,
    host_address: null,
    tls: {
      psk: '',
    },
    edges: {},
    hotkeys: {
      focus_return: 'Ctrl+Alt+H',
      kill_switch: 'Ctrl+Alt+K',
    },
    behavior: {
      edge_threshold_px: 10,
      friction_ms: 100,
      reconnect_delay_ms: 5000,
    },
  })

  const [showPsk, setShowPsk] = useState(false)
  const [activeTab, setActiveTab] = useState<'general' | 'hotkeys' | 'behavior'>('general')
  const [saved, setSaved] = useState(false)
  const [configPath, setConfigPath] = useState<string>('')

  useEffect(() => {
    // Load config path and initial settings
    invoke<string>('get_config_path')
      .then((path) => setConfigPath(path))
      .catch((err) => console.error('Failed to get config path:', err))

    handleLoad()
  }, [])

  const handleSave = async () => {
    try {
      await invoke('save_config', { config: settings, customPath: null })
      setSaved(true)
      setTimeout(() => setSaved(false), 3000)
    } catch (err) {
      console.error('Failed to save config:', err)
      alert(`Failed to save configuration: ${err}`)
    }
  }

  const handleLoad = async () => {
    try {
      const config = await invoke<Settings>('load_config', { customPath: null })
      setSettings(config)
    } catch (err) {
      console.error('Failed to load config:', err)
      // Keep default settings if load fails
    }
  }

  const generatePsk = () => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*'
    let psk = ''
    for (let i = 0; i < 32; i++) {
      psk += chars.charAt(Math.floor(Math.random() * chars.length))
    }
    setSettings({ ...settings, tls: { psk } })
  }

  return (
    <div className="backdrop-blur-xl bg-white/5 p-8 rounded-3xl border border-white/10 shadow-2xl">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-3xl font-bold bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent">
            Configuration
          </h2>
          <p className="text-slate-400 text-xs mt-2 font-mono">{configPath || 'Loading...'}</p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={handleLoad}
            className="px-5 py-2.5 backdrop-blur-sm bg-white/10 hover:bg-white/20 border border-white/20 rounded-xl text-sm font-medium transition-all duration-200 hover:scale-105 flex items-center gap-2"
          >
            <span>🔄</span>
            Reload
          </button>
          <button
            onClick={handleSave}
            className={`px-5 py-2.5 rounded-xl text-sm font-semibold transition-all duration-300 hover:scale-105 flex items-center gap-2 ${
              saved
                ? 'bg-gradient-to-r from-emerald-600 to-green-600 shadow-lg shadow-emerald-500/30'
                : 'bg-gradient-to-r from-purple-600 to-cyan-600 shadow-lg shadow-purple-500/30'
            }`}
          >
            {saved ? (
              <>
                <span>✓</span>
                Saved
              </>
            ) : (
              <>
                <span>💾</span>
                Save
              </>
            )}
          </button>
        </div>
      </div>

      {/* Tabs with modern design */}
      <div className="flex gap-2 mb-6 p-1.5 rounded-2xl bg-white/5 backdrop-blur-sm border border-white/10">
        {[
          { key: 'general', icon: '⚡', label: 'General' },
          { key: 'hotkeys', icon: '⌨️', label: 'Hotkeys' },
          { key: 'behavior', icon: '🎯', label: 'Behavior' }
        ].map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key as typeof activeTab)}
            className={`flex-1 px-6 py-3 rounded-xl font-semibold transition-all duration-300 flex items-center justify-center gap-2 ${
              activeTab === tab.key
                ? 'bg-gradient-to-r from-purple-600 to-cyan-600 text-white shadow-lg shadow-purple-500/50'
                : 'text-slate-300 hover:text-white hover:bg-white/10'
            }`}
          >
            <span className="text-lg">{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {/* General Tab */}
      {activeTab === 'general' && (
        <div className="space-y-6">
          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
              <span>🏷️</span>
              Machine Name
            </label>
            <input
              type="text"
              value={settings.self_name}
              onChange={(e) => setSettings({ ...settings, self_name: e.target.value })}
              className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all"
              placeholder="e.g., host, agent1, laptop"
            />
            <p className="text-xs text-slate-400 mt-2 ml-1">
              Unique identifier for this machine
            </p>
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
              <span>🎭</span>
              Mode
            </label>
            <select
              value={settings.mode}
              onChange={(e) =>
                setSettings({ ...settings, mode: e.target.value as 'host' | 'agent' })
              }
              className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all cursor-pointer"
            >
              <option value="host" className="bg-slate-800">👑 Host (Server)</option>
              <option value="agent" className="bg-slate-800">💻 Agent (Client)</option>
            </select>
            <p className="text-xs text-slate-400 mt-2 ml-1">
              {settings.mode === 'host'
                ? '📡 This machine accepts connections from agents'
                : '🔌 This machine connects to a host'}
            </p>
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
              <span>🔌</span>
              Port
            </label>
            <input
              type="number"
              value={settings.port}
              onChange={(e) =>
                setSettings({ ...settings, port: parseInt(e.target.value) || 53421 })
              }
              className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all"
              min="1024"
              max="65535"
            />
            <p className="text-xs text-slate-400 mt-2 ml-1">TCP port for connections (1024-65535)</p>
          </div>

          {settings.mode === 'agent' && (
            <div className="animate-fade-in">
              <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
                <span>🌐</span>
                Host Address
              </label>
              <input
                type="text"
                value={settings.host_address || ''}
                onChange={(e) => setSettings({ ...settings, host_address: e.target.value || null })}
                className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:border-transparent transition-all"
                placeholder="192.168.1.100:53421"
              />
              <p className="text-xs text-slate-400 mt-2 ml-1">
                IP address and port of the host machine
              </p>
            </div>
          )}

          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
              <span>🔐</span>
              Pre-Shared Key (PSK)
            </label>
            <div className="flex gap-3">
              <div className="relative flex-1">
                <input
                  type={showPsk ? 'text' : 'password'}
                  value={settings.tls.psk}
                  onChange={(e) => setSettings({ ...settings, tls: { psk: e.target.value } })}
                  className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all pr-12"
                  placeholder="Enter secure PSK..."
                />
                <button
                  onClick={() => setShowPsk(!showPsk)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white transition-colors p-1 rounded-lg hover:bg-white/10"
                >
                  {showPsk ? '👁️' : '🔒'}
                </button>
              </div>
              <button
                onClick={generatePsk}
                className="px-5 py-3 bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 rounded-xl font-semibold shadow-lg shadow-purple-500/30 transition-all duration-200 hover:scale-105 whitespace-nowrap flex items-center gap-2"
              >
                <span>✨</span>
                Generate
              </button>
            </div>
            <p className="text-xs text-slate-400 mt-2 ml-1">
              🔒 Shared secret for TLS encryption (stored securely in system keyring)
            </p>
          </div>
        </div>
      )}

      {/* Hotkeys Tab */}
      {activeTab === 'hotkeys' && (
        <div className="space-y-6">
          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
              <span>↩️</span>
              Focus Return
            </label>
            <input
              type="text"
              value={settings.hotkeys?.focus_return || ''}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  hotkeys: {
                    focus_return: e.target.value || null,
                    kill_switch: settings.hotkeys?.kill_switch || null
                  },
                })
              }
              className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all font-mono"
              placeholder="e.g., Ctrl+Alt+H"
            />
            <p className="text-xs text-slate-400 mt-2 ml-1">
              ⌨️ Hotkey to return focus to this machine
            </p>
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-3 flex items-center gap-2">
              <span>🛑</span>
              Kill Switch
            </label>
            <input
              type="text"
              value={settings.hotkeys?.kill_switch || ''}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  hotkeys: {
                    focus_return: settings.hotkeys?.focus_return || null,
                    kill_switch: e.target.value || null
                  },
                })
              }
              className="w-full backdrop-blur-sm bg-white/10 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent transition-all font-mono"
              placeholder="e.g., Ctrl+Alt+K"
            />
            <p className="text-xs text-slate-400 mt-2 ml-1">
              ⚠️ Emergency hotkey to stop all MultiShiva operations
            </p>
          </div>

          <div className="backdrop-blur-xl bg-gradient-to-br from-purple-500/10 to-cyan-500/10 border border-purple-500/30 rounded-2xl p-6 mt-6">
            <h3 className="text-sm font-bold text-purple-300 mb-3 flex items-center gap-2">
              <span>📖</span>
              Hotkey Format
            </h3>
            <ul className="text-xs text-slate-300 space-y-2">
              <li>• Modifiers: Ctrl, Alt, Shift, Meta (Windows/Command)</li>
              <li>• Combine with +, e.g., Ctrl+Alt+Key</li>
              <li>• Use descriptive key names: A-Z, F1-F12, Space, etc.</li>
            </ul>
          </div>
        </div>
      )}

      {/* Behavior Tab */}
      {activeTab === 'behavior' && (
        <div className="space-y-8">
          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-4 flex items-center gap-2">
              <span>📏</span>
              Edge Threshold
            </label>
            <div className="backdrop-blur-xl bg-white/5 border border-white/10 rounded-2xl p-6">
              <div className="flex items-center gap-6">
                <input
                  type="range"
                  min="1"
                  max="50"
                  value={settings.behavior?.edge_threshold_px || 10}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      behavior: {
                        edge_threshold_px: parseInt(e.target.value),
                        friction_ms: settings.behavior?.friction_ms || null,
                        reconnect_delay_ms: settings.behavior?.reconnect_delay_ms || null,
                      },
                    })
                  }
                  className="flex-1 h-3 bg-purple-500/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-6 [&::-webkit-slider-thumb]:h-6 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-gradient-to-r [&::-webkit-slider-thumb]:from-purple-500 [&::-webkit-slider-thumb]:to-cyan-500 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-purple-500/50 [&::-webkit-slider-thumb]:transition-all [&::-webkit-slider-thumb]:hover:scale-110"
                />
                <span className="text-3xl font-bold bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent w-24 text-right">
                  {settings.behavior?.edge_threshold_px || 10}<span className="text-xl">px</span>
                </span>
              </div>
              <p className="text-xs text-slate-400 mt-3">
                Distance from screen edge to trigger machine switch
              </p>
            </div>
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-4 flex items-center gap-2">
              <span>⏱️</span>
              Friction Delay
            </label>
            <div className="backdrop-blur-xl bg-white/5 border border-white/10 rounded-2xl p-6">
              <div className="flex items-center gap-6">
                <input
                  type="range"
                  min="0"
                  max="500"
                  step="50"
                  value={settings.behavior?.friction_ms || 100}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      behavior: {
                        edge_threshold_px: settings.behavior?.edge_threshold_px || null,
                        friction_ms: parseInt(e.target.value),
                        reconnect_delay_ms: settings.behavior?.reconnect_delay_ms || null,
                      },
                    })
                  }
                  className="flex-1 h-3 bg-cyan-500/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-6 [&::-webkit-slider-thumb]:h-6 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-gradient-to-r [&::-webkit-slider-thumb]:from-cyan-500 [&::-webkit-slider-thumb]:to-blue-500 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-cyan-500/50 [&::-webkit-slider-thumb]:transition-all [&::-webkit-slider-thumb]:hover:scale-110"
                />
                <span className="text-3xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent w-24 text-right">
                  {settings.behavior?.friction_ms || 100}<span className="text-xl">ms</span>
                </span>
              </div>
              <p className="text-xs text-slate-400 mt-3">
                Delay before switching to prevent accidental triggers
              </p>
            </div>
          </div>

          <div>
            <label className="block text-sm font-semibold text-slate-200 mb-4 flex items-center gap-2">
              <span>🔄</span>
              Reconnect Delay
            </label>
            <div className="backdrop-blur-xl bg-white/5 border border-white/10 rounded-2xl p-6">
              <div className="flex items-center gap-6">
                <input
                  type="range"
                  min="1000"
                  max="30000"
                  step="1000"
                  value={settings.behavior?.reconnect_delay_ms || 5000}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      behavior: {
                        edge_threshold_px: settings.behavior?.edge_threshold_px || null,
                        friction_ms: settings.behavior?.friction_ms || null,
                        reconnect_delay_ms: parseInt(e.target.value),
                      },
                    })
                  }
                  className="flex-1 h-3 bg-indigo-500/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-6 [&::-webkit-slider-thumb]:h-6 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-gradient-to-r [&::-webkit-slider-thumb]:from-indigo-500 [&::-webkit-slider-thumb]:to-purple-500 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-indigo-500/50 [&::-webkit-slider-thumb]:transition-all [&::-webkit-slider-thumb]:hover:scale-110"
                />
                <span className="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent w-24 text-right">
                  {((settings.behavior?.reconnect_delay_ms || 5000) / 1000).toFixed(1)}<span className="text-xl">s</span>
                </span>
              </div>
              <p className="text-xs text-slate-400 mt-3">
                Wait time before attempting to reconnect after disconnection
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
