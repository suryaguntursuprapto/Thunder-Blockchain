import { useState, useEffect } from 'react'
import { Send as SendIcon, ArrowDownToLine, Coins, LayoutGrid, ChevronDown, ArrowLeft, LogOut, Copy, Check, X, AlertTriangle } from 'lucide-react'
import Onboarding from './Onboarding'
import './index.css'

type Tab = 'tokens' | 'nfts' | 'activity'
type View = 'home' | 'send' | 'receive' | 'stake' | 'mint' | 'token_details' | 'onboarding'

const NETWORKS = [
  { id: 'mainnet', name: 'Thunder Mainnet', rpcUrl: 'https://rpc.thunder-network.com', symbol: 'THDR' },
  { id: 'testnet', name: 'Thunder Testnet', rpcUrl: 'http://127.0.0.1:8080', symbol: 'THDR' }
]

interface ThunderWallet {
  privateKey: string;
  address: string;
}

function App() {
  const [wallets, setWallets] = useState<ThunderWallet[]>([])
  const [activeWalletIndex, setActiveWalletIndex] = useState(0)
  const wallet = wallets[activeWalletIndex] || null

  const [activeTab, setActiveTab] = useState<Tab>('tokens')
  const [view, setView] = useState<View>('home')
  const [poolAddress, setPoolAddress] = useState('')

  // Network & Balance State
  const [network, setNetwork] = useState(NETWORKS[1]) // Default to Testnet for now
  const [showNetworkDropdown, setShowNetworkDropdown] = useState(false)
  const [balance, setBalance] = useState('0.00')
  const [isLoading, setIsLoading] = useState(true)
  const [transactions, setTransactions] = useState<any[]>([])
  const [isLoadingActivity, setIsLoadingActivity] = useState(false)
  const [stakedBalance, setStakedBalance] = useState('0')
  const [modal, setModal] = useState<{ show: boolean, type: 'success' | 'error' | 'confirm', message: string, onConfirm?: () => void }>({ show: false, type: 'success', message: '' })

  useEffect(() => {
    const loadWallets = async () => {
      try {
        const storedWallets = localStorage.getItem('thunder_wallets')
        if (storedWallets) {
          const parsed = JSON.parse(storedWallets)
          setWallets(parsed)
          const activeIdx = localStorage.getItem('thunder_active_wallet_index')
          if (activeIdx) setActiveWalletIndex(parseInt(activeIdx))
        } else {
          // Backward compatibility
          const storedKey = localStorage.getItem('thunder_private_key')
          const storedAddress = localStorage.getItem('thunder_address')
          if (storedKey && storedAddress) {
            setWallets([{ privateKey: storedKey, address: storedAddress }])
            setActiveWalletIndex(0)
            localStorage.setItem('thunder_wallets', JSON.stringify([{ privateKey: storedKey, address: storedAddress }]))
          }
        }
      } catch (err) {
        console.error(err)
      } finally {
        setIsLoading(false)
      }
    }
    loadWallets()
  }, [])

  const handleOnboardingComplete = (privateKey: string, mnemonic: string, address: string) => {
    const newWallet = { privateKey, address }
    const updatedWallets = [...wallets, newWallet]
    setWallets(updatedWallets)
    const newIndex = updatedWallets.length - 1
    setActiveWalletIndex(newIndex)
    
    localStorage.setItem('thunder_wallets', JSON.stringify(updatedWallets))
    localStorage.setItem('thunder_active_wallet_index', newIndex.toString())
    localStorage.setItem('thunder_mnemonic', mnemonic)
    
    if (view === 'onboarding') {
      setView('home')
    }
  }

  const handleAddAccount = async () => {
    setShowAccountDropdown(false)
    setView('onboarding')
  }

  const handleSwitchAccount = (index: number) => {
    setActiveWalletIndex(index)
    localStorage.setItem('thunder_active_wallet_index', index.toString())
    setShowAccountDropdown(false)
  }

  const [showAccountDropdown, setShowAccountDropdown] = useState(false)

  // Fetch Live Balance
  useEffect(() => {
    let interval: any;
    const fetchBalance = async () => {
      if (!wallet) return
      try {
        if (network.id === 'testnet') {
          const res = await fetch(network.rpcUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              method: 'thunder_getBalance',
              params: { address: wallet.address },
              id: 1
            })
          })
          const data = await res.json()
          if (data.result && data.result.balance !== undefined) {
            const balanceEth = Number(data.result.balance) * 1e-9
            setBalance(balanceEth.toLocaleString('en-US', { maximumFractionDigits: 4 }))
          } else {
            setBalance('0.00')
          }
          
          const valRes = await fetch(network.rpcUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              method: 'thunder_getValidators',
              params: [],
              id: 2
            })
          })
          const valData = await valRes.json()
          let stakedAmount = '0'
          if (valData.result && valData.result.validators) {
            const myValidator = valData.result.validators.find((v: any) => v.address.toLowerCase() === wallet.address.toLowerCase())
            if (myValidator) {
              stakedAmount = (Number(myValidator.stake) * 1e-9).toString()
            }
          }
          setStakedBalance(stakedAmount)
          
          const sysRes = await fetch(network.rpcUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              method: 'thunder_getSystemInfo',
              params: {},
              id: 3
            })
          })
          const sysData = await sysRes.json()
          if (sysData.result && sysData.result.system_contracts && sysData.result.system_contracts.StakingPool) {
            if (!poolAddress || poolAddress === '') {
              setPoolAddress(sysData.result.system_contracts.StakingPool)
            }
          }

        } else {
          setBalance('0.00')
          setStakedBalance('0')
        }
      } catch (err) {
        console.error('Failed to fetch balance', err)
        setBalance('0.00')
      }
    }

    fetchBalance()
    interval = setInterval(fetchBalance, 5000)
    return () => clearInterval(interval)
  }, [wallet, network])

  // Fetch Activity
  useEffect(() => {
    let interval: any;
    const fetchActivity = async () => {
      if (!wallet || network.id !== 'testnet') {
        setTransactions([])
        return
      }

      try {
        // Only set loading on first fetch
        if (transactions.length === 0) setIsLoadingActivity(true)
        const res = await fetch(`http://127.0.0.1:5050/api/account/${wallet.address}`)
        if (res.ok) {
          const data = await res.json()
          if (data.transactions) {
            setTransactions(data.transactions)
          } else {
            setTransactions([])
          }
        }
      } catch (err) {
        console.error('Failed to fetch activity', err)
      } finally {
        setIsLoadingActivity(false)
      }
    }

    if (activeTab === 'activity' || view === 'token_details') {
      fetchActivity()
      interval = setInterval(fetchActivity, 5000)
    }
    return () => clearInterval(interval)
  }, [wallet, network, activeTab, view])

  // Send Form State
  const [sendTo, setSendTo] = useState('')
  const [sendAmount, setSendAmount] = useState('')

  // Stake Form State
  const [stakeAmount, setStakeAmount] = useState('')

  // Mint Form State
  const [nftName, setNftName] = useState('')
  const [nftUrl, setNftUrl] = useState('')

  const [copied, setCopied] = useState(false)

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!wallet) return
    if (!sendTo || !sendAmount) {
      setModal({ show: true, type: 'error', message: "Please enter recipient address and amount" })
      return
    }

    setModal({
      show: true,
      type: 'confirm',
      message: `Are you sure you want to send ${sendAmount} THDR to ${sendTo.substring(0, 8)}...?`,
      onConfirm: async () => {
        setModal({ ...modal, show: false })
        try {
          const amountInNano = Math.floor(parseFloat(sendAmount) * 1e9).toString()
          console.log(`Sending ${sendAmount} THDR (${amountInNano} nano) to ${sendTo}...`)
          const res = await fetch('http://127.0.0.1:5050/api/tx/send', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              to: sendTo,
              amount: amountInNano,
              private_key: wallet.privateKey.replace('0x', '') // Pass raw hex without 0x
            })
          });

          const data = await res.json();
          if (res.ok && data.success) {
            setModal({ show: true, type: 'success', message: `Successfully sent ${sendAmount} THDR to ${sendTo.substring(0, 8)}...` })
            setSendTo('');
            setSendAmount('');
            setView('home');
            setActiveTab('activity');
          } else {
            setModal({ show: true, type: 'error', message: data.error || 'Unknown error occurred while sending.' })
          }
        } catch (err: any) {
          setModal({ show: true, type: 'error', message: err.message || 'Failed to send transaction' })
        }
      }
    })
  }

    const handleDepositPool = async () => {
      if (!wallet) return
      if (!stakeAmount || isNaN(parseFloat(stakeAmount)) || parseFloat(stakeAmount) <= 0) {
        setModal({ show: true, type: 'error', message: 'Please enter a valid deposit amount' })
        return
      }
      if (!poolAddress) {
        setModal({ show: true, type: 'error', message: 'Staking pool is not yet available on this network.' })
        return
      }

      setModal({
        show: true,
        type: 'confirm',
        message: `Are you sure you want to deposit ${stakeAmount} THDR into Thunder System Staking Pool?`,
        onConfirm: async () => {
          setModal({ ...modal, show: false })
          setIsLoading(true)
          try {
            const amountInNano = (parseFloat(stakeAmount) * 1e9).toString()
            const response = await fetch(`${network.id === 'testnet' ? 'http://127.0.0.1:5050' : 'https://api.thunder-network.com'}/api/tx/call`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                  private_key: wallet.privateKey,
                  to: poolAddress,
                  function: "deposit",
                  amount: amountInNano
                })
            });

            const data = await response.json()
            if (data.error) throw new Error(data.error)
            
            setModal({ show: true, type: 'success', message: `Successfully deposited ${stakeAmount} THDR into the pool!` })
            setStakeAmount('')
            setView('home')
          } catch (err: any) {
            setModal({ show: true, type: 'error', message: err.message || 'Failed to deposit into pool' })
          } finally {
            setIsLoading(false)
          }
        }
      })
    }

  const handleMint = () => {
    alert(`Minting NFT: ${nftName}...`)
    setView('home')
  }

  const handleLogout = () => {
    if (wallets.length > 1) {
      const updatedWallets = wallets.filter((_, idx) => idx !== activeWalletIndex)
      setWallets(updatedWallets)
      setActiveWalletIndex(0)
      localStorage.setItem('thunder_wallets', JSON.stringify(updatedWallets))
      localStorage.setItem('thunder_active_wallet_index', '0')
    } else {
      localStorage.removeItem('thunder_wallets')
      localStorage.removeItem('thunder_active_wallet_index')
      localStorage.removeItem('thunder_mnemonic')
      setWallets([])
      setActiveWalletIndex(0)
    }
  }

  const handleCopyAddress = () => {
    if (wallet) {
      navigator.clipboard.writeText(wallet.address)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

    const renderModal = () => {
    if (!modal.show) return null;
    return (
      <div className="modal-overlay" onClick={() => setModal({ ...modal, show: false })}>
        <div className={`modal-content ${modal.type === 'error' ? 'error' : ''}`} onClick={e => e.stopPropagation()}>
          <div className={`modal-icon-container ${modal.type === 'error' ? 'error' : ''}`}>
            {modal.type === 'success' ? <Check size={32} /> : modal.type === 'error' ? <X size={32} /> : <AlertTriangle size={32} />}
          </div>
          <h3 className="modal-title">{modal.type === 'success' ? 'Transaction Success' : modal.type === 'error' ? 'Transaction Failed' : 'Confirm Transaction'}</h3>
          <p className="modal-message">{modal.message}</p>
          {modal.type === 'confirm' ? (
            <div style={{ display: 'flex', gap: '12px', marginTop: '24px' }}>
              <button
                className="btn-outline"
                style={{ flex: 1, padding: '12px', borderRadius: '12px', border: '1px solid var(--border)', background: 'transparent', color: 'white', cursor: 'pointer' }}
                onClick={() => setModal({ ...modal, show: false })}
              >
                Cancel
              </button>
              <button
                className="btn-primary"
                style={{ flex: 1, padding: '12px', borderRadius: '12px', background: 'var(--cyan-dim)', border: '1px solid var(--cyan)', color: 'var(--cyan)', fontWeight: 600, cursor: 'pointer' }}
                onClick={modal.onConfirm}
              >
                Confirm
              </button>
            </div>
          ) : (
            <button
              className="btn-primary"
              style={{ width: '100%', padding: '12px', marginTop: '24px', borderRadius: '12px', background: 'var(--bg-secondary)', border: '1px solid var(--border)', color: 'white', cursor: 'pointer' }}
              onClick={() => setModal({ ...modal, show: false })}
            >
              Close
            </button>
          )}
        </div>
      </div>
    );
  };

  if (view === 'send') {
    return (
      <div className="app-container">
        <header className="header" style={{ justifyContent: 'flex-start', gap: 16 }}>
          <ArrowLeft size={20} style={{ cursor: 'pointer' }} onClick={() => setView('home')} />
          <h2 style={{ fontSize: '1.1rem' }}>Send THDR</h2>
        </header>
        <main className="main-content">
          <div className="form-group" style={{ marginBottom: 16 }}>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 8 }}>Send to</label>
            <input
              type="text"
              placeholder="0x..."
              value={sendTo}
              onChange={e => setSendTo(e.target.value)}
              style={{ width: '100%', padding: '12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: 'white' }}
            />
          </div>
          <div className="form-group" style={{ marginBottom: 24 }}>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 8 }}>Amount</label>
            <input
              type="number"
              placeholder="0.0"
              value={sendAmount}
              onChange={e => setSendAmount(e.target.value)}
              style={{ width: '100%', padding: '12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: 'white', fontSize: '1.2rem' }}
            />
          </div>
          <button
            className="btn-outline"
            onClick={handleSend}
            style={{ width: '100%', padding: '14px', borderRadius: 12, background: 'var(--cyan-dim)', border: '1px solid var(--cyan)', color: 'var(--cyan)', fontWeight: 600, cursor: 'pointer' }}>
            Confirm Send
          </button>
        </main>
        {renderModal()}
      </div>
    )
  }

  if (view === 'receive') {
    return (
      <div className="app-container">
        <header className="header" style={{ justifyContent: 'flex-start', gap: 16 }}>
          <ArrowLeft size={20} style={{ cursor: 'pointer' }} onClick={() => setView('home')} />
          <h2 style={{ fontSize: '1.1rem' }}>Receive THDR</h2>
        </header>
        <main className="main-content" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <div style={{ padding: 24, background: 'white', borderRadius: 16, marginBottom: 24 }}>
            {/* Fake QR Code */}
            <div style={{ width: 200, height: 200, background: '#000', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <div style={{ width: 180, height: 180, background: 'repeating-conic-gradient(#000 0% 25%, #fff 0% 50%) 50% / 20px 20px' }}></div>
            </div>
          </div>
          <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem', marginBottom: 8 }}>Your Wallet Address</p>
          <div
            style={{
              background: 'var(--bg-card)',
              border: '1px solid var(--border-color)',
              padding: '12px 16px',
              borderRadius: 12,
              width: '100%',
              textAlign: 'center',
              wordBreak: 'break-all',
              fontSize: '0.85rem',
              marginBottom: 24
            }}
          >
            {wallet?.address}
          </div>
          <button
            className="btn-primary"
            onClick={handleCopyAddress}
            style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8 }}>
            {copied ? <Check size={18} /> : <Copy size={18} />}
            {copied ? 'Copied!' : 'Copy Address'}
          </button>
        </main>
        {renderModal()}
      </div>
    )
  }

  if (view === 'stake') {
    return (
      <div className="app-container">
        <header className="header" style={{ justifyContent: 'flex-start', gap: 16 }}>
          <ArrowLeft size={20} style={{ cursor: 'pointer' }} onClick={() => setView('home')} />
          <h2 style={{ fontSize: '1.1rem' }}>Staking Pools</h2>
        </header>
        <main className="main-content">
          <div style={{ textAlign: 'center', marginBottom: 24 }}>
            <div className="action-icon" style={{ margin: '0 auto', marginBottom: 16 }}>
              <Coins size={24} />
            </div>
            <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem' }}>Deploy a new pool or join an existing one by depositing THDR.</p>
          </div>
          

          
          <h3 style={{ fontSize: '1rem', marginBottom: 16 }}>Deposit into Pool</h3>

          <div className="form-group" style={{ marginBottom: 16 }}>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 8 }}>Pool Name</label>
            <div style={{ width: '100%', padding: '12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: poolAddress ? 'var(--cyan)' : 'var(--text-secondary)', fontSize: '1rem', fontWeight: 600 }}>
              {poolAddress ? 'Thunder System Staking Pool' : 'Detecting StakingPool...'}
            </div>
          </div>

          <div className="form-group" style={{ marginBottom: 24 }}>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 8 }}>Amount to Deposit</label>
            <input
              type="number"
              placeholder="0.0"
              value={stakeAmount}
              onChange={e => setStakeAmount(e.target.value)}
              style={{ width: '100%', padding: '12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: 'white', fontSize: '1.2rem' }}
            />
            <div style={{ textAlign: 'right', fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: 8 }}>Balance: {balance} THDR</div>
          </div>

          <button
            className="btn-outline"
            onClick={handleDepositPool}
            style={{ width: '100%', padding: '14px', borderRadius: 12, background: 'var(--cyan-dim)', border: '1px solid var(--cyan)', color: 'var(--cyan)', fontWeight: 600, cursor: 'pointer' }}>
            Deposit to Pool
          </button>
        </main>
        {renderModal()}
      </div>
    )
  }

  if (view === 'mint') {
    return (
      <div className="app-container">
        <header className="header" style={{ justifyContent: 'flex-start', gap: 16 }}>
          <ArrowLeft size={20} style={{ cursor: 'pointer' }} onClick={() => setView('home')} />
          <h2 style={{ fontSize: '1.1rem' }}>Mint NFT</h2>
        </header>
        <main className="main-content">
          <div className="form-group" style={{ marginBottom: 16 }}>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 8 }}>NFT Name</label>
            <input
              type="text"
              placeholder="e.g. Thunder Ape #1"
              value={nftName}
              onChange={e => setNftName(e.target.value)}
              style={{ width: '100%', padding: '12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: 'white' }}
            />
          </div>
          <div className="form-group" style={{ marginBottom: 24 }}>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 8 }}>Image URL</label>
            <input
              type="text"
              placeholder="ipfs://..."
              value={nftUrl}
              onChange={e => setNftUrl(e.target.value)}
              style={{ width: '100%', padding: '12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: 'white' }}
            />
          </div>
          <button
            className="btn-outline"
            onClick={handleMint}
            style={{ width: '100%', padding: '14px', borderRadius: 12, background: 'rgba(139, 92, 246, 0.2)', border: '1px solid var(--purple)', color: '#b894ff', fontWeight: 600, cursor: 'pointer' }}>
            Mint NFT
          </button>
        </main>
        {renderModal()}
      </div>
    )
  }

  if (isLoading) return <div className="app-container" style={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>Loading...</div>
  if (!wallet || view === 'onboarding') {
    return <Onboarding onComplete={handleOnboardingComplete} onCancel={wallet ? () => setView('home') : undefined} />
  }

  const displayAddress = `${wallet.address.substring(0, 6)}...${wallet.address.substring(wallet.address.length - 4)}`

  return (
    <div className="app-container" onClick={() => setShowNetworkDropdown(false)}>
      {/* Header */}
      <header className="header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', gap: '8px' }}>
          <div
            className="network-selector"
            onClick={(e) => { e.stopPropagation(); setShowAccountDropdown(!showAccountDropdown); setShowNetworkDropdown(false); }}
            style={{ background: 'rgba(139, 92, 246, 0.1)', borderColor: 'rgba(139, 92, 246, 0.3)', color: '#b894ff' }}
          >
            Account {activeWalletIndex + 1}
            <ChevronDown size={14} style={{ marginLeft: 4 }} />

            {showAccountDropdown && (
              <div className="network-dropdown" style={{ left: 0, right: 'auto' }}>
                {wallets.map((w, idx) => (
                  <div
                    key={idx}
                    className="network-item"
                    onClick={() => handleSwitchAccount(idx)}
                    style={{ justifyContent: 'space-between' }}
                  >
                    <span>Account {idx + 1}</span>
                    <span style={{ fontSize: '0.75rem', opacity: 0.6 }}>
                      {w.address.substring(0, 4)}...{w.address.substring(w.address.length - 4)}
                    </span>
                  </div>
                ))}
                <div
                  className="network-item"
                  onClick={(e) => { e.stopPropagation(); handleAddAccount(); }}
                  style={{ borderTop: '1px solid rgba(255,255,255,0.1)', color: 'var(--cyan)', justifyContent: 'center' }}
                >
                  + Add Account
                </div>
              </div>
            )}
          </div>
        </div>

        <div
          className="account-icon"
          onClick={handleLogout}
          title="Logout"
          style={{ background: 'rgba(255, 77, 79, 0.1)', borderColor: 'rgba(255, 77, 79, 0.3)', boxShadow: '0 0 10px rgba(255, 77, 79, 0.2)' }}
        >
          <LogOut size={16} color="#ff4d4f" />
        </div>
      </header>

      {/* Dynamic View Rendering */}
      {view === 'token_details' && (
        <div className="view-container" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          {/* Header */}
          <div style={{ display: 'flex', alignItems: 'center', marginBottom: 32, padding: '0 8px' }}>
            <button
              onClick={() => setView('home')}
              style={{ background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '50%', width: 36, height: 36, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--text-primary)', cursor: 'pointer', transition: 'all 0.2s' }}
            >
              <ArrowLeft size={18} />
            </button>
            <div style={{ flex: 1, textAlign: 'center', marginRight: 36, fontSize: 16, fontWeight: 600, letterSpacing: '0.5px' }}>
              Thunder
            </div>
          </div>

          {/* Token Info & Balance */}
          <div style={{ display: 'flex', flexDirection: 'column', padding: '0 20px', marginBottom: 24 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <div style={{
                  width: 48, height: 48, borderRadius: 16,
                  background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(0, 229, 255, 0.1))',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  boxShadow: '0 4px 12px rgba(139, 92, 246, 0.1)'
                }}>
                  <img src="/logo.png" alt="Thunder" style={{ width: 28, height: 28 }} />
                </div>
                <div>
                  <div style={{ fontSize: 18, fontWeight: 600, color: '#fff' }}>Thunder</div>
                  <div style={{ fontSize: 13, color: 'var(--text-secondary)' }}>THDR</div>
                </div>
              </div>
              <div style={{ textAlign: 'right' }}>
                <div style={{ fontSize: 18, fontWeight: 600, color: '#fff' }}>{balance}</div>
                <div style={{ fontSize: 13, color: 'var(--text-secondary)' }}>$0.00</div>
              </div>
            </div>

            {/* Sparkline Chart (Mock) */}
            <div style={{ height: 100, width: '100%', marginBottom: 24, position: 'relative' }}>
              <svg viewBox="0 0 100 40" preserveAspectRatio="none" style={{ width: '100%', height: '100%', overflow: 'visible' }}>
                <defs>
                  <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--cyan)" stopOpacity="0.3" />
                    <stop offset="100%" stopColor="var(--cyan)" stopOpacity="0" />
                  </linearGradient>
                </defs>
                <path d="M0 40 L0 30 Q 10 20 20 25 T 40 15 T 60 20 T 80 5 T 100 10 L100 40 Z" fill="url(#chartGradient)" />
                <path d="M0 30 Q 10 20 20 25 T 40 15 T 60 20 T 80 5 T 100 10" fill="none" stroke="var(--cyan)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
              <div style={{ position: 'absolute', top: -10, right: 0, background: 'rgba(0, 229, 255, 0.1)', color: 'var(--cyan)', padding: '4px 8px', borderRadius: 8, fontSize: 12, fontWeight: 600 }}>
                +12.5%
              </div>
            </div>

            {/* Action Buttons */}
            <div style={{ display: 'flex', gap: 12 }}>
              <button
                style={{ flex: 1, padding: '12px', borderRadius: 12, background: 'var(--cyan-dim)', color: 'var(--cyan)', border: '1px solid rgba(0, 229, 255, 0.2)', fontWeight: 600, fontSize: 14, cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6, transition: 'all 0.2s' }}
                onClick={() => setView('send')}
              >
                <SendIcon size={16} /> Send
              </button>
              <button
                style={{ flex: 1, padding: '12px', borderRadius: 12, background: 'rgba(255,255,255,0.05)', color: 'var(--text-primary)', border: '1px solid rgba(255,255,255,0.1)', fontWeight: 600, fontSize: 14, cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6, transition: 'all 0.2s' }}
                onClick={() => setView('receive')}
              >
                <ArrowDownToLine size={16} /> Receive
              </button>
            </div>
          </div>

          {/* Activity List */}
          <div style={{ padding: '0 8px' }}>
            <h3 style={{ fontSize: 14, textTransform: 'uppercase', letterSpacing: '1px', color: 'var(--text-secondary)', marginBottom: 16, fontWeight: 600 }}>History</h3>
          </div>
          <div className="activity-list" style={{ flex: 1, overflowY: 'auto', padding: '0 8px', paddingBottom: 20 }}>
            {isLoadingActivity && transactions.length === 0 ? (
              <div style={{ textAlign: 'center', color: 'var(--text-secondary)', padding: '24px 0' }}>
                Loading activity...
              </div>
            ) : transactions.length > 0 ? (
              transactions.map((tx: any, idx: number) => {
                const isSender = tx.from.toLowerCase() === wallet?.address.toLowerCase();
                const amount = tx.value ? ((tx.value * 1e-9).toLocaleString('en-US', { maximumFractionDigits: 4 })) : '0'

                return (
                  <div className="list-item" key={idx}>
                    <div className="list-item-left">
                      <div className="list-item-icon" style={{
                        background: isSender ? 'rgba(255, 77, 79, 0.1)' : 'rgba(0, 229, 255, 0.1)',
                        color: isSender ? '#ff4d4f' : 'var(--cyan)'
                      }}>
                        {isSender ? <ArrowDownToLine size={18} style={{ transform: 'rotate(180deg)' }} /> : <ArrowDownToLine size={18} />}
                      </div>
                      <div>
                        <div className="list-item-title">{isSender ? 'Send' : 'Receive'}</div>
                        <div className="list-item-subtitle">{tx.kind}</div>
                      </div>
                    </div>
                    <div className="list-item-right">
                      <div className="list-item-title" style={{ color: isSender ? 'var(--text-primary)' : 'var(--cyan)' }}>
                        {isSender ? '-' : '+'}{amount} THDR
                      </div>
                      <div className="list-item-subtitle" style={{ opacity: 0.7 }}>
                        <a
                          href={`http://localhost:5173/thunderscan/${network.id}?tx=${tx.hash}`}
                          target="_blank"
                          rel="noreferrer"
                          style={{ color: 'inherit', textDecoration: 'underline' }}
                        >
                          {tx.hash.substring(0, 6)}...{tx.hash.substring(tx.hash.length - 4)}
                        </a>
                      </div>
                    </div>
                  </div>
                )
              })
            ) : (
              <div style={{ textAlign: 'center', color: 'var(--text-secondary)', padding: '24px 0', background: 'rgba(255,255,255,0.02)', borderRadius: 16 }}>
                <p>No transactions yet.</p>
              </div>
            )}
          </div>
        </div>
      )}

      {view === 'home' && (
        <div className="view-container">
          <div className="balance-section">
            <h1 className="balance-amount">{balance} {network.symbol}</h1>
            <p className="balance-usd">$0.00</p>
            {parseFloat(stakedBalance) > 0 && (
              <div style={{ marginTop: 8, fontSize: '0.9rem', color: 'var(--cyan)' }}>
                Staked: {stakedBalance} {network.symbol}
              </div>
            )}
            <div style={{ marginTop: 10, opacity: 0.7, fontSize: 12, background: 'rgba(255,255,255,0.05)', padding: '4px 8px', borderRadius: 12, display: 'inline-block' }}>
              {displayAddress}
            </div>

            <div style={{ marginTop: 16, display: 'flex', justifyContent: 'center' }}>
              <div
                className="network-selector"
                onClick={(e) => { e.stopPropagation(); setShowNetworkDropdown(!showNetworkDropdown); setShowAccountDropdown(false); }}
              >
                <div className="network-dot"></div>
                {network.name}
                <ChevronDown size={14} style={{ marginLeft: 4 }} />

                {showNetworkDropdown && (
                  <div className="network-dropdown" style={{ top: '100%', bottom: 'auto' }}>
                    {NETWORKS.map(net => (
                      <div
                        key={net.id}
                        className="network-item"
                        onClick={() => { setNetwork(net); setShowNetworkDropdown(false) }}
                      >
                        <div className="network-dot" style={{ backgroundColor: net.id === 'mainnet' ? 'var(--primary-color)' : 'var(--cyan)' }}></div>
                        {net.name}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Actions */}
          <section className="action-buttons">
            <button className="action-btn" onClick={() => setView('send')}>
              <div className="action-icon">
                <SendIcon size={20} />
              </div>
              <span>Send</span>
            </button>
            <button className="action-btn" onClick={() => setView('receive')}>
              <div className="action-icon">
                <ArrowDownToLine size={20} />
              </div>
              <span>Receive</span>
            </button>
            <button className="action-btn" onClick={() => setView('stake')}>
              <div className="action-icon">
                <Coins size={20} />
              </div>
              <span>Pools</span>
            </button>
          </section>

          {/* Tabs */}
          <div className="tabs">
            <div
              className={`tab ${activeTab === 'tokens' ? 'active' : ''}`}
              onClick={() => setActiveTab('tokens')}
            >
              Tokens
            </div>
            <div
              className={`tab ${activeTab === 'nfts' ? 'active' : ''}`}
              onClick={() => setActiveTab('nfts')}
            >
              NFTs
            </div>
            <div
              className={`tab ${activeTab === 'activity' ? 'active' : ''}`}
              onClick={() => setActiveTab('activity')}
            >
              Activity
            </div>
          </div>

          {/* Tab Content */}
          <div className="tab-content">
            {activeTab === 'tokens' && (
              <div className="token-list">
                <div
                  className="list-item"
                  style={{ cursor: 'pointer' }}
                  onClick={() => setView('token_details')}
                >
                  <div className="list-item-left">
                    <div className="list-item-icon" style={{ background: 'rgba(139, 92, 246, 0.2)', border: '1px solid rgba(139, 92, 246, 0.3)' }}>
                      <img src="/logo.png" alt="Thunder" style={{ width: 24, height: 24 }} />
                    </div>
                    <div>
                      <div className="list-item-title">Thunder</div>
                      <div className="list-item-subtitle">{balance} THDR</div>
                    </div>
                  </div>
                  <div className="list-item-right">
                    <div className="list-item-title">$0.00</div>
                    <div className="list-item-subtitle" style={{ color: '#ff4d4f' }}>0.00%</div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'nfts' && (
              <div style={{ textAlign: 'center', color: 'var(--text-secondary)', padding: '24px 0' }}>
                <LayoutGrid size={32} style={{ opacity: 0.5, marginBottom: 8 }} />
                <p>No NFTs found.</p>
                <button
                  className="btn-outline"
                  onClick={() => setView('mint')}
                  style={{ marginTop: 12, padding: '6px 12px', borderRadius: 8, background: 'transparent', border: '1px solid var(--border)', color: 'white', cursor: 'pointer' }}>
                  Mint NFT
                </button>
              </div>
            )}

            {activeTab === 'activity' && (
              <div className="activity-list">
                {isLoadingActivity ? (
                  <div style={{ textAlign: 'center', color: 'var(--text-secondary)', padding: '24px 0' }}>
                    Loading activity...
                  </div>
                ) : transactions.length > 0 ? (
                  transactions.map((tx: any, idx: number) => {
                    const isSender = tx.from.toLowerCase() === wallet?.address.toLowerCase();
                    const amount = tx.value ? ((tx.value * 1e-9).toLocaleString('en-US', { maximumFractionDigits: 4 })) : '0'

                    return (
                      <div className="list-item" key={idx}>
                        <div className="list-item-left">
                          <div className="list-item-icon" style={{
                            background: isSender ? 'rgba(255, 77, 79, 0.1)' : 'rgba(0, 229, 255, 0.1)',
                            color: isSender ? '#ff4d4f' : 'var(--cyan)'
                          }}>
                            {isSender ? <ArrowDownToLine size={18} style={{ transform: 'rotate(180deg)' }} /> : <ArrowDownToLine size={18} />}
                          </div>
                          <div>
                            <div className="list-item-title">{isSender ? 'Send' : 'Receive'}</div>
                            <div className="list-item-subtitle">{tx.kind}</div>
                          </div>
                        </div>
                        <div className="list-item-right">
                          <div className="list-item-title" style={{ color: isSender ? 'var(--text-primary)' : 'var(--cyan)' }}>
                            {isSender ? '-' : '+'}{amount} THDR
                          </div>
                          <div className="list-item-subtitle" style={{ opacity: 0.7 }}>
                            <a
                              href={`http://localhost:5173/thunderscan/${network.id}?tx=${tx.hash}`}
                              target="_blank"
                              rel="noreferrer"
                              style={{ color: 'inherit', textDecoration: 'underline' }}
                            >
                              {tx.hash.substring(0, 6)}...{tx.hash.substring(tx.hash.length - 4)}
                            </a>
                          </div>
                        </div>
                      </div>
                    )
                  })
                ) : (
                  <div style={{ textAlign: 'center', color: 'var(--text-secondary)', padding: '24px 0' }}>
                    <p>No recent activity.</p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      
      {renderModal()}
    </div>
  )
}

export default App
