import { useEffect, useRef, useState } from 'react'
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom'
import { motion } from 'framer-motion'
import { Code, KeySquare, Rocket, GitMerge } from 'lucide-react'
import './App.css'

/* ── Hooks (Same) ──────────────────────────────────────────────── */
function useCounter(target: number, duration = 2000, suffix = '') {
  const [count, setCount] = useState(0)
  const ref = useRef<HTMLDivElement>(null)
  const started = useRef(false)

  useEffect(() => {
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting && !started.current) {
        started.current = true
        const start = performance.now()
        const animate = (now: number) => {
          const progress = Math.min((now - start) / duration, 1)
          const eased = 1 - Math.pow(1 - progress, 3)
          setCount(Math.floor(eased * target))
          if (progress < 1) requestAnimationFrame(animate)
        }
        requestAnimationFrame(animate)
      }
    }, { threshold: 0.3 })
    if (ref.current) observer.observe(ref.current)
    return () => observer.disconnect()
  }, [target, duration])
  return { ref, value: count.toLocaleString() + suffix }
}

function useReveal() {
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => entries.forEach(e => {
        if (e.isIntersecting) e.target.classList.add('visible')
      }),
      { threshold: 0.1 }
    )
    const el = ref.current
    if (el) {
      el.querySelectorAll('.reveal').forEach(c => observer.observe(c))
    }
    return () => observer.disconnect()
  }, [])
  return ref
}

/* ── Navbar ────────────────────────────────────────────────────── */
function Navbar() {
  const [scrolled, setScrolled] = useState(false)
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 50)
    window.addEventListener('scroll', onScroll)
    return () => window.removeEventListener('scroll', onScroll)
  }, [])
  return (
    <nav className={`navbar ${scrolled ? 'scrolled' : ''}`}>
      <div className="container">
        <Link to="/" className="nav-logo">
          <img src="/logo.png" alt="Thunder" />
          <span>Thunder</span>
        </Link>
        <div className="nav-links">
          <Link to="/docs">Documentation</Link>
          <a href="/#features">Features</a>
          <a href="/#ecosystem">Ecosystem</a>
          <a href="/#comparison">Comparison</a>
        </div>
        <div className="nav-cta">
          <Link to="/coming-soon?product=wallet" className="btn btn-outline">Thunder Wallet</Link>
          <Link to="/coming-soon?product=scan" className="btn btn-primary">ThunderScan →</Link>
        </div>
      </div>
    </nav>
  )
}

/* ── Missing Components from Landing ────────────────────────────── */
function Hero() {
  const tps = useCounter(400000, 2500, '+')
  const finality = useCounter(1, 1000)
  const fee = useCounter(1, 1500)
  const uptime = useCounter(99, 2000)
  return (
    <section className="hero" id="hero">
      <div className="hero-bg">
        <div className="orb orb-1"></div>
        <div className="orb orb-2"></div>
        <div className="orb orb-3"></div>
      </div>
      <div className="container hero-content">
        <img src="/logo.png" alt="Thunder Blockchain" className="hero-logo" />
        <h1 className="heading-xl hero-tagline">
          The Fastest<br />
          <span className="text-gradient">Sovereign Blockchain</span>
        </h1>
        <p className="text-body hero-desc">
          Powered by DAG-based aBFT Consensus with zero downtime, ultra-low gas fees, and omnichain compatibility. Built entirely in Rust.
        </p>
        <div className="hero-btns">
          <Link to="/coming-soon?product=scan" className="btn btn-primary">⚡ Explore ThunderScan</Link>
          <Link to="/coming-soon?product=wallet" className="btn btn-outline">Download Wallet</Link>
        </div>
        <div className="hero-stats-bar">
          <div className="hero-stat" ref={tps.ref}><div className="hero-stat-value text-gradient">{tps.value}</div><div className="hero-stat-label">TPS Throughput</div></div>
          <div className="hero-stat" ref={finality.ref}><div className="hero-stat-value text-gradient">&lt;{finality.value}s</div><div className="hero-stat-label">Finality</div></div>
          <div className="hero-stat" ref={fee.ref}><div className="hero-stat-value text-gradient">${fee.value === '0' ? '0' : '0.000' + fee.value}</div><div className="hero-stat-label">Avg Gas Fee</div></div>
          <div className="hero-stat" ref={uptime.ref}><div className="hero-stat-value text-gradient">{uptime.value}.99%</div><div className="hero-stat-label">Uptime</div></div>
        </div>
      </div>
    </section>
  )
}

