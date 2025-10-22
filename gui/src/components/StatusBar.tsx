// StatusBar Component - Real-time monitoring
import { useState, useEffect } from 'react'

interface ConnectionStatus {
  connected: boolean
  mode: 'host' | 'agent' | 'disconnected'
  machinesConnected: number
  activeMachine: string | null
  eventsPerSecond: number
  bytesReceived: number
  bytesSent: number
  lastEvent: string | null
  uptime: number
  features: {
    mdnsActive: boolean
    clipboardSync: boolean
  }
}

export default function StatusBar() {
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    mode: 'disconnected',
    machinesConnected: 0,
    activeMachine: null,
    eventsPerSecond: 0,
    bytesReceived: 0,
    bytesSent: 0,
    lastEvent: null,
    uptime: 0,
    features: {
      mdnsActive: false,
      clipboardSync: false,
    },
  })

  const [showDetails, setShowDetails] = useState(false)

  useEffect(() => {
    // TODO: Connect to backend for real-time status updates via Tauri commands
    // For now, simulate with demo data
    const interval = setInterval(() => {
      setStatus((prev) => ({
        connected: true,
        mode: 'host',
        machinesConnected: Math.floor(Math.random() * 3) + 1,
        activeMachine: Math.random() > 0.5 ? 'agent1' : 'host',
        eventsPerSecond: Math.floor(Math.random() * 120),
        bytesReceived: prev.bytesReceived + Math.floor(Math.random() * 5000),
        bytesSent: prev.bytesSent + Math.floor(Math.random() * 3000),
        lastEvent: new Date().toLocaleTimeString(),
        uptime: prev.uptime + 1,
        features: {
          mdnsActive: true,
          clipboardSync: true,
        },
      }))
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
  }

  const formatUptime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60
    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`
    }
    return `${secs}s`
  }

  return (
    <div className="relative backdrop-blur-xl bg-white/5 border-t border-white/10 shadow-2xl">
      {/* Main Status Bar */}
      <div className="px-8 py-4 flex items-center justify-between">
        <div className="flex items-center gap-4">
          {/* Connection Status - Modern pill */}
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="flex items-center gap-2 px-4 py-2 backdrop-blur-sm bg-white/5 hover:bg-white/10 border border-white/10 rounded-full transition-all duration-200 hover:scale-105"
          >
            <div className="relative">
              <div
                className={`w-2.5 h-2.5 rounded-full ${
                  status.connected ? 'bg-emerald-400 shadow-lg shadow-emerald-500/50' : 'bg-red-400 shadow-lg shadow-red-500/50'
                }`}
              />
              {status.connected && (
                <div className="absolute inset-0 w-2.5 h-2.5 rounded-full bg-emerald-400 animate-ping opacity-60" />
              )}
            </div>
            <span className="text-sm font-semibold text-white">
              {status.connected ? status.mode.toUpperCase() : 'OFFLINE'}
            </span>
            <span className="text-xs text-slate-400">
              {showDetails ? '▼' : '▸'}
            </span>
          </button>

          {/* Stats Cards - Compact */}
          <div className="flex items-center gap-3">
            {status.activeMachine && (
              <div className="flex items-center gap-2 px-3 py-1.5 backdrop-blur-sm bg-purple-500/10 border border-purple-500/30 rounded-lg">
                <span className="text-xs text-purple-400 font-medium">Active:</span>
                <span className="text-sm text-white font-semibold font-mono">{status.activeMachine}</span>
              </div>
            )}

            <div className="flex items-center gap-2 px-3 py-1.5 backdrop-blur-sm bg-cyan-500/10 border border-cyan-500/30 rounded-lg">
              <span className="text-lg">💻</span>
              <span className="text-sm font-bold text-white">{status.machinesConnected}</span>
            </div>

            <div className="flex items-center gap-2 px-3 py-1.5 backdrop-blur-sm bg-indigo-500/10 border border-indigo-500/30 rounded-lg">
              <span className="text-lg">⚡</span>
              <span className="text-sm font-bold font-mono text-white">{status.eventsPerSecond}</span>
              <span className="text-xs text-slate-400">/s</span>
            </div>
          </div>

          {/* Feature Badges */}
          <div className="flex items-center gap-2">
            {status.features.mdnsActive && (
              <div
                className="text-xs font-medium text-purple-300 px-2 py-1 bg-purple-500/20 rounded-md border border-purple-500/30"
                title="mDNS Discovery Active"
              >
                🌐 mDNS
              </div>
            )}
            {status.features.clipboardSync && (
              <div
                className="text-xs font-medium text-cyan-300 px-2 py-1 bg-cyan-500/20 rounded-md border border-cyan-500/30"
                title="Clipboard Sync Active"
              >
                📋 Clipboard
              </div>
            )}
          </div>
        </div>

        {/* Right Side - Compact */}
        <div className="flex items-center gap-4">
          <div className="text-xs font-mono text-slate-400">
            ⏱ <span className="text-slate-300">{formatUptime(status.uptime)}</span>
          </div>
          <div className="text-xs font-semibold bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent">
            v1.2.0
          </div>
        </div>
      </div>

      {/* Detailed Stats Panel */}
      {showDetails && (
        <div className="px-6 py-4 bg-gray-900 border-t border-gray-700">
          <div className="grid grid-cols-4 gap-6">
            {/* Network Stats */}
            <div>
              <h4 className="text-xs font-semibold text-gray-400 mb-2">Network</h4>
              <div className="space-y-1">
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Received:</span>
                  <span className="text-green-400 font-mono">
                    {formatBytes(status.bytesReceived)}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Sent:</span>
                  <span className="text-blue-400 font-mono">
                    {formatBytes(status.bytesSent)}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Total:</span>
                  <span className="text-white font-mono">
                    {formatBytes(status.bytesReceived + status.bytesSent)}
                  </span>
                </div>
              </div>
            </div>

            {/* Performance */}
            <div>
              <h4 className="text-xs font-semibold text-gray-400 mb-2">Performance</h4>
              <div className="space-y-1">
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Events/s:</span>
                  <span className="text-white font-mono">{status.eventsPerSecond}</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Avg Latency:</span>
                  <span className="text-green-400 font-mono">
                    {Math.floor(Math.random() * 10) + 1}ms
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">CPU:</span>
                  <span className="text-yellow-400 font-mono">
                    {Math.floor(Math.random() * 15) + 5}%
                  </span>
                </div>
              </div>
            </div>

            {/* Features */}
            <div>
              <h4 className="text-xs font-semibold text-gray-400 mb-2">Features</h4>
              <div className="space-y-1">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-gray-500">mDNS Discovery:</span>
                  <span
                    className={`font-semibold ${
                      status.features.mdnsActive ? 'text-green-400' : 'text-gray-600'
                    }`}
                  >
                    {status.features.mdnsActive ? 'Active' : 'Inactive'}
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-gray-500">Clipboard Sync:</span>
                  <span
                    className={`font-semibold ${
                      status.features.clipboardSync ? 'text-green-400' : 'text-gray-600'
                    }`}
                  >
                    {status.features.clipboardSync ? 'Active' : 'Inactive'}
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-gray-500">TLS Encryption:</span>
                  <span className="text-green-400 font-semibold">Enabled</span>
                </div>
              </div>
            </div>

            {/* Activity */}
            <div>
              <h4 className="text-xs font-semibold text-gray-400 mb-2">Activity</h4>
              <div className="space-y-1">
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Last Event:</span>
                  <span className="text-gray-300 font-mono">
                    {status.lastEvent || 'None'}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Machines:</span>
                  <span className="text-white font-mono">{status.machinesConnected}</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Mode:</span>
                  <span className="text-blue-400 font-mono uppercase">{status.mode}</span>
                </div>
              </div>
            </div>
          </div>

          {/* Quick Actions */}
          <div className="mt-4 flex gap-2">
            <button className="text-xs px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded transition">
              Reconnect
            </button>
            <button className="text-xs px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded transition">
              Clear Stats
            </button>
            <button className="text-xs px-3 py-1 bg-red-900/30 hover:bg-red-900/50 text-red-400 border border-red-700/50 rounded transition">
              Emergency Stop
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
