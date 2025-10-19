// SettingsPanel Component - Configuration management
import { useState } from 'react'

interface Settings {
  version: number
  selfName: string
  mode: 'host' | 'agent'
  port: number
  hostAddress: string
  psk: string
  hotkeys: {
    focusReturn: string
    killSwitch: string
  }
  behavior: {
    edgeThreshold: number
    friction: number
    reconnectDelay: number
  }
  features: {
    mdnsDiscovery: boolean
    clipboardSync: boolean
  }
}

export default function SettingsPanel() {
  const [settings, setSettings] = useState<Settings>({
    version: 1,
    selfName: 'multishiva',
    mode: 'host',
    port: 53421,
    hostAddress: '',
    psk: '',
    hotkeys: {
      focusReturn: 'Ctrl+Alt+H',
      killSwitch: 'Ctrl+Alt+K',
    },
    behavior: {
      edgeThreshold: 10,
      friction: 100,
      reconnectDelay: 5000,
    },
    features: {
      mdnsDiscovery: true,
      clipboardSync: true,
    },
  })

  const [showPsk, setShowPsk] = useState(false)
  const [activeTab, setActiveTab] = useState<'general' | 'hotkeys' | 'behavior' | 'features'>('general')
  const [saved, setSaved] = useState(false)

  const handleSave = () => {
    // In real implementation, this would call Tauri commands to save config
    console.log('Saving settings:', settings)
    setSaved(true)
    setTimeout(() => setSaved(false), 3000)
  }

  const handleLoad = () => {
    // In real implementation, this would call Tauri commands to load config
    console.log('Loading settings...')
  }

  const generatePsk = () => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*'
    let psk = ''
    for (let i = 0; i < 32; i++) {
      psk += chars.charAt(Math.floor(Math.random() * chars.length))
    }
    setSettings({ ...settings, psk })
  }

  return (
    <div className="bg-gray-800 p-6 rounded-lg">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-semibold">Settings</h2>
        <div className="flex gap-2">
          <button
            onClick={handleLoad}
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm transition"
          >
            Load
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
        {['general', 'hotkeys', 'behavior', 'features'].map((tab) => (
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
              value={settings.selfName}
              onChange={(e) => setSettings({ ...settings, selfName: e.target.value })}
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
                value={settings.hostAddress}
                onChange={(e) => setSettings({ ...settings, hostAddress: e.target.value })}
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
                  value={settings.psk}
                  onChange={(e) => setSettings({ ...settings, psk: e.target.value })}
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
              value={settings.hotkeys.focusReturn}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  hotkeys: { ...settings.hotkeys, focusReturn: e.target.value },
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
              value={settings.hotkeys.killSwitch}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  hotkeys: { ...settings.hotkeys, killSwitch: e.target.value },
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
                value={settings.behavior.edgeThreshold}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    behavior: {
                      ...settings.behavior,
                      edgeThreshold: parseInt(e.target.value),
                    },
                  })
                }
                className="flex-1"
              />
              <span className="text-white font-mono w-12 text-right">
                {settings.behavior.edgeThreshold}px
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
                value={settings.behavior.friction}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    behavior: {
                      ...settings.behavior,
                      friction: parseInt(e.target.value),
                    },
                  })
                }
                className="flex-1"
              />
              <span className="text-white font-mono w-12 text-right">
                {settings.behavior.friction}ms
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
                value={settings.behavior.reconnectDelay}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    behavior: {
                      ...settings.behavior,
                      reconnectDelay: parseInt(e.target.value),
                    },
                  })
                }
                className="flex-1"
              />
              <span className="text-white font-mono w-16 text-right">
                {(settings.behavior.reconnectDelay / 1000).toFixed(1)}s
              </span>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Wait time before attempting to reconnect after disconnection
            </p>
          </div>
        </div>
      )}

      {/* Features Tab */}
      {activeTab === 'features' && (
        <div className="space-y-4">
          <div className="bg-gray-900 border border-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-semibold text-white">mDNS Auto-Discovery</h3>
                <p className="text-xs text-gray-400 mt-1">
                  Automatically discover other MultiShiva instances on the network
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.features.mdnsDiscovery}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      features: {
                        ...settings.features,
                        mdnsDiscovery: e.target.checked,
                      },
                    })
                  }
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>
          </div>

          <div className="bg-gray-900 border border-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-semibold text-white">Clipboard Synchronization</h3>
                <p className="text-xs text-gray-400 mt-1">
                  Share clipboard content across connected machines
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.features.clipboardSync}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      features: {
                        ...settings.features,
                        clipboardSync: e.target.checked,
                      },
                    })
                  }
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>
          </div>

          <div className="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4 mt-4">
            <h3 className="text-sm font-semibold text-blue-400 mb-2">Feature Info</h3>
            <ul className="text-xs text-gray-300 space-y-1">
              <li>
                • <strong>mDNS:</strong> Uses Multicast DNS for zero-config discovery
              </li>
              <li>
                • <strong>Clipboard:</strong> Polls every 500ms for changes
              </li>
              <li>
                • All features can be toggled without restarting
              </li>
            </ul>
          </div>
        </div>
      )}
    </div>
  )
}
