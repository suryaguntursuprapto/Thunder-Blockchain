import { useState } from 'react'

import { Rocket, Download, KeyRound, Copy, Check } from 'lucide-react'
import './index.css'

interface OnboardingProps {
  onComplete: (privateKey: string, mnemonic: string, address: string) => void;
  onCancel?: () => void;
}

export default function Onboarding({ onComplete, onCancel }: OnboardingProps) {
  const [view, setView] = useState<'home' | 'create' | 'import'>('home')
  const [mnemonicInput, setMnemonicInput] = useState('')
  const [generatedMnemonic, setGeneratedMnemonic] = useState('')
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(generatedMnemonic)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleCreate = async () => {
    try {
      const res = await fetch('http://127.0.0.1:5050/api/wallet/generate-seed', { method: 'POST' })
      const data = await res.json()
      if (data.mnemonic) {
        setGeneratedMnemonic(data.mnemonic)
        setView('create')
      } else {
        setError('Failed to generate seed')
      }
    } catch (err) {
      console.error(err)
      setError('Failed to generate wallet')
    }
  }

  const handleConfirmCreate = async () => {
    setIsLoading(true)
    setError('')
    try {
      const res = await fetch('http://127.0.0.1:5050/api/wallet/derive-address', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ seed: generatedMnemonic, index: 0 })
      })
      const data = await res.json()
      if (data.address && data.private_key) {
        onComplete(data.private_key, generatedMnemonic, data.address)
      } else {
        setError(data.error || 'Failed to derive address')
      }
    } catch (err) {
      console.error(err)
      setError('Network error: Unable to reach backend')
    } finally {
      setIsLoading(false)
    }
  }

  const handleImport = async () => {
    try {
      const res = await fetch('http://127.0.0.1:5050/api/wallet/derive-address', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ seed: mnemonicInput.trim(), index: 0 })
      })
      const data = await res.json()
      if (data.address && data.private_key) {
        onComplete(data.private_key, mnemonicInput.trim(), data.address)
      } else {
        setError(data.error || 'Invalid Seed Phrase')
      }
    } catch (err) {
      console.error(err)
      setError('Network error: Unable to reach backend')
    }
  }

  return (
    <div className="app-container" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', minHeight: '100vh', padding: '32px 24px', overflowY: 'auto', position: 'relative' }}>
      {onCancel && (
        <button 
          onClick={onCancel}
          style={{ position: 'absolute', top: 16, right: 16, background: 'transparent', border: 'none', color: 'var(--text-secondary)', cursor: 'pointer' }}
        >
          ✕
        </button>
      )}
      <img src="/logo.png" alt="Thunder Logo" style={{ width: 80, height: 80, marginBottom: 24, filter: 'drop-shadow(0 0 15px rgba(139, 92, 246, 0.6))' }} />
      <h2 style={{ fontSize: '28px', fontWeight: '800', marginBottom: '12px', background: 'linear-gradient(to right, #fff, #00e5ff)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent', letterSpacing: '-0.5px', textAlign: 'center' }}>Thunder Wallet</h2>
      
      {view === 'home' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', width: '100%', marginTop: '30px' }}>
          <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginBottom: '10px', fontSize: '14px' }}>Your gateway to the next-gen blockchain.</p>
          <button className="btn-primary" onClick={handleCreate} style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '8px' }}>
            <Rocket size={18} /> Create New Wallet
          </button>
          <button className="btn-outline" onClick={() => setView('import')} style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '8px' }}>
            <Download size={18} /> Import Seed Phrase
          </button>
        </div>
      )}

      {view === 'create' && (
        <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', width: '100%', alignItems: 'center', marginBottom: '16px' }}>
            <p style={{ color: 'var(--text-secondary)', fontSize: '14px', margin: 0, textAlign: 'left' }}>
              Write down these 12 words:
            </p>
            <button onClick={handleCopy} style={{ background: 'rgba(0, 229, 255, 0.1)', border: '1px solid rgba(0, 229, 255, 0.2)', color: 'var(--cyan)', display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer', fontSize: '12px', padding: '4px 10px', borderRadius: '6px', fontWeight: 600 }}>
              {copied ? <Check size={14} /> : <Copy size={14} />}
              {copied ? 'Copied' : 'Copy'}
            </button>
          </div>
          <div style={{ 
            display: 'grid', 
            gridTemplateColumns: 'repeat(3, 1fr)', 
            gap: '8px', 
            background: 'var(--bg-card)', 
            padding: '16px', 
            borderRadius: '16px',
            border: '1px solid var(--border-color)',
            boxShadow: 'inset 0 0 20px rgba(0, 0, 0, 0.5)',
            width: '100%',
            marginBottom: '16px'
          }}>
            {generatedMnemonic.split(' ').map((word, i) => (
              <div key={i} style={{ display: 'flex', gap: '4px', fontSize: '13px', background: 'rgba(255,255,255,0.03)', padding: '6px 8px', borderRadius: '8px' }}>
                <span style={{ color: 'var(--cyan)', fontWeight: 'bold' }}>{i + 1}.</span>
                <span style={{ color: 'white', overflow: 'hidden', textOverflow: 'ellipsis' }}>{word}</span>
              </div>
            ))}
          </div>
          {error && <p style={{ color: '#ff4d4f', fontSize: '13px', marginBottom: '16px', fontWeight: 500 }}>{error}</p>}
          <button className="btn-primary" style={{ width: '100%' }} onClick={handleConfirmCreate} disabled={isLoading}>
            {isLoading ? 'Creating Wallet...' : 'I have saved it'}
          </button>
          <button className="btn-outline" style={{ width: '100%', marginTop: '12px', border: 'none' }} onClick={() => setView('home')} disabled={isLoading}>
            Back
          </button>
        </div>
      )}

      {view === 'import' && (
        <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginBottom: '20px', fontSize: '14px' }}>
            Enter your 12-word seed phrase to restore your wallet.
          </p>
          <textarea
            className="input-field"
            style={{ width: '100%', minHeight: '120px', resize: 'none', marginBottom: '16px', lineHeight: '1.5' }}
            placeholder="Enter the 12 words separated by spaces..."
            value={mnemonicInput}
            onChange={(e) => setMnemonicInput(e.target.value)}
          />
          {error && <p style={{ color: '#ff4d4f', fontSize: '13px', marginBottom: '16px', fontWeight: 500 }}>{error}</p>}
          <button className="btn-primary" style={{ width: '100%' }} onClick={handleImport}>
            <KeyRound size={18} style={{ marginRight: '8px', display: 'inline' }} />
            Import Wallet
          </button>
          <button className="btn-outline" style={{ width: '100%', marginTop: '12px', border: 'none' }} onClick={() => setView('home')}>
            Back
          </button>
        </div>
      )}
    </div>
  )
}
