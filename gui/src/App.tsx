import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'
import MachineGrid from './components/MachineGrid'
import SettingsPanel from './components/SettingsPanel'
import StatusBar from './components/StatusBar'

function App() {
  const [version, setVersion] = useState<string>('')
  const [activeTab, setActiveTab] = useState<'topology' | 'settings'>('topology')

  useEffect(() => {
    // Get version from Tauri backend
    invoke<string>('get_version')
      .then((ver) => setVersion(ver))
      .catch((err) => console.error('Failed to get version:', err))
  }, [])

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold flex items-center gap-2">
              <span>🕉️</span>
              <span>MultiShiva</span>
            </h1>
            <p className="text-sm text-gray-400 mt-1">
              Many arms. One mind. <span className="text-gray-500">v{version}</span>
            </p>
          </div>

          {/* Tab Navigation */}
          <div className="flex gap-2">
            <button
              onClick={() => setActiveTab('topology')}
              className={`px-6 py-2 rounded-lg font-medium transition ${
                activeTab === 'topology'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              📐 Topology
            </button>
            <button
              onClick={() => setActiveTab('settings')}
              className={`px-6 py-2 rounded-lg font-medium transition ${
                activeTab === 'settings'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              ⚙️ Settings
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto p-6">
        <div className="max-w-7xl mx-auto">
          {activeTab === 'topology' && <MachineGrid />}
          {activeTab === 'settings' && <SettingsPanel />}
        </div>
      </main>

      {/* Status Bar (Fixed at bottom) */}
      <StatusBar />
    </div>
  )
}

export default App
