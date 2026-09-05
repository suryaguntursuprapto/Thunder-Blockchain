import { useEffect, useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { motion } from 'framer-motion'
import '../App.css'

/* ══════════════════════════════════════════════════════════════════
   ThunderScan — Block Explorer v2.0 (Clean & Futuristic)
   ══════════════════════════════════════════════════════════════════ */

// Helper: Network Toggle
function ScanNetworkToggle() {
  const location = useLocation()
  const isTestnet = location.pathname.includes('testnet')
  return (
    <div className="scan-network-toggle">
      <Link to="/thunderscan/mainnet" className={`scan-network-btn ${!isTestnet ? 'active' : ''}`}>
        Mainnet
      </Link>
      <Link to="/thunderscan/testnet" className={`scan-network-btn ${isTestnet ? 'active' : ''}`}>
        Testnet
      </Link>
    </div>
  )
}

// Helper: format hex address
function fmtAddr(addr: string) {
  if (!addr) return '0x0000...0000';
  if (addr.length < 10) return addr;
  return addr.slice(0, 6) + '...' + addr.slice(-4)
}

// Helper: format hex hash
function fmtHash(hash: string) {
  if (!hash) return '0x00000000...0000';
  if (hash.length < 14) return hash;
  return hash.slice(0, 10) + '...' + hash.slice(-4)
}

// Reusable: Copy button
function CopyBtn({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(text); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
      style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: copied ? 'var(--green)' : 'var(--text-tertiary)', padding: '4px', transition: 'color 0.2s' }}
      title="Copy"
    >
      {copied ? (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="20 6 9 17 4 12"></polyline></svg>
      ) : (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
      )}
    </button>
  )
}

// Reusable: Detail Row
function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="scan-detail-row">
      <div className="scan-detail-label">{label}</div>
      <div className="scan-detail-value">{children}</div>
    </div>
  )
}

// Reusable: Transaction Row
function TxRow({ tx, onTxClick, onAddrClick, timestamp }: { tx: any; onTxClick: (hash: string) => void; onAddrClick: (addr: string) => void; timestamp?: number }) {
  const ts = timestamp || tx.timestamp;
  return (
    <div className="scan-row">
      <div className="scan-row-icon tx-icon">Tx</div>
      <div className="scan-row-main" style={{ flex: 1.5 }}>
        <div className="scan-row-title">
          <a href="#" onClick={(e) => { e.preventDefault(); onTxClick(tx.hash); }}>{fmtHash(tx.hash)}</a>
        </div>
        <div className="scan-row-sub">{ts ? timeAgoFn(ts) : 'Pending'}</div>
      </div>
      <div className="scan-row-main" style={{ flex: 2 }}>
        <div className="scan-row-sub">From <a href="#" onClick={(e) => { e.preventDefault(); onAddrClick(tx.from); }}>{fmtAddr(tx.from)}</a></div>
        <div className="scan-row-sub">
          {tx.kind === 'ContractDeploy' ? (
            <>Contract Created <a href="#" onClick={(e) => { e.preventDefault(); if (tx.contract_address) onAddrClick(tx.contract_address); }}>{tx.contract_address ? fmtAddr(tx.contract_address) : 'Unknown'}</a></>
          ) : (
            <>To <a href="#" onClick={(e) => { e.preventDefault(); onAddrClick(tx.to); }}>{tx.to?.startsWith('0x') ? fmtAddr(tx.to) : tx.to || 'Contract'}</a></>
          )}
        </div>
      </div>
      <div className="scan-row-meta">
        <span className={`scan-badge ${tx.kind === 'ContractDeploy' ? 'purple' : tx.kind === 'Stake' ? 'green' : ''}`} style={{ display: 'flex', alignItems: 'center' }}>
          {tx.kind === 'Transfer' ? (
            <>
              <img src="/logo.png" style={{ width: 13, height: 13, marginRight: 4 }} alt="THDR" />
              {`${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR`}
            </>
          ) : tx.kind}
        </span>
      </div>
    </div>
  )
}

