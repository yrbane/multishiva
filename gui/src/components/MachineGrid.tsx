// MachineGrid Component - Canvas drag & drop for machine topology
import { useState, useRef } from 'react'

interface Machine {
  id: string
  name: string
  x: number
  y: number
  role: 'host' | 'agent'
}

interface Connection {
  from: string
  to: string
  edge: 'left' | 'right' | 'top' | 'bottom'
}

export default function MachineGrid() {
  const [machines, setMachines] = useState<Machine[]>([
    { id: '1', name: 'host', x: 400, y: 300, role: 'host' },
  ])
  const [connections, setConnections] = useState<Connection[]>([])
  const [selectedMachine, setSelectedMachine] = useState<string | null>(null)
  const [dragging, setDragging] = useState<string | null>(null)
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 })
  const [connecting, setConnecting] = useState<string | null>(null)
  const canvasRef = useRef<HTMLDivElement>(null)

  const handleMouseDown = (e: React.MouseEvent, machineId: string) => {
    const machine = machines.find((m) => m.id === machineId)
    if (!machine) return

    setDragging(machineId)
    setDragOffset({
      x: e.clientX - machine.x,
      y: e.clientY - machine.y,
    })
  }

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!dragging) return

    const canvas = canvasRef.current
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left - dragOffset.x
    const y = e.clientY - rect.top - dragOffset.y

    setMachines((prev) =>
      prev.map((m) =>
        m.id === dragging ? { ...m, x: Math.max(0, Math.min(x, rect.width - 100)), y: Math.max(0, Math.min(y, rect.height - 60)) } : m
      )
    )
  }

  const handleMouseUp = () => {
    setDragging(null)
  }

  const addMachine = () => {
    const newId = String(machines.length + 1)
    setMachines([
      ...machines,
      {
        id: newId,
        name: `agent${machines.length}`,
        x: 200 + machines.length * 50,
        y: 200,
        role: 'agent',
      },
    ])
  }

  const removeMachine = (id: string) => {
    setMachines(machines.filter((m) => m.id !== id))
    setConnections(connections.filter((c) => c.from !== id && c.to !== id))
    if (selectedMachine === id) setSelectedMachine(null)
  }

  const startConnection = (fromId: string) => {
    setConnecting(fromId)
  }

  const finishConnection = (toId: string, edge: 'left' | 'right' | 'top' | 'bottom') => {
    if (connecting && connecting !== toId) {
      setConnections([
        ...connections.filter((c) => !(c.from === connecting && c.to === toId)),
        { from: connecting, to: toId, edge },
      ])
    }
    setConnecting(null)
  }

  const getMachineName = (id: string) => {
    return machines.find((m) => m.id === id)?.name || id
  }

  const renderConnections = () => {
    return connections.map((conn, idx) => {
      const fromMachine = machines.find((m) => m.id === conn.from)
      const toMachine = machines.find((m) => m.id === conn.to)
      if (!fromMachine || !toMachine) return null

      const fromX = fromMachine.x + 50
      const fromY = fromMachine.y + 30
      const toX = toMachine.x + 50
      const toY = toMachine.y + 30

      return (
        <line
          key={idx}
          x1={fromX}
          y1={fromY}
          x2={toX}
          y2={toY}
          stroke="#60a5fa"
          strokeWidth="2"
          markerEnd="url(#arrowhead)"
        />
      )
    })
  }

  return (
    <div className="bg-gray-800 p-6 rounded-lg">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-semibold">Machine Grid</h2>
        <div className="flex gap-2">
          <button
            onClick={addMachine}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition"
          >
            + Add Machine
          </button>
        </div>
      </div>

      <div
        ref={canvasRef}
        className="bg-gray-900 border-2 border-gray-700 rounded-lg p-4 relative"
        style={{ height: '600px' }}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <svg
          className="absolute inset-0 w-full h-full pointer-events-none"
          style={{ zIndex: 0 }}
        >
          <defs>
            <marker
              id="arrowhead"
              markerWidth="10"
              markerHeight="7"
              refX="9"
              refY="3.5"
              orient="auto"
            >
              <polygon points="0 0, 10 3.5, 0 7" fill="#60a5fa" />
            </marker>
          </defs>
          {renderConnections()}
        </svg>

        {machines.map((machine) => (
          <div
            key={machine.id}
            className={`absolute cursor-move select-none ${
              machine.role === 'host'
                ? 'bg-gradient-to-r from-purple-600 to-indigo-600'
                : 'bg-gradient-to-r from-blue-600 to-cyan-600'
            } rounded-lg p-4 shadow-lg transition-transform hover:scale-105 ${
              selectedMachine === machine.id ? 'ring-2 ring-yellow-400' : ''
            }`}
            style={{
              left: `${machine.x}px`,
              top: `${machine.y}px`,
              width: '100px',
              zIndex: dragging === machine.id ? 10 : 1,
            }}
            onMouseDown={(e) => handleMouseDown(e, machine.id)}
            onClick={() => setSelectedMachine(machine.id)}
          >
            <div className="text-sm font-bold text-white mb-1">{machine.name}</div>
            <div className="text-xs text-gray-200">{machine.role}</div>

            {selectedMachine === machine.id && (
              <div className="mt-2 flex gap-1">
                <button
                  onClick={() => startConnection(machine.id)}
                  className="text-xs bg-white/20 hover:bg-white/30 px-2 py-1 rounded"
                >
                  Connect
                </button>
                {machine.role !== 'host' && (
                  <button
                    onClick={() => removeMachine(machine.id)}
                    className="text-xs bg-red-500/20 hover:bg-red-500/30 px-2 py-1 rounded"
                  >
                    Remove
                  </button>
                )}
              </div>
            )}

            {connecting && connecting !== machine.id && (
              <div className="mt-2 grid grid-cols-2 gap-1">
                <button
                  onClick={() => finishConnection(machine.id, 'left')}
                  className="text-xs bg-green-500/20 hover:bg-green-500/30 px-1 py-1 rounded"
                >
                  ← L
                </button>
                <button
                  onClick={() => finishConnection(machine.id, 'right')}
                  className="text-xs bg-green-500/20 hover:bg-green-500/30 px-1 py-1 rounded"
                >
                  R →
                </button>
                <button
                  onClick={() => finishConnection(machine.id, 'top')}
                  className="text-xs bg-green-500/20 hover:bg-green-500/30 px-1 py-1 rounded"
                >
                  ↑ T
                </button>
                <button
                  onClick={() => finishConnection(machine.id, 'bottom')}
                  className="text-xs bg-green-500/20 hover:bg-green-500/30 px-1 py-1 rounded"
                >
                  B ↓
                </button>
              </div>
            )}
          </div>
        ))}

        {machines.length === 0 && (
          <div className="absolute inset-0 flex items-center justify-center">
            <p className="text-gray-500 text-center">
              No machines configured. Click "Add Machine" to get started.
            </p>
          </div>
        )}
      </div>

      <div className="mt-4 bg-gray-900 border border-gray-700 rounded-lg p-4">
        <h3 className="text-lg font-semibold mb-2">Topology</h3>
        <div className="text-sm space-y-1">
          <div className="text-gray-400">Machines: {machines.length}</div>
          <div className="text-gray-400">Connections: {connections.length}</div>
          {connections.length > 0 && (
            <div className="mt-2">
              {connections.map((conn, idx) => (
                <div key={idx} className="text-xs text-gray-500">
                  {getMachineName(conn.from)} → {getMachineName(conn.to)} ({conn.edge})
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
