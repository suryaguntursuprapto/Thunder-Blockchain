import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Droplet, ExternalLink, ShieldCheck, Zap, AlertTriangle, ArrowRight } from 'lucide-react'
import '../App.css'

export function Faucet() {
  const [address, setAddress] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [result, setResult] = useState<{ type: 'success' | 'error', message: string, txHash?: string } | null>(null)
  const [isFocused, setIsFocused] = useState(false)

  const handleRequest = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!address.startsWith('0x') || address.length < 40) {
      setResult({ type: 'error', message: 'Invalid address format. Must start with 0x and be 40+ characters long.' })
      return
    }

    setIsLoading(true)
    setResult(null)

    try {
      const res = await fetch('http://127.0.0.1:5050/api/faucet', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ address })
      })

      const data = await res.json()

      if (!res.ok) {
        throw new Error(data.error || 'Failed to request faucet')
      }

      setResult({
        type: 'success',
        message: '1000 THDR successfully sent to your wallet!',
        txHash: data.tx_hash
      })
      setAddress('')
    } catch (err: any) {
      setResult({
        type: 'error',
        message: err.message || 'An unexpected error occurred. Is the node running?'
      })
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="landing-page" style={{ minHeight: '100vh', paddingTop: '100px', position: 'relative', overflow: 'hidden' }}>
      {/* Dynamic Background Elements */}
      <div className="bg-gradient-top"></div>
      <div className="bg-grid"></div>
      
      {/* Decorative Orbs */}
      <div style={{ position: 'absolute', top: '10%', left: '15%', width: '300px', height: '300px', background: 'radial-gradient(circle, rgba(162, 0, 255, 0.15) 0%, transparent 70%)', filter: 'blur(40px)', zIndex: 0, animation: 'float 10s ease-in-out infinite' }}></div>
      <div style={{ position: 'absolute', top: '40%', right: '10%', width: '400px', height: '400px', background: 'radial-gradient(circle, rgba(0, 229, 255, 0.1) 0%, transparent 70%)', filter: 'blur(50px)', zIndex: 0, animation: 'float 15s ease-in-out infinite reverse' }}></div>

      <div className="hero-section" style={{ padding: '60px 20px', minHeight: 'auto', position: 'relative', zIndex: 1 }}>
        <div className="hero-content" style={{ maxWidth: '1200px', margin: '0 auto' }}>
          
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.7, ease: "easeOut" }}
            style={{ textAlign: 'center', marginBottom: '50px' }}
          >
            <div style={{ 
              display: 'inline-flex', 
              alignItems: 'center', 
              justifyContent: 'center', 
              width: '80px', 
              height: '80px', 
              borderRadius: '24px', 
              background: 'linear-gradient(135deg, rgba(162,0,255,0.2) 0%, rgba(0,229,255,0.2) 100%)',
              border: '1px solid rgba(255,255,255,0.1)',
              boxShadow: '0 0 30px rgba(162,0,255,0.3)',
              marginBottom: '24px'
            }}>
              <Droplet size={40} color="#00e5ff" style={{ filter: 'drop-shadow(0 0 10px rgba(0,229,255,0.8))' }} />
            </div>
            
            <h1 className="hero-title" style={{ fontSize: 'clamp(40px, 6vw, 64px)', letterSpacing: '-1px', marginBottom: '20px' }}>
              Thunder <span className="gradient-text">Faucet</span>
            </h1>
            <p className="hero-subtitle" style={{ maxWidth: '600px', margin: '0 auto', color: 'var(--text-secondary)', fontSize: '18px', lineHeight: '1.6' }}>
              Fund your development wallet instantly. Request testnet THDR tokens to build and test decentralized applications on the Thunder Network.
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            style={{ maxWidth: '650px', margin: '0 auto', perspective: '1000px' }}
          >
            <div style={{ 
              background: 'rgba(15, 15, 20, 0.6)', 
              backdropFilter: 'blur(20px)',
              WebkitBackdropFilter: 'blur(20px)',
              padding: '40px', 
              borderRadius: '30px', 
              border: '1px solid rgba(255,255,255,0.05)', 
              boxShadow: isFocused ? '0 30px 60px rgba(0,0,0,0.6), 0 0 40px rgba(0,229,255,0.1)' : '0 20px 40px rgba(0,0,0,0.5)', 
              position: 'relative',
              transition: 'all 0.5s cubic-bezier(0.4, 0, 0.2, 1)',
              transformStyle: 'preserve-3d',
              zIndex: 2
            }}>
              {/* Glowing Top Border */}
              <div style={{ position: 'absolute', top: 0, left: '50%', transform: 'translateX(-50%)', width: '60%', height: '1px', background: 'linear-gradient(90deg, transparent, var(--cyan), transparent)', opacity: 0.5 }}></div>

              <form onSubmit={handleRequest} style={{ position: 'relative', zIndex: 10 }}>
                <div style={{ marginBottom: '30px', textAlign: 'left' }}>
                  <label style={{ display: 'flex', alignItems: 'center', marginBottom: '12px', color: '#a0aec0', fontSize: '14px', fontWeight: '600', letterSpacing: '0.5px', textTransform: 'uppercase' }}>
                    <Zap size={16} style={{ marginRight: '8px', color: 'var(--cyan)' }} />
                    Wallet Address
                  </label>
                  <div style={{ position: 'relative' }}>
                    <input
                      type="text"
                      value={address}
                      onChange={(e) => setAddress(e.target.value)}
                      onFocus={() => setIsFocused(true)}
                      onBlur={() => setIsFocused(false)}
                      placeholder="Enter your 0x... address"
                      style={{
                        width: '100%',
                        padding: '18px 24px',
                        borderRadius: '16px',
                        border: '1px solid',
                        borderColor: isFocused ? 'rgba(0, 229, 255, 0.5)' : 'rgba(255,255,255,0.1)',
                        background: isFocused ? 'rgba(0,0,0,0.4)' : 'rgba(0,0,0,0.2)',
                        color: 'white',
                        fontSize: '16px',
                        fontFamily: 'monospace',
                        outline: 'none',
                        transition: 'all 0.3s ease',
                        boxShadow: isFocused ? 'inset 0 2px 10px rgba(0,0,0,0.5), 0 0 20px rgba(0,229,255,0.1)' : 'inset 0 2px 5px rgba(0,0,0,0.2)'
                      }}
                    />
                    {address && (
                      <div style={{ position: 'absolute', right: '16px', top: '50%', transform: 'translateY(-50%)' }}>
                        {address.length >= 40 && address.startsWith('0x') ? (
                          <ShieldCheck size={20} color="#00ff80" />
                        ) : (
                          <AlertTriangle size={20} color="#ff4d4f" />
                        )}
                      </div>
                    )}
                  </div>
                </div>

                <button 
                  type="submit" 
                  disabled={isLoading || !address}
                  className="btn-primary" 
                  style={{ 
                    width: '100%', 
                    padding: '18px', 
                    fontSize: '16px', 
                    fontWeight: '600',
                    display: 'flex', 
                    alignItems: 'center', 
                    justifyContent: 'center', 
                    gap: '10px',
                    opacity: isLoading || !address ? 0.6 : 1, 
                    cursor: isLoading || !address ? 'not-allowed' : 'pointer',
                    borderRadius: '16px',
                    boxShadow: isLoading || !address ? 'none' : '0 10px 20px rgba(162, 0, 255, 0.3)',
                    transform: isLoading ? 'scale(0.98)' : 'scale(1)',
                    transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)'
                  }}
                >
                  {isLoading ? (
                    <>
                      <div className="spinner" style={{ width: '20px', height: '20px', border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 1s linear infinite' }}></div>
                      Processing Request...
                    </>
                  ) : (
                    <>
                      Receive 1000 THDR <ArrowRight size={18} />
                    </>
                  )}
                </button>
              </form>

              <AnimatePresence>
                {result && (
                  <motion.div
                    initial={{ opacity: 0, height: 0, marginTop: 0 }}
                    animate={{ opacity: 1, height: 'auto', marginTop: '24px' }}
                    exit={{ opacity: 0, height: 0, marginTop: 0 }}
                    transition={{ duration: 0.3 }}
                    style={{ overflow: 'hidden' }}
                  >
                    <div style={{
                      padding: '20px',
                      borderRadius: '16px',
                      background: result.type === 'success' ? 'rgba(0, 255, 128, 0.05)' : 'rgba(255, 77, 79, 0.05)',
                      border: `1px solid ${result.type === 'success' ? 'rgba(0, 255, 128, 0.2)' : 'rgba(255, 77, 79, 0.2)'}`,
                      textAlign: 'left',
                      position: 'relative'
                    }}>
                      {/* Success Glow */}
                      {result.type === 'success' && <div style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', background: 'radial-gradient(circle at center, rgba(0,255,128,0.1) 0%, transparent 70%)', pointerEvents: 'none' }}></div>}
                      
                      <h3 style={{ color: result.type === 'success' ? '#00ff80' : '#ff4d4f', display: 'flex', alignItems: 'center', marginBottom: '12px', fontSize: '18px', position: 'relative', zIndex: 1 }}>
                        {result.type === 'success' ? <ShieldCheck size={22} style={{ marginRight: '8px' }} /> : <AlertTriangle size={22} style={{ marginRight: '8px' }} />}
                        {result.type === 'success' ? 'Transaction Successful!' : 'Request Failed'}
                      </h3>
                      <p style={{ color: '#cbd5e1', fontSize: '15px', lineHeight: '1.6', position: 'relative', zIndex: 1 }}>{result.message}</p>
                      
                      {result.txHash && (
                        <div style={{ marginTop: '16px', padding: '16px', background: 'rgba(0,0,0,0.4)', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.05)', position: 'relative', zIndex: 1 }}>
                          <div style={{ color: '#8b9bb4', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '1px', marginBottom: '8px' }}>Transaction Hash</div>
                          <div style={{ color: 'var(--cyan)', fontFamily: 'monospace', fontSize: '13px', wordBreak: 'break-all', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            {result.txHash}
                            <a href={`/thunderscan/testnet?tx=${result.txHash}`} target="_blank" rel="noreferrer" style={{ color: 'var(--cyan)', opacity: 0.7, padding: '4px' }}>
                              <ExternalLink size={16} />
                            </a>
                          </div>
                        </div>
                      )}
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </motion.div>

          <motion.div 
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.4 }}
            style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '30px', maxWidth: '1000px', margin: '80px auto 0' }}
          >
            <div className="feature-card reveal" style={{ opacity: 1, transform: 'none', background: 'rgba(15, 15, 20, 0.4)', backdropFilter: 'blur(10px)', WebkitBackdropFilter: 'blur(10px)', border: '1px solid rgba(255,255,255,0.03)', transition: 'transform 0.3s ease, box-shadow 0.3s ease' }} onMouseEnter={(e) => { e.currentTarget.style.transform = 'translateY(-5px)'; e.currentTarget.style.boxShadow = '0 10px 30px rgba(162,0,255,0.1)'; }} onMouseLeave={(e) => { e.currentTarget.style.transform = 'none'; e.currentTarget.style.boxShadow = 'none'; }}>
              <div className="feature-icon" style={{ background: 'rgba(162,0,255,0.1)' }}><Zap size={24} color="#a200ff" /></div>
              <h3 style={{ fontSize: '18px', marginBottom: '12px' }}>Instant Execution</h3>
              <p style={{ color: '#8b9bb4', fontSize: '15px', lineHeight: '1.6' }}>Smart contracts process faucet requests in less than a second, delivering testnet tokens directly to your wallet.</p>
            </div>
            <div className="feature-card reveal" style={{ opacity: 1, transform: 'none', background: 'rgba(15, 15, 20, 0.4)', backdropFilter: 'blur(10px)', WebkitBackdropFilter: 'blur(10px)', border: '1px solid rgba(255,255,255,0.03)', transition: 'transform 0.3s ease, box-shadow 0.3s ease' }} onMouseEnter={(e) => { e.currentTarget.style.transform = 'translateY(-5px)'; e.currentTarget.style.boxShadow = '0 10px 30px rgba(0,229,255,0.1)'; }} onMouseLeave={(e) => { e.currentTarget.style.transform = 'none'; e.currentTarget.style.boxShadow = 'none'; }}>
              <div className="feature-icon" style={{ background: 'rgba(0,229,255,0.1)' }}><Droplet size={24} color="#00e5ff" /></div>
              <h3 style={{ fontSize: '18px', marginBottom: '12px' }}>1000 THDR Quota</h3>
              <p style={{ color: '#8b9bb4', fontSize: '15px', lineHeight: '1.6' }}>Each request automatically dispenses 1000 THDR. Designed for developers building robust dApps.</p>
            </div>
            <div className="feature-card reveal" style={{ opacity: 1, transform: 'none', background: 'rgba(15, 15, 20, 0.4)', backdropFilter: 'blur(10px)', WebkitBackdropFilter: 'blur(10px)', border: '1px solid rgba(255,255,255,0.03)', transition: 'transform 0.3s ease, box-shadow 0.3s ease' }} onMouseEnter={(e) => { e.currentTarget.style.transform = 'translateY(-5px)'; e.currentTarget.style.boxShadow = '0 10px 30px rgba(255,255,255,0.05)'; }} onMouseLeave={(e) => { e.currentTarget.style.transform = 'none'; e.currentTarget.style.boxShadow = 'none'; }}>
              <div className="feature-icon" style={{ background: 'rgba(255,255,255,0.05)' }}><ExternalLink size={24} color="#fff" /></div>
              <h3 style={{ fontSize: '18px', marginBottom: '12px' }}>Fully Transparent</h3>
              <p style={{ color: '#8b9bb4', fontSize: '15px', lineHeight: '1.6' }}>Track all faucet distributions natively on the ThunderScan block explorer in real-time.</p>
            </div>
          </motion.div>
        </div>
      </div>
      
      <style>{`
        @keyframes spin {
          100% { transform: rotate(360deg); }
        }
        @keyframes float {
          0% { transform: translateY(0px) translateX(0px); }
          50% { transform: translateY(-20px) translateX(20px); }
          100% { transform: translateY(0px) translateX(0px); }
        }
      `}</style>
    </div>
  )
}
