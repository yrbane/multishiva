import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
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
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-indigo-950 text-white flex flex-col relative overflow-hidden">
      {/* Animated background */}
      <div className="absolute inset-0 opacity-30">
        <div className="absolute top-0 left-1/4 w-96 h-96 bg-purple-500 rounded-full mix-blend-multiply filter blur-3xl animate-pulse" />
        <div className="absolute top-1/3 right-1/4 w-96 h-96 bg-cyan-500 rounded-full mix-blend-multiply filter blur-3xl animate-pulse animation-delay-2000" />
        <div className="absolute bottom-0 left-1/2 w-96 h-96 bg-indigo-500 rounded-full mix-blend-multiply filter blur-3xl animate-pulse animation-delay-4000" />
      </div>

      {/* Header with glassmorphism */}
      <header className="relative backdrop-blur-xl bg-white/5 border-b border-white/10 shadow-2xl">
        <div className="px-8 py-6">
          <div className="flex items-center justify-between max-w-7xl mx-auto">
            <div className="flex items-center gap-4">
              <div className="relative">
                <div className="absolute inset-0 bg-gradient-to-r from-purple-600 to-cyan-600 rounded-2xl blur-lg opacity-75 animate-pulse" />
                <div className="relative text-5xl bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent font-bold p-2">
                  🕉️
                </div>
              </div>
              <div>
                <h1 className="text-4xl font-black bg-gradient-to-r from-white via-purple-200 to-cyan-200 bg-clip-text text-transparent">
                  MultiShiva
                </h1>
                <p className="text-sm text-slate-400 mt-1 font-medium">
                  Many arms. One mind. <span className="text-purple-400 font-mono">v{version}</span>
                </p>
              </div>
            </div>

            {/* Tab Navigation with modern design */}
            <div className="flex gap-3 p-1.5 rounded-2xl bg-white/5 backdrop-blur-sm border border-white/10">
              <button
                onClick={() => setActiveTab('topology')}
                className={`px-8 py-3 rounded-xl font-semibold transition-all duration-300 flex items-center gap-2 ${
                  activeTab === 'topology'
                    ? 'bg-gradient-to-r from-purple-600 to-cyan-600 text-white shadow-lg shadow-purple-500/50 scale-105'
                    : 'text-slate-300 hover:text-white hover:bg-white/10'
                }`}
              >
                <span className="text-xl">🗺️</span>
                <span>Topology</span>
              </button>
              <button
                onClick={() => setActiveTab('settings')}
                className={`px-8 py-3 rounded-xl font-semibold transition-all duration-300 flex items-center gap-2 ${
                  activeTab === 'settings'
                    ? 'bg-gradient-to-r from-purple-600 to-cyan-600 text-white shadow-lg shadow-purple-500/50 scale-105'
                    : 'text-slate-300 hover:text-white hover:bg-white/10'
                }`}
              >
                <span className="text-xl">⚙️</span>
                <span>Settings</span>
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content with fade transition */}
      <main className="relative flex-1 overflow-y-auto p-8">
        <div className="max-w-7xl mx-auto">
          <div className={`transition-all duration-500 ${activeTab === 'topology' ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4 absolute'}`}>
            {activeTab === 'topology' && <MachineGrid />}
          </div>
          <div className={`transition-all duration-500 ${activeTab === 'settings' ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4 absolute'}`}>
            {activeTab === 'settings' && <SettingsPanel />}
          </div>
        </div>
      </main>

      {/* Status Bar (Fixed at bottom) */}
      <StatusBar />
    </div>
  )
}

export default App