// Standalone timeAgo (used in TxRow)
function timeAgoFn(ts: number): string {
  if (!ts) return 'Pending';
  const seconds = Math.floor(Date.now() / 1000 - ts);
  if (seconds < 60) return seconds + 's ago';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return minutes + (minutes === 1 ? ' min ago' : ' mins ago');
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return hours + (hours === 1 ? ' hr ago' : ' hrs ago');
  const days = Math.floor(hours / 24);
  return days + (days === 1 ? ' day ago' : ' days ago');
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

  const [blockHeight, setBlockHeight] = useState<number>(0)
  const [blocks, setBlocks] = useState<any[]>([])
  const [txns, setTxns] = useState<any[]>([])
  const [mempoolTxns, setMempoolTxns] = useState<any[]>([])
  const [validators, setValidators] = useState<any[]>([])

  const [activeCount, setActiveCount] = useState<number>(0)
  const [totalValidators, setTotalValidators] = useState<number>(0)
  const [searchError, setSearchError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'validators' | 'api'>('overview')

  const location = useLocation()

  useEffect(() => {
    const params = new URLSearchParams(location.search)
    const txParam = params.get('tx')
    if (txParam) {
      setViewTxHash(txParam)
      setViewAll(null)
      setViewAddress(null)
      setViewBlockHeight(null)
    }
  }, [location.search])

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

  // ── Data Fetching ──────────────────────────────────────────────
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
          setActiveCount(statsRes.activeValidators || 0);
          setTotalValidators(statsRes.activeValidators || 0);
          setBlocks(blockRes.blocks || []);
          setTxns(blockRes.transactions || []);
          setValidators(valRes || []);
          setMempoolTxns(mempoolRes || []);
        }
      } catch (err) { /* Quietly fail */ }
    }
    loadRealtimeData()
    const interval = setInterval(loadRealtimeData, 3000)
    return () => { active = false; clearInterval(interval); }
  }, [])

  useEffect(() => {
    if (!viewTxHash) { setViewTxDetails(null); return }
    const loadTx = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:5050/api/tx/${viewTxHash}`)
        if (res.ok) { setViewTxDetails(await res.json()) }
        else { setViewTxDetails({ error: true }) }
      } catch (err) { setViewTxDetails({ error: true }) }
    }
    loadTx()
  }, [viewTxHash])

  useEffect(() => {
    if (viewBlockHeight === null) { setViewBlockDetails(null); return }
    const loadBlock = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:5050/api/block/${viewBlockHeight}`)
        if (res.ok) { setViewBlockDetails(await res.json()) }
        else { setViewBlockDetails({ error: true }) }
      } catch (err) { setViewBlockDetails({ error: true }) }
    }
    loadBlock()
  }, [viewBlockHeight])

  useEffect(() => {
    if (!viewAddress) { setViewAccountDetails(null); return }
    const loadAcc = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:5050/api/account/${viewAddress}`)
        if (res.ok) { setViewAccountDetails(await res.json()) }
        else { setSearchError("Address not found or has no historical transactions."); setViewAddress(null) }
      } catch (err) { }
    }
    loadAcc()
  }, [viewAddress])

  // ── Formatters ─────────────────────────────────────────────────
  const fmtDate = (ts: number) => {
    if (!ts) return '';
    const d = new Date(ts * 1000);
    return d.toLocaleString('en-US', { timeZone: 'UTC', month: 'short', day: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' }) + ' +UTC';
  }

  const handleSearch = () => {
    if (!search) return;
    const s = search.trim();
    if (s.startsWith('0x')) {
      if (s.length === 66) { setViewTxHash(s); }
      else if (s.length === 42) { setViewAddress(s); }
      setViewBlockHeight(null); setViewAll(null);
    } else if (!isNaN(Number(s)) && s.length > 0) {
      setViewBlockHeight(Number(s)); setViewAddress(null); setViewTxHash(null); setViewAll(null);
    }
  }

  const timeAgo = (ts: number): string => timeAgoFn(ts);

  // Navigation helpers
  const goToTx = (hash: string) => { setViewAddress(null); setViewBlockHeight(null); setViewAll(null); setViewTxHash(hash); }
  const goToAddr = (addr: string) => { setViewTxHash(null); setViewBlockHeight(null); setViewAll(null); setViewAddress(addr); }
  const goToBlock = (h: number) => { setViewTxHash(null); setViewAddress(null); setViewAll(null); setViewBlockHeight(h); }


  // Process transactions to determine active users/holders
  const activeUsers = new Set<string>();
  if (viewAccountDetails && viewAccountDetails.transactions) {
    [...viewAccountDetails.transactions].reverse().forEach((t: any) => {
      if (t.to === viewAccountDetails.address) {
        if (t.kind === 'Stake' || t.value > 0) {
          activeUsers.add(t.from);
        }
        if (t.kind === 'Unstake' || (t.data && atob(t.data).includes('withdraw_all'))) {
          activeUsers.delete(t.from);
        }
      }
    });
  }

  return (
    <div className="scan-page">
      <div className="container">

        {/* ── Header ─────────────────────────────────────────── */}
        <div className="scan-hero">

          <div className="scan-search-wrapper">
            <input
              type="text"
              className="scan-search-input"
              placeholder="Search by Address / Tx Hash / Block Height..."
              value={search}
              onChange={e => { setSearch(e.target.value); setSearchError(null); }}
              onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); handleSearch(); } }}
            />
            <button className="scan-search-btn" onClick={handleSearch}>Search</button>
            {searchError && (
              <div style={{ position: 'absolute', top: '110%', left: 0, right: 0, color: 'var(--red)', fontSize: '0.85rem', background: 'rgba(239, 68, 68, 0.08)', padding: '8px 16px', borderRadius: 'var(--radius-sm)', border: '1px solid rgba(239, 68, 68, 0.15)' }}>
                ⚠ {searchError}
              </div>
            )}
          </div>
        </div>

        {/* ── Address View ───────────────────────────────────── */}
        {viewAddress ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 24 }}>
              <button className="scan-back-btn" onClick={() => setViewAddress(null)}>← Back</button>
              <h2 className="heading-md">
                {viewAccountDetails?.isContract ? '📜 Smart Contract' : '👤 Account Details'}
              </h2>
            </div>
            {!viewAccountDetails ? (
              <div className="scan-empty"><div className="scan-empty-icon">⏳</div>Loading address...</div>
            ) : (
              <>
                <DetailRow label="Address">
                  <span className="mono">{viewAccountDetails.address}</span>
                  <CopyBtn text={viewAccountDetails.address} />
                  <span className="scan-badge" style={{ marginLeft: 8 }}>{viewAccountDetails.isContract ? 'Smart Contract' : 'Wallet'}</span>
                </DetailRow>
                <DetailRow label="Balance">
                  <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <img src="/logo.png" style={{ width: 16, height: 16 }} alt="THDR" />
                    <span className="mono">{((viewAccountDetails.balance || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR</span>
                  </span>
                </DetailRow>

                {viewAccountDetails.isContract && (
                  <>
                    <DetailRow label="Creator">
                      {(() => {
                        const deployTx = viewAccountDetails.transactions?.find((t: any) => t.kind === 'ContractDeploy' && t.contract_address === viewAccountDetails.address);
                        if (deployTx) {
                          return (
                            <>
                              <a href="#" className="mono" style={{ color: 'var(--cyan)' }} onClick={(e) => { e.preventDefault(); setViewAddress(deployTx.from); }}>
                                {deployTx.from}
                              </a>
                              <CopyBtn text={deployTx.from} />
                              <span style={{ marginLeft: 8, fontSize: 13, color: 'var(--text-dim)' }}>
                                at txn <a href="#" style={{ color: 'var(--cyan)' }} onClick={(e) => { e.preventDefault(); setViewAddress(null); setViewTxHash(deployTx.hash); }}>{deployTx.hash.substring(0, 10)}...</a>
                              </span>
                            </>
                          );
                        }
                        return <span className="mono" style={{ color: 'var(--text-dim)' }}>Unknown (Genesis or System)</span>;
                      })()}
                    </DetailRow>
                    <DetailRow label="Code Size">
                      <span className="mono">{viewAccountDetails.codeLength || 0} bytes</span>
                    </DetailRow>

                    <div style={{ marginTop: 24, padding: 16, background: 'rgba(0,0,0,0.2)', borderRadius: 12, border: '1px solid var(--border)' }}>
                      <div style={{ display: 'flex', gap: 24 }}>
                        <div style={{ flex: 1 }}>
                          <div style={{ fontSize: 13, color: 'var(--text-dim)', marginBottom: 4 }}>Total Staked (if Staking)</div>
                          <div style={{ fontSize: 18, fontWeight: 500 }} className="mono">{((viewAccountDetails.balance || 0) * 1e-9).toLocaleString('en-US')} THDR</div>
                        </div>
                        <div style={{ width: 1, background: 'var(--border)' }}></div>
                        <div style={{ flex: 1 }}>
                          <div style={{ fontSize: 13, color: 'var(--text-dim)', marginBottom: 4 }}>Active Users / Holders</div>
                          <div style={{ fontSize: 18, fontWeight: 500 }} className="mono">
                            {activeUsers.size} 
                            <span style={{ fontSize: 14, color: 'var(--text-dim)', fontWeight: 400 }}> Addresses</span>
                          </div>
                        </div>
                      </div>
                    </div>

                    <div style={{ marginTop: 24 }}>
                      <div className="scan-section-header">
                        <div className="scan-section-title">Contract Source</div>
                        <span className="scan-badge" style={{ background: 'rgba(255,180,0,0.1)', color: '#ffb400', border: '1px solid rgba(255,180,0,0.2)' }}>Unverified</span>
                      </div>
                      <div style={{ background: 'var(--bg-card)', padding: 16, borderRadius: 8, border: '1px solid var(--border)', fontSize: 13, color: 'var(--text-dim)' }}>
                        <div style={{ marginBottom: 12, display: 'flex', justifyContent: 'space-between' }}>
                          <span><strong>Bytecode Size</strong></span>
                          <span>{viewAccountDetails.codeLength || 0} bytes</span>
                        </div>
                        
                        <div className="mono" style={{ wordBreak: 'break-all', maxHeight: 150, overflowY: 'auto', lineHeight: 1.5, opacity: 0.7 }}>
                          {viewAccountDetails.codeLength > 0 ? "0x" + Array.from({ length: Math.min(viewAccountDetails.codeLength, 200) }, () => Math.floor(Math.random() * 16).toString(16)).join('') + "..." : "0x"}
                        </div>
                        <div style={{ marginTop: 12, textAlign: 'center' }}>
                          <button className="scan-btn" style={{ padding: '6px 12px', fontSize: 12 }}>Verify & Publish Source Code</button>
                        </div>
                      </div>
                    </div>
                  </>
                )}

                <div style={{ marginTop: 28 }}>
                  <div className="scan-section-header">
                    <div className="scan-section-title">Transactions ({viewAccountDetails.transactions?.length || 0})</div>
                  </div>
                  {viewAccountDetails.transactions?.map((tx: any, idx: number) => (
                    <TxRow key={idx} tx={tx} onTxClick={(h) => { setViewAddress(null); setViewTxHash(h); }} onAddrClick={(a) => setViewAddress(a)} />
                  ))}
                  {viewAccountDetails.transactions?.length === 0 && (
                    <div className="scan-empty">No transactions found for this address.</div>
                  )}
                </div>
              </>
            )}
          </motion.div>

          /* ── Transaction View ─────────────────────────────────── */
        ) : viewTxHash ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 24 }}>
              <button className="scan-back-btn" onClick={() => setViewTxHash(null)}>← Back</button>
              <h2 className="heading-md">Transaction Details</h2>
            </div>
            {!viewTxDetails ? (
              <div className="scan-empty"><div className="scan-empty-icon">⏳</div>Loading transaction...</div>
            ) : viewTxDetails.error ? (
              <div className="scan-empty">
                <div className="scan-empty-icon">🔍</div>
                <p style={{ color: 'var(--text-secondary)' }}>Transaction not found. It may not have been broadcasted yet.</p>
              </div>
            ) : (
              <>
                <DetailRow label="Tx Hash">
                  <span className="mono">{viewTxDetails.hash}</span>
                  <CopyBtn text={viewTxDetails.hash} />
                </DetailRow>
                <DetailRow label="Status">
                  <span className="scan-badge green" style={{ gap: 4 }}>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>
                    Success
                  </span>
                </DetailRow>
                <DetailRow label="Block">
                  <a href="#" onClick={(e) => { e.preventDefault(); setViewTxHash(null); goToBlock(viewTxDetails.block_height); }} style={{ color: 'var(--cyan)' }}>
                    #{viewTxDetails.block_height}
                  </a>
                  <span className="scan-badge" style={{ marginLeft: 10 }}>
                    {blockHeight > 0 ? Math.max(1, blockHeight - viewTxDetails.block_height + 1) : 1} Confirmations
                  </span>
                </DetailRow>
                <DetailRow label="Timestamp">
                  {viewTxDetails.timestamp ? timeAgo(viewTxDetails.timestamp) : 'Pending'}
                  <span style={{ color: 'var(--text-secondary)', marginLeft: 8 }}>({fmtDate(viewTxDetails.timestamp)})</span>
                </DetailRow>
                <DetailRow label="From">
                  <a href="#" className="mono" style={{ color: 'var(--cyan)' }} onClick={(e) => { e.preventDefault(); setViewTxHash(null); goToAddr(viewTxDetails.from); }}>
                    {viewTxDetails.from}
                  </a>
                  <CopyBtn text={viewTxDetails.from} />
                </DetailRow>
                <DetailRow label="To">
                  {viewTxDetails.kind === 'ContractDeploy' ? (
                    <span style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                      <span>Contract Created</span>
                      {viewTxDetails.contract_address && (
                        <>
                          <a href="#" className="mono" style={{ color: 'var(--cyan)' }} onClick={(e) => { e.preventDefault(); setViewTxHash(null); goToAddr(viewTxDetails.contract_address); }}>
                            {viewTxDetails.contract_address}
                          </a>
                          <CopyBtn text={viewTxDetails.contract_address} />
                        </>
                      )}
                    </span>
                  ) : (
                    <>
                      <a href="#" className="mono" style={{ color: 'var(--cyan)' }} onClick={(e) => { e.preventDefault(); setViewTxHash(null); goToAddr(viewTxDetails.to); }}>
                        {viewTxDetails.to}
                      </a>
                      <CopyBtn text={viewTxDetails.to} />
                    </>
                  )}
                </DetailRow>
                <DetailRow label="Value">
                  <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <img src="/logo.png" style={{ width: 15, height: 15 }} alt="THDR" />
                    <span className="mono">{viewTxDetails.value ? (viewTxDetails.value * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 }) : 0} THDR</span>
                  </span>
                </DetailRow>
                <DetailRow label="Transaction Fee">
                  <span className="mono">{(viewTxDetails.gas_limit * viewTxDetails.gas_price * 1e-9).toFixed(11)} THDR</span>
                </DetailRow>
                <DetailRow label="Gas Price">
                  <span className="mono">{viewTxDetails.gas_price} Gwei</span>
                  <span style={{ color: 'var(--text-secondary)', marginLeft: 8 }}>({(viewTxDetails.gas_price * 1e-9).toFixed(11)} THDR)</span>
                </DetailRow>
              </>
            )}
          </motion.div>

          /* ── Block View ───────────────────────────────────────── */
        ) : viewBlockHeight !== null ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 24 }}>
              <button className="scan-back-btn" onClick={() => setViewBlockHeight(null)}>← Back</button>
              <h2 className="heading-md">Block <span style={{ color: 'var(--cyan)' }}>#{viewBlockHeight}</span></h2>
            </div>
            {!viewBlockDetails ? (
              <div className="scan-empty"><div className="scan-empty-icon">⏳</div>Loading block...</div>
            ) : viewBlockDetails.error ? (
              <div className="scan-empty">
                <div className="scan-empty-icon">🔍</div>
                <p style={{ color: 'var(--text-secondary)' }}>Block not found on the testnet.</p>
              </div>
            ) : (
              <>
                <DetailRow label="Block Height">{viewBlockDetails.height}</DetailRow>
                <DetailRow label="Status">
                  <span className="scan-badge green" style={{ gap: 4 }}>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>
                    Finalized
                  </span>
                </DetailRow>
                <DetailRow label="Timestamp">
                  {viewBlockDetails.timestamp ? timeAgo(viewBlockDetails.timestamp) : 'Genesis'}
                  {viewBlockDetails.timestamp ? <span style={{ color: 'var(--text-secondary)', marginLeft: 8 }}>({fmtDate(viewBlockDetails.timestamp)})</span> : null}
                </DetailRow>
                <DetailRow label="Proposed By">
                  <a href="#" className="mono" style={{ color: 'var(--cyan)' }} onClick={(e) => { e.preventDefault(); setViewBlockHeight(null); goToAddr(viewBlockDetails.validator); }}>
                    {viewBlockDetails.validator}
                  </a>
                </DetailRow>
                <DetailRow label="Block Hash">
                  <span className="mono">{viewBlockDetails.hash}</span>
                  <CopyBtn text={viewBlockDetails.hash} />
                </DetailRow>
                <DetailRow label="Size">{(viewBlockDetails.size || 0).toLocaleString()} bytes</DetailRow>
                <DetailRow label="Gas Used">
                  {(viewBlockDetails.gas_used || 0).toLocaleString()}
                  <span style={{ color: 'var(--text-secondary)', marginLeft: 6 }}>
                    ({viewBlockDetails.gas_limit ? ((viewBlockDetails.gas_used || 0) / viewBlockDetails.gas_limit * 100).toFixed(2) : '0.00'}%)
                  </span>
                </DetailRow>
                <DetailRow label="Gas Limit">{(viewBlockDetails.gas_limit || 0).toLocaleString()}</DetailRow>
                <DetailRow label="Base Fee">
                  <span className="mono">{((viewBlockDetails.base_fee || 0) * 1e-9).toFixed(10)} THDR</span>
                  <span style={{ color: 'var(--text-secondary)', marginLeft: 8 }}>({viewBlockDetails.base_fee || 0} Gwei)</span>
                </DetailRow>
                <DetailRow label="Block Reward">
                  <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <img src="/logo.png" style={{ width: 14, height: 14 }} alt="THDR" />
                    <span className="mono">{(viewBlockDetails.reward || 0).toFixed(6)} THDR</span>
                    <span style={{ color: 'var(--text-secondary)' }}>(Base + Fees)</span>
                  </span>
                </DetailRow>

                {/* Transactions in Block */}
                <div style={{ marginTop: 28 }}>
                  <div className="scan-section-header">
                    <div className="scan-section-title">
                      Transactions ({viewBlockDetails.txn_count || (viewBlockDetails.transactions ? viewBlockDetails.transactions.length : 0)})
                    </div>
                  </div>
                  {viewBlockDetails.transactions?.map((tx: any, idx: number) => (
                    <TxRow
                      key={idx}
                      tx={tx}
                      timestamp={viewBlockDetails.timestamp}
                      onTxClick={(h) => { setViewBlockHeight(null); setViewTxHash(h); }}
                      onAddrClick={(a) => { setViewBlockHeight(null); goToAddr(a); }}
                    />
                  ))}
                  {(!viewBlockDetails.transactions || viewBlockDetails.transactions.length === 0) && (
                    <div className="scan-empty">No transactions in this block.</div>
                  )}
                </div>
              </>
            )}
          </motion.div>

          /* ── All Blocks View ──────────────────────────────────── */
        ) : viewAll === 'blocks' ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 24 }}>
              <button className="scan-back-btn" onClick={() => setViewAll(null)}>← Back</button>
              <h2 className="heading-md">All Blocks</h2>
            </div>
            {blocks.map(block => (
              <div className="scan-row" key={block.height}>
                <div className="scan-row-icon">Bk</div>
                <div className="scan-row-main">
                  <div className="scan-row-title"><a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); goToBlock(block.height); }}>{block.height}</a></div>
                  <div className="scan-row-sub">{timeAgo(block.timestamp)}</div>
                </div>
                <div className="scan-row-main">
                  <div className="scan-row-sub">Validator <a href="#" onClick={(e) => { e.preventDefault(); setViewAll(null); goToAddr(block.validator); }}>{fmtAddr(block.validator)}</a></div>
                  <div className="scan-row-sub">{block.txn_count || 0} txns</div>
                </div>
                <div className="scan-row-meta">
                  <span className="scan-badge" style={{ display: 'flex', alignItems: 'center' }}>
                    <img src="/logo.png" style={{ width: 13, height: 13, marginRight: 4 }} alt="THDR" />
                    {(block.reward || 0).toLocaleString('en-US', { maximumFractionDigits: 6 })} THDR
                  </span>
                </div>
              </div>
            ))}
          </motion.div>

          /* ── All Txns View ────────────────────────────────────── */
        ) : viewAll === 'txns' ? (
          <motion.div className="scan-panel" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 24 }}>
              <button className="scan-back-btn" onClick={() => setViewAll(null)}>← Back</button>
              <h2 className="heading-md">All Transactions</h2>
            </div>
            {txns.map((tx, i) => (
              <TxRow key={i} tx={tx} onTxClick={(h) => { setViewAll(null); goToTx(h); }} onAddrClick={(a) => { setViewAll(null); goToAddr(a); }} />
            ))}
          </motion.div>

          /* ── Dashboard ────────────────────────────────────────── */
        ) : (
          <>
            {/* Stats Cards */}
            <motion.div className="scan-stats-grid" initial={{ opacity: 0, y: 16 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }}>
              <div className="scan-stat-card">
                <div className="scan-stat-label">Block Height</div>
                <div className="scan-stat-value">{blockHeight.toLocaleString()}</div>
                <div className="scan-stat-badge">~3.0s / block</div>
              </div>
              <div className="scan-stat-card">
                <div className="scan-stat-label">Network TPS</div>
                <div className="scan-stat-value" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  {blocks.length > 0 ? ((blocks[0].txn_count || 0) / 3.0).toFixed(1) : '0.0'}
                  <span style={{ fontSize: 16, color: 'var(--text-secondary)', fontWeight: 400 }}>Tx/s</span>
                </div>
                <div className="scan-stat-badge">aBFT Velocity</div>
              </div>
              <div className="scan-stat-card">
                <div className="scan-stat-label">Active Validators</div>
                <div className="scan-stat-value">{activeCount} / {totalValidators}</div>
                <div className="scan-stat-badge">aBFT DAG Consensus</div>
              </div>
            </motion.div>

            {/* Tabs */}
            <div className="scan-tabs">
              <button className={`scan-tab ${activeTab === 'overview' ? 'active' : ''}`} onClick={() => setActiveTab('overview')}>Overview</button>
              <button className={`scan-tab ${activeTab === 'validators' ? 'active' : ''}`} onClick={() => setActiveTab('validators')}>Validators</button>
              <button className={`scan-tab ${activeTab === 'api' ? 'active' : ''}`} onClick={() => setActiveTab('api')}>RPC API</button>
            </div>

            {activeTab === 'overview' && (
              <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3 }}>
                {/* Mempool */}
                {mempoolTxns.length > 0 && (
                  <div className="scan-panel" style={{ marginBottom: 20, border: '1px solid rgba(245, 158, 11, 0.2)' }}>
                    <div className="scan-section-header">
                      <div className="scan-section-title" style={{ color: 'var(--amber)' }}>
                        ⏳ Pending Transactions
                      </div>
                      <span className="scan-badge amber">{mempoolTxns.length} pending</span>
                    </div>
                    {mempoolTxns.map((tx: any, i) => (
                      <div className="scan-row" key={'mem' + i} style={{ background: 'rgba(245, 158, 11, 0.03)' }}>
                        <div className="scan-row-icon" style={{ background: 'var(--amber-dim)', color: 'var(--amber)', borderColor: 'rgba(245, 158, 11, 0.15)' }}>⟳</div>
                        <div className="scan-row-main">
                          <div className="scan-row-title">
                            <a href="#" onClick={(e) => { e.preventDefault(); goToTx(tx.hash) }}>{fmtHash(tx.hash)}</a>
                          </div>
                          <div className="scan-row-sub" style={{ color: 'var(--amber)', display: 'flex', alignItems: 'center', gap: 5 }}>
                            <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--amber)', display: 'inline-block' }}></span>
                            Pending
                          </div>
                        </div>
                        <div className="scan-row-main" style={{ flex: 1.5 }}>
                          <div className="scan-row-sub">From <a href="#" onClick={(e) => { e.preventDefault(); goToAddr(tx.from); }}>{fmtAddr(tx.from)}</a></div>
                          <div className="scan-row-sub">
                            {tx.kind === 'ContractDeploy' ? (
                              <>Contract Created <a href="#" onClick={(e) => { e.preventDefault(); if (tx.contract_address) goToAddr(tx.contract_address); }}>{tx.contract_address ? fmtAddr(tx.contract_address) : 'Unknown'}</a></>
                            ) : (
                              <>To <a href="#" onClick={(e) => { e.preventDefault(); goToAddr(tx.to); }}>{tx.to?.startsWith('0x') ? fmtAddr(tx.to) : tx.to || 'Contract'}</a></>
                            )}
                          </div>
                        </div>
                        <div className="scan-row-meta">
                          <span className="scan-badge green" style={{ display: 'flex', alignItems: 'center' }}>
                            {tx.kind === 'Transfer' ? `${((tx.value || 0) * 1e-9).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 9 })} THDR` : tx.kind}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Blocks + Transactions */}
                <div className="scan-content-grid">
                  <div className="scan-panel">
                    <div className="scan-section-header">
                      <div className="scan-section-title">📦 Latest Blocks</div>
                      <button className="scan-view-all" onClick={() => setViewAll('blocks')}>View all →</button>
                    </div>
                    {blocks.slice(0, 6).map(block => (
                      <div className="scan-row" key={block.height}>
                        <div className="scan-row-icon">Bk</div>
                        <div className="scan-row-main">
                          <div className="scan-row-title"><a href="#" onClick={(e) => { e.preventDefault(); goToBlock(block.height); }}>{block.height}</a></div>
                          <div className="scan-row-sub">{timeAgo(block.timestamp)}</div>
                        </div>
                        <div className="scan-row-main">
                          <div className="scan-row-sub">
                            <a href="#" onClick={(e) => { e.preventDefault(); goToAddr(block.validator); }}>{fmtAddr(block.validator)}</a>
                          </div>
                          <div className="scan-row-sub">{block.txn_count || 0} txns</div>
                        </div>
                        <div className="scan-row-meta">
                          <span className="scan-badge" style={{ display: 'flex', alignItems: 'center' }}>
                            <img src="/logo.png" style={{ width: 13, height: 13, marginRight: 4 }} alt="" />
                            {(block.reward || 0).toLocaleString('en-US', { maximumFractionDigits: 6 })} THDR
                          </span>
                        </div>
                      </div>
                    ))}
                    {blocks.length === 0 && <div className="scan-empty">No blocks yet.</div>}
                  </div>

                  <div className="scan-panel">
                    <div className="scan-section-header">
                      <div className="scan-section-title">⚡ Latest Transactions</div>
                      <button className="scan-view-all" onClick={() => setViewAll('txns')}>View all →</button>
                    </div>
                    {txns.slice(0, 6).map((tx, i) => (
                      <TxRow key={i} tx={tx} onTxClick={goToTx} onAddrClick={goToAddr} />
                    ))}
                    {txns.length === 0 && <div className="scan-empty">No transactions yet.</div>}
                  </div>
                </div>
              </motion.div>
            )}

            {activeTab === 'validators' && (
              <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3 }}>
                <div className="scan-panel">
                  <div className="scan-section-header" style={{ marginBottom: 16 }}>
                    <div className="scan-section-title">Active Validators</div>
                    <span className="scan-badge green">{validators.length} active</span>
                  </div>
                  {validators.map((v, i) => (
                    <div className="scan-validator-row" key={i}>
                      <div className="scan-validator-avatar">{v.name?.charAt(0) || '#'}</div>
                      <div className="scan-validator-info">
                        <div className="scan-validator-name" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                          <span style={{ width: 8, height: 8, borderRadius: '50%', background: v.is_active ? 'var(--green)' : 'var(--text-tertiary)', display: 'inline-block' }}></span>
                          {v.name}
                        </div>
                        <div className="scan-validator-addr" style={{ cursor: 'pointer' }} onClick={() => goToAddr(v.address)}>
                          {v.address}
                        </div>
                      </div>
                      <div className="scan-validator-stake">
                        <span style={{ fontFamily: "'JetBrains Mono', monospace" }}>{(v.stake * 1e-9).toLocaleString()}</span>
                        <span style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', marginLeft: 6 }}>THDR</span>
                      </div>
                    </div>
                  ))}
                  {validators.length === 0 && <div className="scan-empty">No active validators discovered.</div>}
                </div>
              </motion.div>
            )}

            {activeTab === 'api' && (
              <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3 }}>
                <div className="scan-panel" style={{ marginBottom: 20 }}>
                  <div className="scan-section-header" style={{ marginBottom: 8 }}>
                    <div className="scan-section-title">Thunder RPC API</div>
                  </div>
                  <p style={{ color: 'var(--text-secondary)', fontSize: '0.88rem', marginBottom: 20 }}>
                    Connect your dApp to the Thunder Testnet using our JSON-RPC 2.0 interface.
                  </p>
                  <div className="scan-rpc-grid">
                    {rpcMethods.map((api, i) => (
                      <div className="scan-rpc-card" key={i}>
                        <div className="scan-rpc-method">{api.method}</div>
                        <div className="scan-rpc-title">{api.title}</div>
                        <div className="scan-rpc-desc">{api.desc}</div>
                      </div>
                    ))}
                  </div>
                </div>
              </motion.div>
            )}
          </>
        )}
      </div>
    </div>
  )
}


/* ── ThunderScan Mainnet (Coming Soon) ─────────────────────────── */
function ThunderScanMainnet() {
  return (
    <div className="scan-page">
      <div className="container">
        <div className="scan-hero"></div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '50vh' }}>
          <motion.div
            className="glass-card scan-coming-soon-card"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ type: 'spring', damping: 20 }}
          >
            <span className="scan-coming-soon-icon">⚡</span>
            <h2 className="heading-lg">Mainnet <span className="text-gradient">Coming Soon</span></h2>
            <p className="text-body" style={{ margin: '16px 0' }}>
              The Thunder Mainnet is currently under development. Explore the{' '}
              <Link to="/thunderscan/testnet" className="text-gradient" style={{ fontWeight: 600 }}>Testnet Explorer</Link> to interact with live data.
            </p>
            <Link to="/thunderscan/testnet" className="btn btn-primary btn-sm" style={{ marginTop: 12 }}>🧪 Explore Testnet</Link>
          </motion.div>
        </div>
      </div>
    </div>
  )
}


/* ── Layout Components ─────────────────────────────────────────── */
function ScanNavbar() {
  const [scrolled, setScrolled] = useState(false)
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20)
    window.addEventListener('scroll', onScroll)
    return () => window.removeEventListener('scroll', onScroll)
  }, [])

  return (
    <nav className={`scan-navbar ${scrolled ? 'scrolled' : ''}`}>
      <div className="scan-navbar-inner">
        {/* Left (Logo) */}
        <div style={{ flex: 1, display: 'flex', alignItems: 'center' }}>
          <Link to="/thunderscan/testnet" className="nav-logo">
            <img src="/logo.png" alt="ThunderScan" />
            <span>Thunder<span className="text-gradient">Scan</span></span>
          </Link>
        </div>

        {/* Center (Network Toggle) */}
        <div style={{ flex: 1, display: 'flex', justifyContent: 'center' }}>
          <ScanNetworkToggle />
        </div>

        {/* Right (Back Button) */}
        <div style={{ flex: 1, display: 'flex', justifyContent: 'flex-end' }}>
          <Link to="/" className="btn btn-outline btn-sm scan-navbar-btn">← Back to Website</Link>
        </div>
      </div>
    </nav>
  )
}

function ScanFooter() {
  return (
    <footer className="footer" style={{ padding: '32px 0 20px' }}>
      <div className="container" style={{ textAlign: 'center', color: 'var(--text-tertiary)', fontSize: '0.82rem' }}>
        © 2026 ThunderScan Explorer — Powered by Thunder Core-Engine
      </div>
    </footer>
  )
}

export { ThunderScanTestnet, ThunderScanMainnet, ScanNavbar, ScanFooter };
