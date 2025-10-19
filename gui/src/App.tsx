import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

function App() {
  const [version, setVersion] = useState<string>('')
  const [greeting, setGreeting] = useState<string>('')

  useEffect(() => {
    // Get version from Tauri backend
    invoke<string>('get_version')
      .then((ver) => setVersion(ver))
      .catch((err) => console.error('Failed to get version:', err))

    // Get greeting
    invoke<string>('greet', { name: 'User' })
      .then((msg) => setGreeting(msg))
      .catch((err) => console.error('Failed to get greeting:', err))
  }, [])

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <div className="container mx-auto p-4">
        <header className="mb-8">
          <h1 className="text-4xl font-bold mb-2">🕉️ MultiShiva</h1>
          <p className="text-gray-400">Many arms. One mind. v{version}</p>
          {greeting && <p className="text-green-400 mt-2">{greeting}</p>}
        </header>

        <main className="grid grid-cols-1 gap-6">
          <div className="bg-gray-800 p-6 rounded-lg">
            <h2 className="text-2xl font-semibold mb-4">Welcome</h2>
            <p className="text-gray-300">
              Control multiple computers with one keyboard and mouse.
            </p>
          </div>

          <div className="bg-gray-800 p-6 rounded-lg">
            <h3 className="text-xl font-semibold mb-4">Quick Start</h3>
            <p className="text-gray-300 mb-2">GUI components coming soon...</p>
            <ul className="list-disc list-inside text-gray-400">
              <li>Machine Grid - Visual topology editor</li>
              <li>Settings Panel - Configuration management</li>
              <li>Status Bar - Real-time monitoring</li>
            </ul>
          </div>

          <div className="bg-gray-800 p-6 rounded-lg">
            <h3 className="text-xl font-semibold mb-4">Status</h3>
            <p className="text-gray-300">
              ✅ Tauri + React + TypeScript setup complete
            </p>
            <p className="text-gray-300">
              ✅ TailwindCSS configured
            </p>
            <p className="text-gray-300">
              ✅ Frontend ↔ Backend communication working
            </p>
          </div>
        </main>
      </div>
    </div>
  )
}

export default App
