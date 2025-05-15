import { useState } from 'react'
import { AdminLogin } from './components/AdminLogin'
import { AdminProducts } from './components/AdminProducts'
import './App.css'

function App() {
  const [token, setToken] = useState<string | null>(null)

  if (!token) {
    return <AdminLogin onAuth={setToken} />
  }

  return (
    <div style={{ maxWidth: 600, margin: '2em auto', padding: 24 }}>
      <h1>Admin Dashboard</h1>
      <p>Welcome! (JWT: <code>{token.slice(0, 16)}...</code>)</p>
      <AdminProducts token={token} />
    </div>
  )
}

export default App