function Features() {
  const ref = useReveal()
  const features = [{ icon: '⚡', cls: '', title: 'aBFT DAG Consensus', desc: 'Leaderless virtual voting using Hashgraph-inspired DAG. No single point of failure.' }, { icon: '🔮', cls: 'purple', title: 'ThunderVM & ThunderScript', desc: 'Custom VM compiling to bytecode with micro-gas metering.' }, { icon: '🌐', cls: 'pink', title: 'Omnichain Bridge', desc: 'Cross-chain bridges to ETH, BSC, and SOL featuring 15% authority caps.' }, { icon: '🛡️', cls: '', title: 'Defense-in-Depth', desc: '5-layer security architecture.' }, { icon: '💎', cls: 'purple', title: 'RocksDB Engine', desc: 'Parallelized storage via C++ bindings.' }, { icon: '🔐', cls: 'pink', title: 'Quantum-Ready', desc: 'Roadmap includes CRYSTALS-Dilithium.' }]
  return (
    <section className="section" id="features" ref={ref}>
      <div className="container">
        <div className="section-label reveal">⚡ Core Technology</div>
        <h2 className="heading-lg section-title reveal">Why <span className="text-gradient">Thunder?</span></h2>
        <div className="features-grid">
          {features.map((f, i) => (
            <div className="glass-card reveal" key={i}>
              <div className={`feature-icon ${f.cls}`}>{f.icon}</div>
              <h3 className="heading-md feature-title">{f.title}</h3>
              <p className="feature-desc">{f.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

function Ecosystem() {
  const ref = useReveal()
  return (
    <section className="section" id="ecosystem" ref={ref} style={{ background: 'var(--bg-secondary)' }}>
      <div className="container">
        <div className="section-label reveal">🌐 Products</div>
        <h2 className="heading-lg section-title reveal">Thunder <span className="text-gradient">Ecosystem</span></h2>
        <div className="eco-grid">
          <div className="glass-card eco-card reveal"><span className="eco-icon">🔍</span><h3 className="eco-title">ThunderScan</h3><p className="eco-desc">Block explorer.</p><Link to="/coming-soon?product=scan" className="eco-link">Learn More →</Link></div>
          <div className="glass-card eco-card reveal"><span className="eco-icon">💼</span><h3 className="eco-title">Thunder Wallet</h3><p className="eco-desc">Browser extension wallet.</p><Link to="/coming-soon?product=wallet" className="eco-link">Learn More →</Link></div>
          <div className="glass-card eco-card reveal"><span className="eco-icon">📜</span><h3 className="eco-title">ThunderScript</h3><p className="eco-desc">Smart contract language.</p><Link to="/docs" className="eco-link">Read Docs →</Link></div>
        </div>
      </div>
    </section>
  )
}

function Comparison() {
  const ref = useReveal()
  const rows = [
    { metric: 'Consensus', eth: 'Proof of Stake', bsc: 'Delegated PoS', sol: 'Proof of History', tdr: 'aBFT DAG (Virtual Voting)' },
    { metric: 'Speed (TPS)', eth: '~15-30', bsc: '~300', sol: '~65,000', tdr: '400,000+' },
    { metric: 'Finality', eth: '~12 min', bsc: '~3 sec', sol: '~400ms', tdr: 'Instant (Mathematical)' },
    { metric: 'Gas Fee', eth: 'Very High', bsc: 'Low', sol: 'Very Low', tdr: 'Ultra Low (Micro-Gas)' },
    { metric: 'Downtime Risk', eth: 'Low', bsc: 'Medium (21 Nodes)', sol: 'High (Frequent Outages)', tdr: 'Zero (Leaderless)' },
    { metric: 'Smart Contract VM', eth: 'EVM', bsc: 'EVM (Fork)', sol: 'Sealevel (eBPF)', tdr: 'ThunderVM (Custom)' },
  ]

  return (
    <section className="section" id="comparison" ref={ref}>
      <div className="container">
        <div className="section-label reveal">📊 Benchmarks</div>
        <h2 className="heading-lg section-title reveal">Chain <span className="text-gradient">Comparison</span></h2>
        <div className="reveal" style={{ overflowX: 'auto' }}>
          <table className="comparison-table">
            <thead>
              <tr>
                <th>Metric</th>
                <th>Ethereum</th>
                <th>BSC</th>
                <th>Solana</th>
                <th className="thunder-col">⚡ Thunder</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r, i) => (
                <tr key={i}>
                  <td style={{ fontWeight: 600, color: 'var(--white)' }}>{r.metric}</td>
                  <td>{r.eth}</td>
                  <td>{r.bsc}</td>
                  <td>{r.sol}</td>
                  <td className="thunder-col">{r.tdr}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  )
}

function Footer() {
  return (
    <footer className="footer">
      <div className="container" style={{ textAlign: 'center', color: 'var(--text-secondary)' }}>
        © 2026 Thunder Blockchain. Built with ⚡ in Rust.
      </div>
    </footer>
  )
}

/* ── UPDATED: How It Works (With Animation/Lucide Icons) ────────── */
function HowItWorks() {
  const steps = [
    { icon: <KeySquare size={36} color="var(--cyan)" />, title: 'Create Wallet', desc: 'Securely generate Ed25519 keypairs entirely offline.' },
    { icon: <Code size={36} color="#ec4899" />, title: 'Write Contract', desc: 'Author intelligent ThunderScript contracts in Rust-like syntax.' },
    { icon: <Rocket size={36} color="var(--purple)" />, title: 'Deploy & Execute', desc: 'Deploy via our local ThunderVM sandboxed compiler.' },
    { icon: <GitMerge size={36} color="var(--cyan)" />, title: 'Bridge Liquidity', desc: 'Lock ETH/BSC assets and seamlessly mint wrapped THDR equivalents.' }
  ]

  return (
    <section className="section" id="how-it-works" style={{ background: 'var(--bg-secondary)' }}>
      <div className="container">
        <motion.div initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true }}>
          <div className="section-label">🚀 Interactive Process</div>
          <h2 className="heading-lg section-title">How <span className="text-gradient">It Works</span></h2>
          <p className="text-body section-desc">
            A frictionless pipeline moving from bare-metal code to an interconnected omnichain reality.
          </p>
        </motion.div>

        <div className="steps-container">
          {steps.map((st, i) => (
            <motion.div
              key={i}
              className="glass-card animated-step-card"
              custom={i}
              initial="hidden"
              whileInView="visible"
              whileHover="hover"
              viewport={{ once: true, margin: "-50px" }}
            >
              <div className="step-icon-wrapper">
                {st.icon}
                <div className="step-glow"></div>
              </div>
              <h3 className="heading-md">{st.title}</h3>
              <p className="text-body">{st.desc}</p>

              {i < steps.length - 1 && (
                <div className="step-connector">
                  <motion.div
                    className="connector-line"
                    initial={{ width: 0 }}
                    whileInView={{ width: '100%' }}
                    transition={{ delay: 1 + (i * 0.2), duration: 0.8 }}
                    viewport={{ once: true }}
                  />
                </div>
              )}
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  )
}

/* ── Docs & Coming Soon ────────────────────────────────────────── */
function Landing() {
  return <><Hero /><Features /><Ecosystem /><HowItWorks /><Comparison /></>
}

function Docs() {
  return (
    <div style={{ paddingTop: 100, minHeight: '80vh', paddingBottom: 100 }}>
      <div className="container">
        <h1 className="heading-xl">ThunderScript <span className="text-gradient">Documentation</span></h1>
        <p className="text-body" style={{ marginTop: 24, fontSize: '1.2rem', maxWidth: 800 }}>
          Welcome to the official developer portal for Thunder Blockchain. Learn how to scaffold, deploy, and interact with smart contracts natively.
        </p>

        <div className="docs-grid" style={{ marginTop: 64, display: 'grid', gridTemplateColumns: 'minmax(250px, 1fr) 3fr', gap: 48 }}>
          <div className="docs-sidebar glass-card">
            <h3 className="heading-md" style={{ marginBottom: 16 }}>Table of Contents</h3>
            <ul style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <li><a href="#" className="text-gradient" style={{ fontWeight: 600 }}>1. Intro to ThunderScript</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)' }}>2. Creating a Wallet</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)' }}>3. Writing your First Contract</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)' }}>4. Compiling via CLI</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)' }}>5. Deploying to Testnet</a></li>
            </ul>
          </div>
          <div className="docs-content glass-card">
            <h2>1. Intro to ThunderScript</h2>
            <p>ThunderScript is a stack-based smart contract language optimized specifically for `ThunderVM`. It forces O(1) memory mappings to prevent infinite loops and recursive depth attacks.</p>
            <br />
            <h3>Example: Setting a basic String</h3>
            <div style={{ background: '#000', padding: 16, borderRadius: 8, marginTop: 16, border: '1px solid var(--glass-border)', fontFamily: 'monospace', color: 'var(--cyan)' }}>
              fn init() {'{'}<br />
              &nbsp;&nbsp;PStore "MyFirstContract"<br />
              {'}'}
            </div>
            <br />
            <h3>Executing via CLI</h3>
            <p style={{ marginTop: 8 }}>Use the built in CLI debugging toolbox to compile and dry-run without spending gas.</p>
            <div style={{ background: '#000', padding: 16, borderRadius: 8, marginTop: 16, border: '1px solid var(--glass-border)', fontFamily: 'monospace', color: 'var(--purple)' }}>
              $ cargo run -p thunder-cli -- contract run examples/token.thunder
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function ComingSoon() {
  const query = new URLSearchParams(window.location.search)
  const product = query.get('product') || 'App'

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <motion.div
        className="glass-card"
        style={{ textAlign: 'center', maxWidth: 600, padding: 64 }}
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ type: 'spring' }}
      >
        <div style={{ fontSize: '4rem', marginBottom: 24 }}>🚀</div>
        <h1 className="heading-lg">Thunder <span className="text-gradient">{product === 'scan' ? 'Scan' : 'Wallet'}</span></h1>
        <p className="text-body" style={{ margin: '24px 0' }}>
          We are currently putting the final touches on {product === 'scan' ? 'the block explorer' : 'the browser extension wallet'}.
          It will be released in Phase 12 of the roadmap.
        </p>
        <Link to="/" className="btn btn-outline" style={{ marginTop: 24 }}>← Back Home</Link>
      </motion.div>
    </div>
  )
}

/* ── Main App ─────────────────────────────────────────────────── */
function App() {
  return (
    <Router>
      <Navbar />
      <Routes>
        <Route path="/" element={<Landing />} />
        <Route path="/docs" element={<Docs />} />
        <Route path="/coming-soon" element={<ComingSoon />} />
      </Routes>
      <Footer />
    </Router>
  )
}

export default App
