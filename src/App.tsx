import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './styles/globals.css'

function App() {
  const [message, setMessage] = useState('')

  async function testCommand() {
    const result = await invoke<string>('cmd_hello_world')
    setMessage(result)
  }

  return (
    <div className="container mx-auto p-8">
      <h1 className="text-3xl font-bold mb-4">Agents for PPT</h1>
      <p className="text-gray-600 mb-6">AI-driven multi-agent system for PPT generation</p>
      <button
        onClick={testCommand}
        className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition"
      >
        Test Command
      </button>
      {message && (
        <div className="mt-4 p-4 bg-green-100 border border-green-400 rounded">
          <p className="text-green-800">{message}</p>
        </div>
      )}
    </div>
  )
}

export default App
