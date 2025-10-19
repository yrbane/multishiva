// MachineGrid Component - Canvas drag & drop for machine topology
import { useState } from 'react'

interface Machine {
  id: string
  name: string
  x: number
  y: number
}

export default function MachineGrid() {
  const [machines, setMachines] = useState<Machine[]>([])

  return (
    <div className="bg-gray-800 p-6 rounded-lg">
      <h2 className="text-2xl font-semibold mb-4">Machine Grid</h2>
      <div className="bg-gray-900 border-2 border-gray-700 rounded-lg p-4 min-h-[400px]">
        <p className="text-gray-500 text-center mt-20">
          Drag & drop topology editor coming soon...
        </p>
        {machines.length === 0 && (
          <p className="text-gray-600 text-center mt-2">No machines configured yet</p>
        )}
      </div>
    </div>
  )
}
