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
    <div className="bg-gray-800 border-t border-gray-700">
      {/* Main Status Bar */}
      <div className="px-6 py-3 flex items-center justify-between">
        <div className="flex items-center space-x-6">
          {/* Connection Status */}
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="flex items-center space-x-2 hover:bg-gray-700 px-2 py-1 rounded transition"
          >
            <div className="relative">
              <div
                className={`w-3 h-3 rounded-full ${
                  status.connected ? 'bg-green-500' : 'bg-red-500'
                }`}
              />
              {status.connected && (
                <div className="absolute inset-0 w-3 h-3 rounded-full bg-green-500 animate-ping opacity-75" />
              )}
            </div>
            <span className="text-sm text-gray-300 font-medium">
              {status.connected ? status.mode.toUpperCase() : 'Disconnected'}
            </span>
            <span className="text-xs text-gray-500">
              {showDetails ? '▼' : '▶'}
            </span>
          </button>

          {/* Active Machine */}
          {status.activeMachine && (
            <div className="flex items-center space-x-2">
              <span className="text-xs text-gray-500">Active:</span>
              <span className="text-sm text-blue-400 font-semibold">
                {status.activeMachine}
              </span>
            </div>
          )}

          {/* Connected Machines */}
          <div className="flex items-center space-x-2">
            <svg
              className="w-4 h-4 text-gray-400"
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
            </svg>
            <span className="text-sm text-gray-400">
              <span className="text-white font-semibold">{status.machinesConnected}</span>{' '}
              {status.machinesConnected === 1 ? 'machine' : 'machines'}
            </span>
          </div>

          {/* Event Rate */}
          <div className="flex items-center space-x-2">
            <span className="text-xs text-gray-500">Events/s:</span>
            <span className="text-sm text-white font-mono font-semibold">
              {status.eventsPerSecond}
            </span>
          </div>

          {/* Feature Indicators */}
          <div className="flex items-center space-x-3">
            {status.features.mdnsActive && (
              <div
                className="text-xs text-purple-400 px-2 py-1 bg-purple-900/30 rounded border border-purple-700/50"
                title="mDNS Discovery Active"
              >
                mDNS
              </div>
            )}
            {status.features.clipboardSync && (
              <div
                className="text-xs text-cyan-400 px-2 py-1 bg-cyan-900/30 rounded border border-cyan-700/50"
                title="Clipboard Sync Active"
              >
                Clipboard
              </div>
            )}
          </div>
        </div>

        {/* Right Side */}
        <div className="flex items-center space-x-4">
          <div className="text-xs text-gray-500">
            Uptime: <span className="text-gray-400 font-mono">{formatUptime(status.uptime)}</span>
          </div>
          <div className="text-xs text-gray-500">MultiShiva v1.0.0</div>
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
