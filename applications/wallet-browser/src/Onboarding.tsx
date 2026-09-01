import { useState } from 'react'
import { ethers } from 'ethers'
import { Rocket, Download, KeyRound } from 'lucide-react'
import './index.css'

interface OnboardingProps {
  onComplete: (privateKey: string, mnemonic: string, address: string) => void;
}

export default function Onboarding({ onComplete }: OnboardingProps) {
  const [view, setView] = useState<'home' | 'create' | 'import'>('home')
  const [mnemonicInput, setMnemonicInput] = useState('')
  const [generatedMnemonic, setGeneratedMnemonic] = useState('')
  const [error, setError] = useState('')

  const handleCreate = () => {
    try {
      const wallet = ethers.Wallet.createRandom()
      if (wallet.mnemonic) {
        setGeneratedMnemonic(wallet.mnemonic.phrase)
        setView('create')
      }
    } catch (err) {
      console.error(err)
      setError('Failed to create wallet')
    }
  }

  const handleConfirmCreate = async () => {
    try {
      const wallet = ethers.Wallet.fromPhrase(generatedMnemonic)
      const res = await fetch('http://localhost:5050/api/wallet/derive-address', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ private_key: wallet.privateKey })
      })
      const data = await res.json()
      if (data.address) {
        onComplete(wallet.privateKey, generatedMnemonic, data.address)
      } else {
        setError('Failed to derive address')
      }
    } catch (err) {
      console.error(err)
    }
  }

  const handleImport = async () => {
    try {
      const wallet = ethers.Wallet.fromPhrase(mnemonicInput.trim())
      const res = await fetch('http://localhost:5050/api/wallet/derive-address', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ private_key: wallet.privateKey })
      })
      const data = await res.json()
      if (data.address) {
        onComplete(wallet.privateKey, mnemonicInput.trim(), data.address)
      } else {
        setError('Invalid Seed Phrase')
      }
    } catch (err) {
      console.error(err)
      setError('Invalid Seed Phrase')
    }
  }

  return (
    <div className="app-container" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh', padding: '24px' }}>
      <img src="/logo.png" alt="Thunder Logo" style={{ width: 80, height: 80, marginBottom: 24, filter: 'drop-shadow(0 0 15px rgba(139, 92, 246, 0.6))' }} />
      <h2 style={{ fontSize: '28px', fontWeight: '800', marginBottom: '12px', background: 'linear-gradient(to right, #fff, #00e5ff)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent', letterSpacing: '-0.5px' }}>Thunder Wallet</h2>
      
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
          <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginBottom: '20px', fontSize: '14px' }}>
            Write down these 12 words in order and keep them safe.
          </p>
          <div style={{ 
            display: 'grid', 
            gridTemplateColumns: '1fr 1fr', 
            gap: '12px', 
            background: 'var(--bg-card)', 
            padding: '20px', 
            borderRadius: '16px',
            border: '1px solid var(--border-color)',
            boxShadow: 'inset 0 0 20px rgba(0, 0, 0, 0.5)',
            width: '100%',
            marginBottom: '24px'
          }}>
            {generatedMnemonic.split(' ').map((word, i) => (
              <div key={i} style={{ display: 'flex', gap: '8px', fontSize: '14px', background: 'rgba(255,255,255,0.03)', padding: '6px 10px', borderRadius: '8px' }}>
                <span style={{ color: 'var(--cyan)', width: '20px', fontWeight: 'bold' }}>{i + 1}.</span>
                <span style={{ color: 'white' }}>{word}</span>
              </div>
            ))}
          </div>
          <button className="btn-primary" style={{ width: '100%' }} onClick={handleConfirmCreate}>
            I have saved it
          </button>
          <button className="btn-outline" style={{ width: '100%', marginTop: '12px', border: 'none' }} onClick={() => setView('home')}>
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
