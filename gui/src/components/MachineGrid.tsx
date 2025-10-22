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
          stroke="url(#connectionGradient)"
          strokeWidth="3"
          markerEnd="url(#arrowhead)"
          className="drop-shadow-lg"
          strokeDasharray="5,5"
          style={{ animation: 'dash 20s linear infinite' }}
        />
      )
    })
  }

  return (
    <div className="backdrop-blur-xl bg-white/5 p-8 rounded-3xl border border-white/10 shadow-2xl">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-3xl font-bold bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent">
            Network Topology
          </h2>
          <p className="text-slate-400 text-sm mt-1">Drag machines to position them spatially</p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={addMachine}
            className="px-6 py-3 bg-gradient-to-r from-purple-600 to-cyan-600 hover:from-purple-500 hover:to-cyan-500 rounded-xl font-semibold shadow-lg shadow-purple-500/30 transition-all duration-300 hover:scale-105 flex items-center gap-2"
          >
            <span className="text-xl">➕</span>
            Add Machine
          </button>
        </div>
      </div>

      <div
        ref={canvasRef}
        className="relative backdrop-blur-sm bg-slate-950/50 border-2 border-purple-500/30 rounded-2xl p-6 overflow-hidden shadow-inner"
        style={{ height: '600px' }}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {/* Grid pattern background */}
        <div className="absolute inset-0 opacity-10" style={{
          backgroundImage: 'linear-gradient(rgba(139, 92, 246, 0.3) 1px, transparent 1px), linear-gradient(90deg, rgba(139, 92, 246, 0.3) 1px, transparent 1px)',
          backgroundSize: '50px 50px'
        }} />
        <svg
          className="absolute inset-0 w-full h-full pointer-events-none"
          style={{ zIndex: 0 }}
        >
          <defs>
            <marker
              id="arrowhead"
              markerWidth="12"
              markerHeight="10"
              refX="11"
              refY="5"
              orient="auto"
            >
              <polygon points="0 0, 12 5, 0 10" fill="url(#connectionGradient)" />
            </marker>
            <linearGradient id="connectionGradient" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="#a78bfa" />
              <stop offset="100%" stopColor="#22d3ee" />
            </linearGradient>
          </defs>
          {renderConnections()}
        </svg>

        {machines.map((machine) => (
          <div
            key={machine.id}
            className={`absolute cursor-move select-none group ${
              selectedMachine === machine.id ? 'ring-2 ring-cyan-400 ring-offset-2 ring-offset-slate-950' : ''
            }`}
            style={{
              left: `${machine.x}px`,
              top: `${machine.y}px`,
              width: '140px',
              zIndex: dragging === machine.id ? 10 : 1,
            }}
            onMouseDown={(e) => handleMouseDown(e, machine.id)}
            onClick={() => setSelectedMachine(machine.id)}
          >
            {/* Glow effect */}
            <div className={`absolute inset-0 rounded-2xl blur-xl opacity-50 ${
              machine.role === 'host'
                ? 'bg-gradient-to-r from-purple-500 to-indigo-500'
                : 'bg-gradient-to-r from-blue-500 to-cyan-500'
            }`} />

            {/* Card */}
            <div className={`relative backdrop-blur-xl rounded-2xl p-4 shadow-2xl border transition-all duration-300 ${
              machine.role === 'host'
                ? 'bg-gradient-to-br from-purple-600/90 to-indigo-600/90 border-purple-400/50 hover:from-purple-500/90 hover:to-indigo-500/90'
                : 'bg-gradient-to-br from-blue-600/90 to-cyan-600/90 border-cyan-400/50 hover:from-blue-500/90 hover:to-cyan-500/90'
            } group-hover:scale-110 group-hover:shadow-2xl`}>
              <div className="flex items-center justify-between mb-2">
                <div className="text-2xl">{machine.role === 'host' ? '👑' : '💻'}</div>
                {dragging === machine.id && <div className="text-xs">✋</div>}
              </div>
              <div className="font-bold text-white text-sm truncate">{machine.name}</div>
              <div className="text-xs text-white/70 uppercase tracking-wider mt-1">{machine.role}</div>

            {selectedMachine === machine.id && (
              <div className="mt-3 flex gap-2">
                <button
                  onClick={() => startConnection(machine.id)}
                  className="flex-1 text-xs bg-white/20 hover:bg-white/30 backdrop-blur-sm px-3 py-1.5 rounded-lg font-medium transition-all duration-200 hover:scale-105"
                >
                  🔗 Link
                </button>
                {machine.role !== 'host' && (
                  <button
                    onClick={() => removeMachine(machine.id)}
                    className="text-xs bg-red-500/30 hover:bg-red-500/50 backdrop-blur-sm px-3 py-1.5 rounded-lg font-medium transition-all duration-200 hover:scale-105"
                  >
                    🗑️
                  </button>
                )}
              </div>
            )}

            {connecting && connecting !== machine.id && (
              <div className="mt-3 grid grid-cols-2 gap-1.5">
                <button
                  onClick={() => finishConnection(machine.id, 'left')}
                  className="text-xs bg-emerald-500/30 hover:bg-emerald-500/50 backdrop-blur-sm px-2 py-2 rounded-lg font-bold transition-all duration-200 hover:scale-110"
                >
                  ← Left
                </button>
                <button
                  onClick={() => finishConnection(machine.id, 'right')}
                  className="text-xs bg-emerald-500/30 hover:bg-emerald-500/50 backdrop-blur-sm px-2 py-2 rounded-lg font-bold transition-all duration-200 hover:scale-110"
                >
                  Right →
                </button>
                <button
                  onClick={() => finishConnection(machine.id, 'top')}
                  className="text-xs bg-emerald-500/30 hover:bg-emerald-500/50 backdrop-blur-sm px-2 py-2 rounded-lg font-bold transition-all duration-200 hover:scale-110"
                >
                  ↑ Top
                </button>
                <button
                  onClick={() => finishConnection(machine.id, 'bottom')}
                  className="text-xs bg-emerald-500/30 hover:bg-emerald-500/50 backdrop-blur-sm px-2 py-2 rounded-lg font-bold transition-all duration-200 hover:scale-110"
                >
                  Bottom ↓
                </button>
              </div>
            )}
            </div>
          </div>
        ))}

        {machines.length === 0 && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center backdrop-blur-xl bg-white/5 p-8 rounded-2xl border border-white/10">
              <div className="text-6xl mb-4">🌐</div>
              <p className="text-slate-300 text-lg font-semibold">
                No machines configured yet
              </p>
              <p className="text-slate-500 text-sm mt-2">
                Click "Add Machine" to get started
              </p>
            </div>
          </div>
        )}
      </div>

      <div className="mt-6 backdrop-blur-xl bg-white/5 border border-white/10 rounded-2xl p-6">
        <h3 className="text-xl font-bold bg-gradient-to-r from-purple-400 to-cyan-400 bg-clip-text text-transparent mb-4">
          Topology Summary
        </h3>
        <div className="grid grid-cols-2 gap-4">
          <div className="backdrop-blur-sm bg-purple-500/10 border border-purple-500/30 rounded-xl p-4">
            <div className="text-purple-400 text-sm font-medium mb-1">Machines</div>
            <div className="text-3xl font-bold text-white">{machines.length}</div>
          </div>
          <div className="backdrop-blur-sm bg-cyan-500/10 border border-cyan-500/30 rounded-xl p-4">
            <div className="text-cyan-400 text-sm font-medium mb-1">Connections</div>
            <div className="text-3xl font-bold text-white">{connections.length}</div>
          </div>
        </div>
        {connections.length > 0 && (
          <div className="mt-4 space-y-2">
            <div className="text-sm font-medium text-slate-400 mb-2">Active Links:</div>
            {connections.map((conn, idx) => (
              <div
                key={idx}
                className="flex items-center gap-2 text-sm backdrop-blur-sm bg-white/5 border border-white/10 rounded-lg p-2"
              >
                <span className="font-mono text-purple-400">{getMachineName(conn.from)}</span>
                <span className="text-slate-500">→</span>
                <span className="font-mono text-cyan-400">{getMachineName(conn.to)}</span>
                <span className="ml-auto text-xs bg-white/10 px-2 py-1 rounded">{conn.edge}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
