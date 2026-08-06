import { useEffect, useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { motion } from 'framer-motion'
import '../App.css'



/* ══════════════════════════════════════════════════════════════════
   ThunderScan — Block Explorer Pages
   ══════════════════════════════════════════════════════════════════ */

// Helper: Network Toggle component used on both Mainnet & Testnet
function ScanNetworkToggle() {
  const location = useLocation()
  const isTestnet = location.pathname.includes('testnet')
  return (
    <div className="scan-network-toggle">
      <Link to="/thunderscan/mainnet" className={`scan-network-btn ${!isTestnet ? 'active' : ''}`}>
        🌐 Mainnet
      </Link>
      <Link to="/thunderscan/testnet" className={`scan-network-btn ${isTestnet ? 'active' : ''}`}>
        🧪 Testnet
      </Link>
    </div>
  )
}

// Helper: format hex address for display
function fmtAddr(addr: string) {
  if (!addr) return '0x0000...0000';
  if (addr.length < 10) return addr;
  return addr.slice(0, 6) + '...' + addr.slice(-4)
}

// Helper: format hex hash for display
function fmtHash(hash: string) {
  if (!hash) return '0x00000000...0000';
  if (hash.length < 14) return hash;
  return hash.slice(0, 10) + '...' + hash.slice(-4)
}

/* ── ThunderScan Testnet (Full Explorer) ───────────────────────── */
function ThunderScanTestnet() {
  const [search, setSearch] = useState('')
  const [viewTxHash, setViewTxHash] = useState<string | null>(null);
  const [viewBlockHeight, setViewBlockHeight] = useState<number | null>(null);
  const [viewAddress, setViewAddress] = useState<string | null>(null);
  const [viewAll, setViewAll] = useState<'blocks' | 'txns' | null>(null);

  const [viewTxDetails, setViewTxDetails] = useState<any>(null);
  const [viewBlockDetails, setViewBlockDetails] = useState<any>(null)

  const [viewAccountDetails, setViewAccountDetails] = useState<any>(null)
  // ── Network States ──
  const [blockHeight, setBlockHeight] = useState<number>(0)
  const [blocks, setBlocks] = useState<any[]>([])
  const [txns, setTxns] = useState<any[]>([])
  const [mempoolTxns, setMempoolTxns] = useState<any[]>([])
  const [validators, setValidators] = useState<any[]>([])

  const [totalStake, setTotalStake] = useState<number>(0)
  const [activeCount, setActiveCount] = useState<number>(0)
  const [supermajority, setSupermajority] = useState<number>(0)
  const [totalValidators, setTotalValidators] = useState<number>(0)
  const [searchError, setSearchError] = useState<string | null>(null)

  // Mirrors: thunder_rpc::server — JSON-RPC API methods
  const rpcMethods = [
    { method: 'thunder_chainId', title: 'Get Chain ID', desc: 'Returns the current chain identifier for the connected network.' },
    { method: 'thunder_blockNumber', title: 'Get Block Number', desc: 'Returns the latest block height from the WorldState.' },
    { method: 'thunder_getBlock', title: 'Get Block by Height', desc: 'Returns block header, transactions, and validator info for a given height.' },
    { method: 'thunder_getBalance', title: 'Get Account Balance', desc: 'Queries the WorldState for an account balance by address.' },
    { method: 'thunder_sendTransaction', title: 'Send Transaction', desc: 'Submit a signed transaction to the Mempool for inclusion.' },
    { method: 'thunder_getValidators', title: 'Get Validators', desc: 'Returns the active ValidatorSet with stake and status info.' },
    { method: 'thunder_compileContract', title: 'Compile Contract', desc: 'Compiles ThunderScript source code to bytecode via ThunderLang.' },
    { method: 'thunder_bridgeMint', title: 'Bridge Mint', desc: 'Queue a cross-chain mint request via the Thunder Relayer.' },
  ]

  useEffect(() => {
    let active = true;

    const loadRealtimeData = async () => {
      try {
        if (!active) return;
        const [statsRes, blockRes, valRes, mempoolRes] = await Promise.all([
          fetch('http://127.0.0.1:5050/api/stats').then(res => res.json()),
          fetch('http://127.0.0.1:5050/api/blocks/latest?limit=50').then(res => res.json()),
          fetch('http://127.0.0.1:5050/api/validators').then(res => res.json()),
          fetch('http://127.0.0.1:5050/api/mempool').then(res => res.json())
        ]);

        if (active) {
          setBlockHeight(statsRes.blockHeight || 0);
          setTotalStake(statsRes.totalStaked || 0);
          setActiveCount(statsRes.activeValidators || 0);
          setTotalValidators(statsRes.activeValidators || 0);
          setSupermajority(Math.ceil((statsRes.activeValidators || 0) * 2 / 3));

          setBlocks(blockRes.blocks || []);
          setTxns(blockRes.transactions || []);
          setValidators(valRes || []);
          setMempoolTxns(mempoolRes || []);
        }
      } catch (err) {
        // Quietly fail or show empty state if backend is down
      }
    }

    // Poll the Explorer Backend every 3 seconds
    loadRealtimeData()
    const interval = setInterval(loadRealtimeData, 3000)
    return () => { active = false; clearInterval(interval); }
  }, [])

  useEffect(() => {
    if (!viewTxHash) {
      setViewTxDetails(null)
      return
    }
    const loadTx = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:5050/api/tx/${viewTxHash}`)
        if (res.ok) {
          const data = await res.json()
          setViewTxDetails(data)
        } else {
          setViewTxDetails({ error: true })
        }
      } catch (err) {
        setViewTxDetails({ error: true })
      }
    }
    loadTx()
  }, [viewTxHash])

  useEffect(() => {
    if (viewBlockHeight === null) {
      setViewBlockDetails(null)
      return
    }
    const loadBlock = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:5050/api/block/${viewBlockHeight}`)
        if (res.ok) {
          const data = await res.json()
          setViewBlockDetails(data)
        } else {
          setViewBlockDetails({ error: true })
        }
      } catch (err) {
        setViewBlockDetails({ error: true })
      }
    }
    loadBlock()
  }, [viewBlockHeight])

  useEffect(() => {
    if (!viewAddress) {
      setViewAccountDetails(null)
      return
    }
    const loadAcc = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:5050/api/account/${viewAddress}`)
        if (res.ok) {
          const data = await res.json()
          setViewAccountDetails(data)
        } else {
          setSearchError("Address not found or has no historical transactions.")
          setViewAddress(null)
        }
      } catch (err) { }
    }
    loadAcc()
  }, [viewAddress])

  // date formatter
  const fmtDate = (ts: number) => {
    if (!ts) return '';
    const d = new Date(ts * 1000);
    return d.toLocaleString('en-US', { timeZone: 'UTC', month: 'short', day: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' }) + ' +UTC';
  }

  // search dispatcher
  const handleSearch = () => {
    if (!search) return;
    const s = search.trim();
    if (s.startsWith('0x')) {
      if (s.length === 66) {
        setViewTxHash(s);
      } else if (s.length === 42) {
        setViewAddress(s);
      }
      setViewBlockHeight(null);
      setViewAll(null);
    } else if (!isNaN(Number(s)) && s.length > 0) {
      setViewBlockHeight(Number(s));
      setViewAddress(null);
      setViewTxHash(null);
      setViewAll(null);
    }
  }

  // relative time formatter
  const timeAgo = (ts: number): string => {
    if (!ts) return 'Pending';
    const seconds = Math.floor(Date.now() / 1000 - ts);
    if (seconds < 60) return seconds + ' secs ago';
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return minutes + (minutes === 1 ? ' min ago' : ' mins ago');
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return hours + (hours === 1 ? ' hr ago' : ' hrs ago');
    const days = Math.floor(hours / 24);
    if (days < 30) return days + (days === 1 ? ' day ago' : ' days ago');
    const months = Math.floor(days / 30);
    if (months < 12) return months + (months === 1 ? ' mo ago' : ' mos ago');
    const years = Math.floor(days / 365);
    return years + (years === 1 ? ' yr ago' : ' yrs ago');
  }


  return (
    <div className="scan-page">
      <div className="container">
        {/* Hero + Network Toggle */}
        <div className="scan-hero">
          <div className="scan-hero-brand">
            <img src="/logo.png" alt="ThunderScan" />
            <h1>Thunder<span className="text-gradient">Scan</span></h1>
          </div>
          <p className="text-body">Explore blocks, transactions, and validators on the Thunder Testnet.</p>
          <ScanNetworkToggle />

          {/* Search */}
          <div className="scan-search-wrap" style={{ position: 'relative' }}>
            <input
              type="text"
              className="scan-search-input"
              placeholder="Search by Address / Txn Hash / Block Height"
              value={search}
              onChange={e => { setSearch(e.target.value); setSearchError(null); }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  handleSearch();
                }
              }}
            />
            <button className="scan-search-btn" onClick={handleSearch}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg>
            </button>
            {searchError && (
              <div style={{ position: 'absolute', top: '110%', left: '0', color: '#F87171', fontSize: '0.9rem', background: 'rgba(239, 68, 68, 0.1)', padding: '8px 16px', borderRadius: '8px', border: '1px solid rgba(239, 68, 68, 0.2)' }}>
                ⚠ {searchError}
              </div>
            )}
          </div>
        </div>

        {viewAddress ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} style={{ padding: '32px' }}>
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '24px', gap: '16px' }}>
              <button
                onClick={() => setViewAddress(null)}
                style={{ background: 'transparent', padding: '8px', cursor: 'pointer', border: 'none', color: 'var(--text-secondary)' }}
              >
                ← Back
              </button>
              <h2 className="heading-md">
                {viewAccountDetails?.type === 'Smart Contract' ? '📜 ' : '👤 '}
                {viewAccountDetails?.type || 'Account'} Details
              </h2>
            </div>
            {!viewAccountDetails ? (
              <p style={{ color: 'var(--text-secondary)' }}>Loading address...</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px' }}>
                    <div style={{ width: '200px', color: 'var(--text-secondary)' }}>Address:</div>
                    <div style={{ fontFamily: 'monospace', color: 'var(--text)' }}>
                      {viewAccountDetails.address}{" "}
                      <span className="scan-badge" style={{ marginLeft: 12 }}>{viewAccountDetails.type}</span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px' }}>
                    <div style={{ width: '200px', color: 'var(--text-secondary)' }}>Balance:</div>
                    <div style={{ fontFamily: 'monospace', color: 'var(--text)', display: 'flex', alignItems: 'center' }}>
                      <img src="/logo.png" style={{ width: 16, height: 16, marginRight: 6 }} alt="THDR" />
                      {((viewAccountDetails.balance || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR
                    </div>
                  </div>
                </div>

                <div>
                  <h3 style={{ fontSize: 16, marginBottom: 16, paddingBottom: 8, borderBottom: '1px solid var(--border)' }}>
                    Transactions ({viewAccountDetails.transactions?.length || 0})
                  </h3>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {viewAccountDetails.transactions?.map((tx: any, idx: number) => (
                      <div className="scan-row" style={{ padding: '12px 16px', background: 'rgba(0,0,0,0.2)' }} key={idx}>
                        <div className="scan-row-icon">Tx</div>
                        <div className="scan-row-main" style={{ flex: 1.5 }}>
                          <div className="scan-row-title">
                            <a href="#" onClick={(e) => { e.preventDefault(); setViewAddress(null); setViewTxHash(tx.hash); }}>
                              {fmtHash(tx.hash)}
                            </a>
                          </div>
                          <div className="scan-row-sub">{timeAgo(tx.timestamp)}</div>
                        </div>
                        <div className="scan-row-main" style={{ flex: 2 }}>
                          <div className="scan-row-sub">From: <span style={{ fontFamily: 'monospace', color: tx.from === viewAccountDetails.address ? 'var(--text-secondary)' : 'var(--accent)' }}>{fmtAddr(tx.from)}</span></div>
                          <div className="scan-row-sub">To: <span style={{ fontFamily: 'monospace', color: tx.to === viewAccountDetails.address ? 'var(--text-secondary)' : 'var(--accent)' }}>{tx.to ? fmtAddr(tx.to) : 'Unknown'}</span></div>
                        </div>
                        <div className="scan-row-meta">
                          <span className={`scan-badge ${tx.kind === 'Deploy' ? 'purple' : tx.kind === 'Stake' ? 'green' : ''}`} style={{ display: 'flex', alignItems: 'center' }}>
                            {tx.kind === 'Transfer' ? (
                              <>
                                <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 4 }} alt="THDR" />
                                {`${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR`}
                              </>
                            ) : tx.kind}
                          </span>
                        </div>
                      </div>
                    ))}
                    {viewAccountDetails.transactions?.length === 0 && (
                      <p style={{ color: 'var(--text-secondary)' }}>No matching transactions found.</p>
                    )}
                  </div>
                </div>
              </div>
            )}
          </motion.div>
        ) : viewTxHash ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} style={{ padding: '32px' }}>
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '24px', gap: '16px' }}>
              <button
                onClick={() => setViewTxHash(null)}
                style={{ background: 'transparent', padding: '8px', cursor: 'pointer', border: 'none', color: 'var(--text-secondary)' }}
              >
                ← Back
              </button>
              <h2 className="heading-md">Transaction Details</h2>
            </div>

            {!viewTxDetails ? (
              <p style={{ color: 'var(--text-secondary)' }}>Loading transaction...</p>
            ) : viewTxDetails.error ? (
              <div style={{ padding: '40px 0', textAlign: 'center' }}>
                <h3 className="heading-md" style={{ color: 'var(--text-secondary)' }}>Transaction Not Found</h3>
                <p>Ensure the transaction hash is correct. It may not have been broadcasted to the network yet.</p>
              </div>
            ) : (
              <div className="glass-card" style={{ padding: '24px', display: 'flex', flexDirection: 'column', gap: '20px', borderRadius: '12px' }}>
                <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    Transaction Hash:
                  </div>
                  <div style={{ fontFamily: 'monospace', color: 'var(--text)', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    {viewTxDetails.hash}
                    <button onClick={() => navigator.clipboard.writeText(viewTxDetails.hash)} style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)', padding: '4px' }} title="Copy Tx Hash">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                    </button>
                  </div>
                </div>
                <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    Status:
                  </div>
                  <div>
                    <span className="scan-badge green" style={{ display: 'inline-flex', alignItems: 'center', gap: '4px' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                      Success
                    </span>
                  </div>
                </div>
                <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    Block:
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <div style={{ color: 'var(--accent)', cursor: 'pointer', display: 'flex', alignItems: 'center' }} onClick={() => { setViewTxHash(null); setViewBlockHeight(viewTxDetails.block_height); }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 4 }}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                      {viewTxDetails.block_height}
                    </div>
                    <span className="scan-badge" style={{ background: 'rgba(255,255,255,0.1)', color: 'var(--text)', border: '1px solid rgba(255,255,255,0.1)' }}>
                      {blockHeight > 0 ? Math.max(1, blockHeight - viewTxDetails.block_height + 1) : 1} Block Confirmations
                    </span>
                  </div>
                </div>
                <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    Timestamp:
                  </div>
                  <div style={{ color: 'var(--text)', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ opacity: 0.7 }}><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                    {viewTxDetails.timestamp ? timeAgo(viewTxDetails.timestamp) : 'Pending'}
                    <span style={{ color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', gap: '4px' }}>| {fmtDate(viewTxDetails.timestamp)}</span>
                  </div>
                </div>
                <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    From:
                  </div>
                  <div style={{ fontFamily: 'monospace', color: 'var(--accent)', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '8px' }} onClick={() => { setViewTxHash(null); setViewAddress(viewTxDetails.from); }}>
                    {viewTxDetails.from}
                    <button onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(viewTxDetails.from); }} style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)', padding: '4px' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                    </button>
                  </div>
                </div>
                <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    To:
                  </div>
                  <div style={{ fontFamily: 'monospace', color: 'var(--accent)', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '8px' }} onClick={() => { setViewTxHash(null); setViewAddress(viewTxDetails.to); }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ color: 'var(--text-secondary)' }}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                    {viewTxDetails.to}
                    <button onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(viewTxDetails.to); }} style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)', padding: '4px' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                    </button>
                  </div>
                </div>
                <div style={{ display: 'flex', paddingBottom: '8px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path></svg>
                    Value:
                  </div>
                  <div style={{ fontFamily: 'monospace', color: 'var(--text)', display: 'flex', alignItems: 'center' }}>
                    <img src="/logo.png" style={{ width: 16, height: 16, marginRight: 6 }} alt="THDR" />
                    {viewTxDetails.value ? (viewTxDetails.value * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 }) : 0} THDR
                  </div>
                </div>
                <div style={{ display: 'flex', paddingBottom: '8px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path></svg>
                    Transaction Fee:
                  </div>
                  <div style={{ fontFamily: 'monospace', color: 'var(--text)', display: 'flex', alignItems: 'center' }}>
                    {(viewTxDetails.gas_limit * viewTxDetails.gas_price * 1e-9).toFixed(11)} THDR
                  </div>
                </div>
                <div style={{ display: 'flex', paddingBottom: '8px', alignItems: 'center' }}>
                  <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                    Gas Price:
                  </div>
                  <div style={{ fontFamily: 'monospace', color: 'var(--text)', display: 'flex', alignItems: 'center' }}>
                    {viewTxDetails.gas_price} Gwei <span style={{ color: 'var(--text-secondary)', marginLeft: 8 }}>({(viewTxDetails.gas_price * 1e-9).toFixed(11)} THDR)</span>
                  </div>
                </div>
              </div>
            )}
          </motion.div>
        ) : viewBlockHeight !== null ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} style={{ padding: '32px' }}>
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '24px', gap: '16px' }}>
              <button
                onClick={() => setViewBlockHeight(null)}
                style={{ background: 'transparent', padding: '8px', cursor: 'pointer', border: 'none', color: 'var(--text-secondary)' }}
              >
                ← Back
              </button>
              <h2 className="heading-md">Block Details <span style={{ color: 'var(--text-secondary)' }}>#{viewBlockHeight}</span></h2>
            </div>

            {!viewBlockDetails ? (
              <p style={{ color: 'var(--text-secondary)' }}>Loading block...</p>
            ) : viewBlockDetails.error ? (
              <div style={{ padding: '40px 0', textAlign: 'center' }}>
                <h3 className="heading-md" style={{ color: 'var(--text-secondary)' }}>Block Not Found</h3>
                <p>The requested block height could not be located on the testnet.</p>
              </div>
            ) : (
              <>
                <div className="glass-card" style={{ padding: '24px', display: 'flex', flexDirection: 'column', gap: '20px', borderRadius: '12px' }}>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Block Height:
                    </div>
                    <div style={{ color: 'var(--text)', display: 'flex', alignItems: 'center', gap: '12px' }}>
                      {viewBlockDetails.height}
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Status:
                    </div>
                    <div>
                      <span className="scan-badge green" style={{ display: 'inline-flex', alignItems: 'center', gap: '4px' }}>
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                        Finalized (Safe)
                      </span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Timestamp:
                    </div>
                    <div style={{ color: 'var(--text)', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ opacity: 0.7 }}><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                      {viewBlockDetails.timestamp ? timeAgo(viewBlockDetails.timestamp) : 'Pending'}
                      <span style={{ color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', gap: '4px' }}>| {fmtDate(viewBlockDetails.timestamp)}</span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Proposed By:
                    </div>
                    <div style={{ fontFamily: 'monospace', color: 'var(--accent)', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '8px' }} onClick={() => { setViewBlockHeight(null); setViewAddress(viewBlockDetails.validator); }}>
                      {viewBlockDetails.validator}
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Block Hash:
                    </div>
                    <div style={{ fontFamily: 'monospace', color: 'var(--text)', display: 'flex', alignItems: 'center', gap: '8px' }}>
                      {viewBlockDetails.hash}
                      <button onClick={() => navigator.clipboard.writeText(viewBlockDetails.hash)} style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)', padding: '4px' }} title="Copy Block Hash">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                      </button>
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Size:
                    </div>
                    <div style={{ color: 'var(--text)' }}>
                      {(viewBlockDetails.size || 0).toLocaleString()} bytes
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Gas Used:
                    </div>
                    <div style={{ color: 'var(--text)' }}>
                      {(viewBlockDetails.gas_used || 0).toLocaleString()} <span style={{ color: 'var(--text-secondary)' }}>({viewBlockDetails.gas_limit ? ((viewBlockDetails.gas_used || 0) / viewBlockDetails.gas_limit * 100).toFixed(2) : '0.00'}%)</span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Gas Limit:
                    </div>
                    <div style={{ color: 'var(--text)' }}>
                      {(viewBlockDetails.gas_limit || 0).toLocaleString()}
                    </div>
                  </div>
                  <div style={{ display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.05)', paddingBottom: '16px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Base Fee Per Gas:
                    </div>
                    <div style={{ color: 'var(--text)' }}>
                      {((viewBlockDetails.base_fee || 0) * 1e-9).toFixed(10)} THDR <span style={{ color: 'var(--text-secondary)' }}>({viewBlockDetails.base_fee || 0} Gwei)</span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', paddingBottom: '8px', alignItems: 'center' }}>
                    <div style={{ width: '250px', color: 'var(--text-secondary)', display: 'flex', alignItems: 'center' }}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, opacity: 0.5 }}><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                      Block Reward:
                    </div>
                    <div style={{ color: 'var(--text)', display: 'flex', alignItems: 'center' }}>
                      <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 6 }} alt="THDR" />
                      {(viewBlockDetails.reward || 0).toFixed(6)} THDR <span style={{ color: 'var(--text-secondary)', marginLeft: 8 }}>(Base + Fees)</span>
                    </div>
                  </div>
                </div>

                {/* Etherscan-Styled Embedded Transactions Table */}
                <div style={{ marginTop: '24px' }}>
                  <h3 style={{ fontSize: 16, marginBottom: 16, paddingBottom: 12, borderBottom: '1px solid rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', fontWeight: 600 }}>
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: 8, color: 'var(--text-secondary)' }}><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                    A total of {viewBlockDetails.txn_count || (viewBlockDetails.transactions ? viewBlockDetails.transactions.length : 0)} transactions found
                  </h3>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {viewBlockDetails.transactions?.map((tx: any, idx: number) => (
                      <div className="scan-row" style={{ padding: '16px', background: 'rgba(255,255,255,0.02)', border: '1px solid rgba(255,255,255,0.05)', borderRadius: '12px', display: 'flex', alignItems: 'center' }} key={idx}>
                        <div className="scan-row-icon" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--glass-border)' }}>Tx</div>
                        <div className="scan-row-main" style={{ flex: 1.5 }}>
                          <div className="scan-row-title">
                            <a href="#" onClick={(e) => { e.preventDefault(); setViewBlockHeight(null); setViewTxHash(tx.hash); }}>
                              {fmtHash(tx.hash)}
                            </a>
                          </div>
                          <div className="scan-row-sub">{timeAgo(viewBlockDetails.timestamp)}</div>
                        </div>
                        <div className="scan-row-main" style={{ flex: 2 }}>
                          <div className="scan-row-sub">From: <a href="#" onClick={(e) => { e.preventDefault(); setViewBlockHeight(null); setViewAddress(tx.from); }}>{fmtAddr(tx.from)}</a></div>
                          <div className="scan-row-sub">To: {tx.to ? <a href="#" onClick={(e) => { e.preventDefault(); setViewBlockHeight(null); setViewAddress(tx.to); }}>{tx.to?.startsWith('0x') ? fmtAddr(tx.to) : tx.to}</a> : 'Unknown'}</div>
                        </div>
                        <div className="scan-row-meta">
                          <span className={`scan-badge ${tx.kind === 'Deploy' ? 'purple' : tx.kind === 'Stake' ? 'green' : ''}`} style={{ display: 'flex', alignItems: 'center' }}>
                            {tx.kind === 'Transfer' ? (
                              <>
                                <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 4 }} alt="THDR" />
                                {`${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR`}
                              </>
                            ) : tx.kind}
                          </span>
                        </div>
                      </div>
                    ))}
                    {(!viewBlockDetails.transactions || viewBlockDetails.transactions.length === 0) && (
                      <p style={{ color: 'var(--text-secondary)', padding: '16px', textAlign: 'center', background: 'rgba(255,255,255,0.01)', borderRadius: '8px' }}>No transactions found for this block.</p>
                    )}
                  </div>
                </div>
              </>
            )}
          </motion.div>
        ) : viewAll === 'blocks' ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} style={{ padding: '32px' }}>
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '24px', gap: '16px' }}>
              <button onClick={() => setViewAll(null)} style={{ background: 'transparent', padding: '8px', cursor: 'pointer', border: 'none', color: 'var(--text-secondary)' }}>← Back</button>
              <h2 className="heading-md">All Blocks</h2>
            </div>
            {blocks.map(block => (
              <div className="scan-row" key={block.height}>
                <div className="scan-row-icon">Bk</div>
                <div className="scan-row-main">
                  <div className="scan-row-title"><a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); setViewBlockHeight(block.height); }}>{block.height}</a></div>
                  <div className="scan-row-sub">{timeAgo(block.timestamp)}</div>
                </div>
                <div className="scan-row-main">
                  <div className="scan-row-sub">Validator <a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); setViewAddress(block.validator); }}>{fmtAddr(block.validator)}</a></div>
                  <div className="scan-row-sub">{block.txn_count || 0} txns</div>
                </div>
                <div className="scan-row-meta">
                  <span className="scan-badge" style={{ display: 'flex', alignItems: 'center' }}>
                    <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 4 }} alt="THDR" />
                    {(block.reward || 0).toLocaleString('en-US', { maximumFractionDigits: 6 })} THDR
                  </span>
                </div>
              </div>
            ))}
          </motion.div>
        ) : viewAll === 'txns' ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} style={{ padding: '32px' }}>
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '24px', gap: '16px' }}>
              <button onClick={() => setViewAll(null)} style={{ background: 'transparent', padding: '8px', cursor: 'pointer', border: 'none', color: 'var(--text-secondary)' }}>← Back</button>
              <h2 className="heading-md">All Transactions</h2>
            </div>
            {txns.map((tx, i) => (
              <div className="scan-row" key={i}>
                <div className="scan-row-icon">Tx</div>
                <div className="scan-row-main">
                  <div className="scan-row-title">
                    <a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); setViewTxHash(tx.hash) }}>{fmtHash(tx.hash)}</a>
                  </div>
                  <div className="scan-row-sub">{timeAgo(tx.timestamp)}</div>
                </div>
                <div className="scan-row-main" style={{ flex: 1.2 }}>
                  <div className="scan-row-sub">From <a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); setViewAddress(tx.from); }}>{fmtAddr(tx.from)}</a></div>
                  <div className="scan-row-sub">To <a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); setViewAddress(tx.to); }}>{tx.to?.startsWith('0x') ? fmtAddr(tx.to) : tx.to || 'Unknown'}</a></div>
                </div>
                <div className="scan-row-meta">
                  <span className={`scan-badge ${tx.kind === 'Deploy' ? 'purple' : tx.kind === 'Stake' ? 'green' : ''}`} style={{ display: 'flex', alignItems: 'center' }}>
                    {tx.kind === 'Transfer' ? (
                      <>
                        <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 4 }} alt="THDR" />
                        {`${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR`}
                      </>
                    ) : tx.kind}
                  </span>
                </div>
              </div>
            ))}
          </motion.div>
        ) : (
          <>
            {/* Stats Grid — mirrors WorldState + ValidatorSet */}
            <motion.div className="scan-stats-grid" initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5 }}>
              <div className="scan-stat-card">
                <div className="scan-stat-label">Thunder Price</div>
                <div className="scan-stat-value">$1.24</div>
                <div className="scan-stat-badge">+5.2%</div>
              </div>
              <div className="scan-stat-card">
                <div className="scan-stat-label">Block Height</div>
                <div className="scan-stat-value">{blockHeight.toLocaleString()}</div>
                <div className="scan-stat-badge">~3.0s / block</div>
                </div>
                <div className="scan-stat-card">
                  <div className="scan-stat-label">Network TPS</div>
                  <div className="scan-stat-value" style={{ display: "flex", alignItems: "center", gap: 6 }}>
                    {blocks.length > 0 ? ((blocks[0].txn_count || 0) / 3.0).toFixed(1) : "0.0"} <span style={{ fontSize: 18, color: "var(--text-secondary)" }}>Tx/s</span>
                  </div>
                  <div className="scan-stat-badge">aBFT Velocity</div>
                </div>
                <div className="scan-stat-card">
                  <div className="scan-stat-label">Active Validators</div>
                  <div className="scan-stat-value">{activeCount} / {totalValidators}</div>
                  <div className="scan-stat-badge">aBFT DAG Consensus</div>
                </div>
            </motion.div>

            {/* Latest Blocks & Transactions — mirrors Block & Transaction structs */}
            <motion.div className="scan-data-grid" initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: 0.1 }}>
              {/* Blocks Panel */}
              <div className="scan-panel" style={{ display: 'flex', flexDirection: 'column' }}>
                <div className="scan-panel-header">
                  <h3>📦 Latest Blocks</h3>
                </div>
                {blocks.slice(0, 6).map(block => (
                  <div className="scan-row" key={block.height}>
                    <div className="scan-row-icon">Bk</div>
                    <div className="scan-row-main">
                      <div className="scan-row-title"><a href="#" onClick={(e) => { e.preventDefault(); setViewBlockHeight(block.height); }}>{block.height}</a></div>
                      <div className="scan-row-sub">{timeAgo(block.timestamp)}</div>
                    </div>
                    <div className="scan-row-main">
                      <div className="scan-row-sub">Validator <a href="#" onClick={(e) => { e.preventDefault(); setViewAddress(block.validator); }}>{fmtAddr(block.validator)}</a></div>
                      <div className="scan-row-sub">{block.txn_count || 0} txns</div>
                    </div>
                    <div className="scan-row-meta">
                      <span className="scan-badge" style={{ display: 'flex', alignItems: 'center' }}>
                        <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 4 }} alt="THDR" />
                        {(block.reward || 0).toLocaleString('en-US', { maximumFractionDigits: 6 })} THDR
                      </span>
                    </div>
                  </div>
                ))}
                {blocks.length === 0 && <p style={{ color: 'var(--text-secondary)', padding: '16px' }}>No blocks processed yet.</p>}
                <a href="#" className="scan-view-all" style={{ marginTop: 'auto' }} onClick={(e) => { e.preventDefault(); setViewAll('blocks'); }}>View all blocks →</a>
              </div>

              {/* Mempool Panel */}
              {mempoolTxns.length > 0 && (
                <div className="scan-panel" style={{ gridColumn: '1 / -1', border: '1px solid rgba(251, 191, 36, 0.4)', borderRadius: 16 }}>
                  <div className="scan-panel-header" style={{ borderBottom: '1px solid rgba(251, 191, 36, 0.1)', paddingBottom: 16, marginBottom: 8, display: 'flex', justifyContent: 'space-between' }}>
                    <h3 style={{ color: '#FCD34D', display: 'flex', alignItems: 'center', gap: 10, margin: 0 }}>
                      <span style={{ fontSize: '1.2rem' }}>⏳</span> Pending Transactions (Mempool)
                    </h3>
                    <span className="scan-badge" style={{ backgroundColor: 'rgba(251, 191, 36, 0.15)', color: '#FCD34D', border: '1px solid rgba(251, 191, 36, 0.3)' }}>{mempoolTxns.length} pending</span>
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    {mempoolTxns.map((tx: any, i) => (
                      <div className="scan-row" key={'mem' + i} style={{ background: 'linear-gradient(90deg, rgba(251, 191, 36, 0.08) 0%, rgba(251, 191, 36, 0.02) 100%)', borderRadius: 12, padding: '16px 20px', display: 'flex', alignItems: 'center' }}>
                        <div className="scan-row-icon" style={{ background: 'rgba(251, 191, 36, 0.15)', color: '#FCD34D', border: '1px solid rgba(251, 191, 36, 0.3)' }}>⟳</div>
                        <div className="scan-row-main" style={{ minWidth: 200, flex: 1 }}>
                          <div className="scan-row-title">
                            <a href="#" onClick={(e) => { e.preventDefault(); setViewTxHash(tx.hash) }} style={{ color: '#60A5FA' }}>{fmtHash(tx.hash)}</a>
                          </div>
                          <div className="scan-row-sub" style={{ color: '#FCD34D', marginTop: 4, display: 'flex', alignItems: 'center', gap: 6 }}>
                            <span style={{ display: 'inline-block', width: 6, height: 6, borderRadius: '50%', background: '#FBBF24', boxShadow: '0 0 8px #FBBF24' }}></span>
                            Pending Validation
                          </div>
                        </div>
                        <div className="scan-row-main" style={{ flex: 1.5 }}>
                          <div className="scan-row-sub" style={{ display: 'flex', gap: 8, marginBottom: 4 }}>
                            <span style={{ color: 'var(--text-secondary)', width: 40 }}>From</span>
                            <a href="#" onClick={(e) => { e.preventDefault(); setViewAddress(tx.from); }}>{fmtAddr(tx.from)}</a>
                          </div>
                          <div className="scan-row-sub" style={{ display: 'flex', gap: 8 }}>
                            <span style={{ color: 'var(--text-secondary)', width: 40 }}>To</span>
                            <a href="#" onClick={(e) => { e.preventDefault(); setViewAddress(tx.to); }}>{tx.to?.startsWith('0x') ? fmtAddr(tx.to) : tx.to || 'Unknown'}</a>
                          </div>
                        </div>
                        <div className="scan-row-meta" style={{ flex: 0.5, textAlign: 'right' }}>
                          <span className="scan-badge" style={{ backgroundColor: 'rgba(52, 211, 153, 0.1)', color: '#34D399', border: '1px solid rgba(52, 211, 153, 0.2)', padding: '6px 12px' }}>
                            {tx.kind === 'Transfer' ? (
                              <>{`${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR`}</>
                            ) : tx.kind}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Transactions Panel */}
              <div className="scan-panel" style={{ display: 'flex', flexDirection: 'column' }}>
                <div className="scan-panel-header">
                  <h3>⚡ Latest Transactions</h3>
                </div>
                {txns.slice(0, 6).map((tx, i) => (
                  <div className="scan-row" key={i}>
                    <div className="scan-row-icon">Tx</div>
                    <div className="scan-row-main">
                      <div className="scan-row-title">
                        <a href="#" onClick={(e) => { e.preventDefault(); setViewTxHash(tx.hash) }}>{fmtHash(tx.hash)}</a>
                      </div>
                      <div className="scan-row-sub">{timeAgo(tx.timestamp)}</div>
                    </div>
                    <div className="scan-row-main" style={{ flex: 1.2 }}>
                      <div className="scan-row-sub">From <a href="#" onClick={(e) => { e.preventDefault(); setViewAddress(tx.from); }}>{fmtAddr(tx.from)}</a></div>
                      <div className="scan-row-sub">To <a href="#" onClick={(e) => { e.preventDefault(); setViewAddress(tx.to); }}>{tx.to?.startsWith('0x') ? fmtAddr(tx.to) : tx.to || 'Unknown'}</a></div>
                    </div>
                    <div className="scan-row-meta">
                      <span className={`scan-badge ${tx.kind === 'Deploy' ? 'purple' : tx.kind === 'Stake' ? 'green' : ''}`} style={{ display: 'flex', alignItems: 'center' }}>
                        {tx.kind === 'Transfer' ? (
                          <>
                            <img src="/logo.png" style={{ width: 14, height: 14, marginRight: 4 }} alt="THDR" />
                            {`${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR`}
                          </>
                        ) : tx.kind}
                      </span>
                    </div>
                  </div>
                ))}
                {txns.length === 0 && <p style={{ color: 'var(--text-secondary)', padding: '16px' }}>No finalized transactions yet.</p>}
                <a href="#" className="scan-view-all" style={{ marginTop: 'auto' }} onClick={(e) => { e.preventDefault(); setViewAll('txns'); }}>View all transactions →</a>
              </div>
            </motion.div>

            {/* Validators Section & API Section */}
            <motion.div className="scan-validators-section" initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: 0.2 }}>
              <h2 className="heading-lg" style={{ marginBottom: 24 }}>Active <span className="text-gradient">Validators</span></h2>
              <div className="scan-validators-grid">
                {validators.map((v, i) => (
                  <div className="scan-validator-card" key={i}>
                    <div className="scan-validator-avatar">{v.name.charAt(0)}</div>
                    <div className="scan-validator-info">
                      <div className="scan-validator-name">
                        <span className={`scan-status-dot ${v.is_active ? 'active' : 'inactive'}`}></span>
                        {v.name}
                      </div>
                      <div className="scan-validator-addr" style={{ cursor: 'pointer' }} onClick={() => setViewAddress(v.address)}>
                        {v.address}
                      </div>
                    </div>
                    <div className="scan-validator-stake">
                      <div className="scan-validator-stake-value">{(v.stake * 1e-9).toLocaleString()}</div>
                      <div className="scan-validator-stake-label">THDR Staked</div>
                    </div>
                  </div>
                ))}
                {validators.length === 0 && <p style={{ color: 'var(--text-secondary)' }}>No active validators discovered.</p>}
              </div>
            </motion.div>
            <motion.div className="scan-api-section" initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, delay: 0.3 }}>
              <h2 className="heading-lg" style={{ marginBottom: 8 }}>Thunder <span className="text-gradient">RPC API</span></h2>
              <p className="text-body" style={{ marginBottom: 32 }}>Connect your dApp to the Thunder Testnet using our JSON-RPC 2.0 interface.</p>
              <div className="scan-api-grid">
                {rpcMethods.map((api, i) => (
                  <div className="scan-api-card" key={i}>
                    <div className="scan-api-method">{api.method}</div>
                    <div className="scan-api-title">{api.title}</div>
                    <div className="scan-api-desc">{api.desc}</div>
                  </div>
                ))}
              </div>
            </motion.div>
          </>
        )}
      </div>
    </div >
  )
}

/* ── ThunderScan Mainnet (Coming Soon) ─────────────────────────── */
function ThunderScanMainnet() {
  return (
    <div className="scan-page">
      <div className="container">
        <div className="scan-hero">
          <div className="scan-hero-brand">
            <img src="/logo.png" alt="ThunderScan" />
            <h1>Thunder<span className="text-gradient">Scan</span></h1>
          </div>
          <p className="text-body">Select a network to explore.</p>
          <ScanNetworkToggle />
        </div>

        <div className="scan-coming-soon">
          <motion.div
            className="glass-card scan-coming-soon-card"
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ type: 'spring', damping: 15 }}
          >
            <span className="scan-coming-soon-icon">⚡</span>
            <h2>Mainnet <span className="text-gradient">Coming Soon</span></h2>
            <p>
              The Thunder Mainnet is currently under development. Our team is finalizing the genesis validator set, security audits, and cross-chain bridge infrastructure.
              In the meantime, explore the <Link to="/thunderscan/testnet" className="text-gradient" style={{ fontWeight: 600 }}>Testnet Explorer</Link> to interact with live data.
            </p>
            <Link to="/thunderscan/testnet" className="btn btn-primary">🧪 Explore Testnet</Link>
          </motion.div>
        </div>
      </div>
    </div>
  )
}


/* ── ThunderScan Layout ─────────────────────────────────────────── */
function ScanNavbar() {
  const [scrolled, setScrolled] = useState(false)
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20)
    window.addEventListener('scroll', onScroll)
    return () => window.removeEventListener('scroll', onScroll)
  }, [])

  return (
    <nav className={`navbar ${scrolled ? 'scrolled' : ''}`}>
      <div className="container">
        <Link to="/thunderscan/testnet" className="nav-logo">
          <img src="/logo.png" alt="ThunderScan" />
          <span>Thunder<span className="text-gradient">Scan</span></span>
        </Link>
        <div className="nav-links">
          <Link to="/thunderscan/mainnet">Mainnet</Link>
          <Link to="/thunderscan/testnet">Testnet Edge</Link>
        </div>
        <div className="nav-cta">
          <Link to="/" className="btn btn-outline" style={{ padding: '10px 20px', fontSize: '0.85rem' }}>← Back to Website</Link>
        </div>
      </div>
    </nav>
  )
}

function ScanFooter() {
  return (
    <footer className="footer" style={{ background: 'var(--bg-primary)', borderTop: '1px solid var(--glass-border)', padding: '40px 0 20px' }}>
      <div className="container" style={{ textAlign: 'center', color: 'var(--text-secondary)', fontSize: '0.85rem' }}>
        © 2026 ThunderScan Explorer. Powered by Thunder Core-Engine.
      </div>
    </footer>
  )
}

export { ThunderScanTestnet, ThunderScanMainnet, ScanNavbar, ScanFooter };
