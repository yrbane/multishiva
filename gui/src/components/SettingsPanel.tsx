// SettingsPanel Component - Configuration management
import { useState } from 'react'

interface Settings {
  psk: string
  port: number
  mode: 'host' | 'agent'
}

export default function SettingsPanel() {
  const [settings, setSettings] = useState<Settings>({
    psk: '',
    port: 53421,
    mode: 'host',
  })

  return (
    <div className="bg-gray-800 p-6 rounded-lg">
      <h2 className="text-2xl font-semibold mb-4">Settings</h2>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Mode
          </label>
          <select
            value={settings.mode}
            onChange={(e) => setSettings({ ...settings, mode: e.target.value as 'host' | 'agent' })}
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
          >
            <option value="host">Host</option>
            <option value="agent">Agent</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Port
          </label>
          <input
            type="number"
            value={settings.port}
            onChange={(e) => setSettings({ ...settings, port: parseInt(e.target.value) })}
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Pre-Shared Key (PSK)
          </label>
          <input
            type="password"
            value={settings.psk}
            onChange={(e) => setSettings({ ...settings, psk: e.target.value })}
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
            placeholder="Enter secure PSK..."
          />
        </div>

        <button className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded">
          Save Configuration
        </button>
      </div>
    </div>
  )
}
