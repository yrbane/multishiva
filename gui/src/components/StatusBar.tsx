// StatusBar Component - Real-time monitoring
import { useState, useEffect } from 'react'

interface ConnectionStatus {
  connected: boolean
  machinesConnected: number
  eventsPerSecond: number
}

export default function StatusBar() {
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    machinesConnected: 0,
    eventsPerSecond: 0,
  })

  useEffect(() => {
    // TODO: Connect to backend for real-time status updates
    // For now, simulate with dummy data
    const interval = setInterval(() => {
      setStatus({
        connected: true,
        machinesConnected: Math.floor(Math.random() * 4),
        eventsPerSecond: Math.floor(Math.random() * 50),
      })
    }, 2000)

    return () => clearInterval(interval)
  }, [])

  return (
    <div className="bg-gray-800 border-t border-gray-700 px-6 py-3 flex items-center justify-between">
      <div className="flex items-center space-x-6">
        <div className="flex items-center space-x-2">
          <div
            className={`w-3 h-3 rounded-full ${
              status.connected ? 'bg-green-500' : 'bg-red-500'
            }`}
          />
          <span className="text-sm text-gray-300">
            {status.connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>

        <div className="text-sm text-gray-400">
          Machines: <span className="text-white font-semibold">{status.machinesConnected}</span>
        </div>

        <div className="text-sm text-gray-400">
          Events/s: <span className="text-white font-semibold">{status.eventsPerSecond}</span>
        </div>
      </div>

      <div className="text-xs text-gray-500">
        MultiShiva v0.3.0
      </div>
    </div>
  )
}
