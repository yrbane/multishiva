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
    <div className="bg-gray-800 p-6 rounded-lg">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-semibold">Settings</h2>
        <div className="flex gap-2">
          <div className="text-xs text-gray-500 mr-2 self-center">
            Config: <span className="text-gray-400 font-mono text-[10px]">{configPath}</span>
          </div>
          <button
            onClick={handleLoad}
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm transition"
          >
            Reload
          </button>
          <button
            onClick={handleSave}
            className={`px-3 py-1 rounded text-sm transition ${
              saved
                ? 'bg-green-600 hover:bg-green-700'
                : 'bg-blue-600 hover:bg-blue-700'
            }`}
          >
            {saved ? '✓ Saved' : 'Save'}
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-4 border-b border-gray-700">
        {['general', 'hotkeys', 'behavior'].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab as typeof activeTab)}
            className={`px-4 py-2 text-sm font-medium transition ${
              activeTab === tab
                ? 'border-b-2 border-blue-500 text-blue-400'
                : 'text-gray-400 hover:text-gray-200'
            }`}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      {/* General Tab */}
      {activeTab === 'general' && (
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Machine Name
            </label>
            <input
              type="text"
              value={settings.self_name}
              onChange={(e) => setSettings({ ...settings, self_name: e.target.value })}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
              placeholder="e.g., host, agent1, laptop"
            />
            <p className="text-xs text-gray-500 mt-1">
              Unique identifier for this machine
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Mode</label>
            <select
              value={settings.mode}
              onChange={(e) =>
                setSettings({ ...settings, mode: e.target.value as 'host' | 'agent' })
              }
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
            >
              <option value="host">Host (Server)</option>
              <option value="agent">Agent (Client)</option>
            </select>
            <p className="text-xs text-gray-500 mt-1">
              {settings.mode === 'host'
                ? 'This machine accepts connections from agents'
                : 'This machine connects to a host'}
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Port</label>
            <input
              type="number"
              value={settings.port}
              onChange={(e) =>
                setSettings({ ...settings, port: parseInt(e.target.value) || 53421 })
              }
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
              min="1024"
              max="65535"
            />
            <p className="text-xs text-gray-500 mt-1">TCP port for connections (1024-65535)</p>
          </div>

          {settings.mode === 'agent' && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Host Address
              </label>
              <input
                type="text"
                value={settings.host_address || ''}
                onChange={(e) => setSettings({ ...settings, host_address: e.target.value || null })}
                className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
                placeholder="192.168.1.100:53421"
              />
              <p className="text-xs text-gray-500 mt-1">
                IP address and port of the host machine
              </p>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Pre-Shared Key (PSK)
            </label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <input
                  type={showPsk ? 'text' : 'password'}
                  value={settings.tls.psk}
                  onChange={(e) => setSettings({ ...settings, tls: { psk: e.target.value } })}
                  className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white pr-10"
                  placeholder="Enter secure PSK..."
                />
                <button
                  onClick={() => setShowPsk(!showPsk)}
                  className="absolute right-2 top-2 text-gray-400 hover:text-gray-200"
                >
                  {showPsk ? '👁️' : '👁️‍🗨️'}
                </button>
              </div>
              <button
                onClick={generatePsk}
                className="px-3 py-2 bg-purple-600 hover:bg-purple-700 rounded text-sm whitespace-nowrap"
              >
                Generate
              </button>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Shared secret for TLS encryption (stored securely in system keyring)
            </p>
          </div>
        </div>
      )}

      {/* Hotkeys Tab */}
      {activeTab === 'hotkeys' && (
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
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
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
              placeholder="e.g., Ctrl+Alt+H"
            />
            <p className="text-xs text-gray-500 mt-1">
              Hotkey to return focus to this machine
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
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
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
              placeholder="e.g., Ctrl+Alt+K"
            />
            <p className="text-xs text-gray-500 mt-1">
              Emergency hotkey to stop all MultiShiva operations
            </p>
          </div>

          <div className="bg-gray-900 border border-gray-700 rounded-lg p-4 mt-4">
            <h3 className="text-sm font-semibold mb-2">Hotkey Format</h3>
            <ul className="text-xs text-gray-400 space-y-1">
              <li>• Modifiers: Ctrl, Alt, Shift, Meta (Windows/Command)</li>
              <li>• Combine with +, e.g., Ctrl+Alt+Key</li>
              <li>• Use descriptive key names: A-Z, F1-F12, Space, etc.</li>
            </ul>
          </div>
        </div>
      )}

      {/* Behavior Tab */}
      {activeTab === 'behavior' && (
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Edge Threshold (pixels)
            </label>
            <div className="flex items-center gap-4">
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
                className="flex-1"
              />
              <span className="text-white font-mono w-12 text-right">
                {settings.behavior?.edge_threshold_px || 10}px
              </span>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Distance from screen edge to trigger machine switch
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Friction (milliseconds)
            </label>
            <div className="flex items-center gap-4">
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
                className="flex-1"
              />
              <span className="text-white font-mono w-12 text-right">
                {settings.behavior?.friction_ms || 100}ms
              </span>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Delay before switching to prevent accidental triggers
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Reconnect Delay (milliseconds)
            </label>
            <div className="flex items-center gap-4">
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
                className="flex-1"
              />
              <span className="text-white font-mono w-16 text-right">
                {((settings.behavior?.reconnect_delay_ms || 5000) / 1000).toFixed(1)}s
              </span>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Wait time before attempting to reconnect after disconnection
            </p>
          </div>
        </div>
      )}
    </div>
  )
}
