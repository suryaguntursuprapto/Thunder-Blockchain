import { useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import { motion } from 'framer-motion'
import { Code, KeySquare, Rocket, GitMerge } from 'lucide-react'
import '../App.css'

/* ── Hooks ─────────────────────────────────────────────────────── */
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
          <Link to="/thunderscan/testnet">ThunderScan</Link>
        </div>
        <div className="nav-cta">
          <Link to="/coming-soon?product=wallet" className="btn btn-outline btn-sm">Thunder Wallet</Link>
          <Link to="/thunderscan/testnet" className="btn btn-primary btn-sm">⚡ ThunderScan</Link>
        </div>
      </div>
    </nav>
  )
}

/* ── Hero ──────────────────────────────────────────────────────── */
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
      <div className="hero-grid-bg"></div>

      <div className="container hero-content">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: [0.4, 0, 0.2, 1] }}
        >
          <img src="/logo.png" alt="Thunder Blockchain" className="hero-logo" />
          <h1 className="heading-xl hero-tagline">
            The Fastest<br />
            <span className="text-gradient">Sovereign Blockchain</span>
          </h1>
          <p className="text-body hero-desc">
            Powered by DAG-based aBFT Consensus with zero downtime, ultra-low gas fees, and omnichain compatibility. Built entirely in Rust.
          </p>
          <div className="hero-btns">
            <Link to="/thunderscan/testnet" className="btn btn-primary">⚡ Explore ThunderScan</Link>
            <Link to="/coming-soon?product=wallet" className="btn btn-outline">Download Wallet</Link>
          </div>
        </motion.div>

        <motion.div
          className="hero-stats-bar"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.4 }}
        >
          <div className="hero-stat" ref={tps.ref}>
            <div className="hero-stat-value text-gradient">{tps.value}</div>
            <div className="hero-stat-label">TPS Throughput</div>
          </div>
          <div className="hero-stat" ref={finality.ref}>
            <div className="hero-stat-value text-gradient">&lt;{finality.value}s</div>
            <div className="hero-stat-label">Finality</div>
          </div>
          <div className="hero-stat" ref={fee.ref}>
            <div className="hero-stat-value text-gradient">${fee.value === '0' ? '0' : '0.000' + fee.value}</div>
            <div className="hero-stat-label">Avg Gas Fee</div>
          </div>
          <div className="hero-stat" ref={uptime.ref}>
            <div className="hero-stat-value text-gradient">{uptime.value}.99%</div>
            <div className="hero-stat-label">Uptime</div>
          </div>
        </motion.div>
      </div>
    </section>
  )
}

/* ── Features ──────────────────────────────────────────────────── */
function Features() {
  const ref = useReveal()
  const features = [
    { icon: '⚡', cls: '', title: 'aBFT DAG Consensus', desc: 'Leaderless virtual voting using Hashgraph-inspired DAG. No single point of failure.' },
    { icon: '🔮', cls: 'purple', title: 'ThunderVM & ThunderScript', desc: 'Custom VM compiling to bytecode with micro-gas metering and O(1) memory safety.' },
    { icon: '🌐', cls: 'pink', title: 'Omnichain Bridge', desc: 'Cross-chain bridges to ETH, BSC, and SOL featuring 15% authority caps for security.' },
    { icon: '🛡️', cls: '', title: 'Defense-in-Depth', desc: '5-layer security architecture with Ed25519 signatures and quantum-ready roadmap.' },
    { icon: '💎', cls: 'purple', title: 'RocksDB Engine', desc: 'Parallelized persistent storage via C++ bindings for blazing-fast state queries.' },
    { icon: '🔐', cls: 'pink', title: 'Quantum-Ready Cryptography', desc: 'Roadmap includes CRYSTALS-Dilithium lattice-based signatures for post-quantum era.' }
  ]

  return (
    <section className="section" id="features" ref={ref}>
      <div className="container">
        <div className="section-label reveal">⚡ Core Technology</div>
        <h2 className="heading-lg section-title reveal">Why <span className="text-gradient">Thunder?</span></h2>
        <p className="text-body section-desc reveal">
          A next-generation Layer 1 blockchain engineered for maximum throughput, instant finality, and unbreakable security.
        </p>
        <div className="features-grid">
          {features.map((f, i) => (
            <motion.div
              className="glass-card reveal"
              key={i}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.1, duration: 0.5 }}
            >
              <div className={`feature-icon ${f.cls}`}>{f.icon}</div>
              <h3 className="heading-md feature-title">{f.title}</h3>
              <p className="feature-desc">{f.desc}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  )
}

/* ── Ecosystem ─────────────────────────────────────────────────── */
function Ecosystem() {
  const ref = useReveal()
  return (
    <section className="section" id="ecosystem" ref={ref} style={{ background: 'var(--bg-secondary)' }}>
      <div className="container">
        <div className="section-label reveal">🌐 Products</div>
        <h2 className="heading-lg section-title reveal">Thunder <span className="text-gradient">Ecosystem</span></h2>
        <p className="text-body section-desc reveal">
          A complete suite of tools and applications built on the Thunder network.
        </p>
        <div className="eco-grid">
          <motion.div className="glass-card eco-card reveal" whileHover={{ y: -4 }}>
            <span className="eco-icon">🔍</span>
            <h3 className="eco-title">ThunderScan</h3>
            <p className="eco-desc">Real-time block explorer with live transaction tracking, validator monitoring, and network analytics.</p>
            <Link to="/thunderscan/testnet" className="eco-link">Explore Testnet →</Link>
          </motion.div>
          <motion.div className="glass-card eco-card reveal" whileHover={{ y: -4 }}>
            <span className="eco-icon">💼</span>
            <h3 className="eco-title">Thunder Wallet</h3>
            <p className="eco-desc">Secure browser extension wallet with Ed25519 keypair generation and cross-chain asset management.</p>
            <Link to="/coming-soon?product=wallet" className="eco-link">Learn More →</Link>
          </motion.div>
          <motion.div className="glass-card eco-card reveal" whileHover={{ y: -4 }}>
            <span className="eco-icon">📜</span>
            <h3 className="eco-title">ThunderScript</h3>
            <p className="eco-desc">Stack-based smart contract language with forced O(1) memory mappings and micro-gas metering.</p>
            <Link to="/docs" className="eco-link">Read Docs →</Link>
          </motion.div>
        </div>
      </div>
    </section>
  )
}

/* ── How It Works ──────────────────────────────────────────────── */
function HowItWorks() {
  const steps = [
    { icon: <KeySquare size={28} color="var(--cyan)" />, title: 'Create Wallet', desc: 'Generate Ed25519 keypairs entirely offline with military-grade entropy.' },
    { icon: <Code size={28} color="#ec4899" />, title: 'Write Contract', desc: 'Author ThunderScript contracts with Rust-like syntax and built-in safety.' },
    { icon: <Rocket size={28} color="var(--purple)" />, title: 'Deploy & Execute', desc: 'Deploy via ThunderVM sandboxed compiler with micro-gas metering.' },
    { icon: <GitMerge size={28} color="var(--cyan)" />, title: 'Bridge Liquidity', desc: 'Lock ETH/BSC assets and mint wrapped THDR equivalents seamlessly.' }
  ]

  return (
    <section className="section" id="how-it-works" style={{ background: 'var(--bg-secondary)' }}>
      <div className="container">
        <motion.div initial={{ opacity: 0, y: 20 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true }}>
          <div className="section-label">🚀 Developer Experience</div>
          <h2 className="heading-lg section-title">How <span className="text-gradient">It Works</span></h2>
          <p className="text-body section-desc">
            A frictionless pipeline from bare-metal code to an interconnected omnichain reality.
          </p>
        </motion.div>

        <div className="steps-container">
          {steps.map((st, i) => (
            <motion.div
              key={i}
              className="glass-card animated-step-card"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.15, duration: 0.5 }}
            >
              <div className="step-icon-wrapper">
                {st.icon}
                <div className="step-glow"></div>
              </div>
              <h3 className="heading-md" style={{ marginBottom: 8 }}>{st.title}</h3>
              <p className="text-body" style={{ fontSize: '0.88rem' }}>{st.desc}</p>

              {i < steps.length - 1 && (
                <div className="step-connector">
                  <motion.div
                    className="connector-line"
                    initial={{ width: 0 }}
                    whileInView={{ width: '100%' }}
                    transition={{ delay: 0.8 + (i * 0.2), duration: 0.6 }}
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

/* ── Comparison ────────────────────────────────────────────────── */
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
        <p className="text-body section-desc reveal">
          See how Thunder stacks up against the industry's leading blockchain networks.
        </p>
        <motion.div
          className="reveal"
          style={{ overflowX: 'auto' }}
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
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
        </motion.div>
      </div>
    </section>
  )
}

/* ── Footer ────────────────────────────────────────────────────── */
function Footer() {
  return (
    <footer className="footer">
      <div className="container">
        <div className="footer-grid">
          <div className="footer-brand">
            <Link to="/" className="nav-logo" style={{ marginBottom: 4 }}>
              <img src="/logo.png" alt="Thunder" />
              <span>Thunder</span>
            </Link>
            <p>The fastest sovereign blockchain. Built with ⚡ in Rust for the next generation of decentralized applications.</p>
          </div>
          <div className="footer-col">
            <h4>Products</h4>
            <Link to="/thunderscan/testnet">ThunderScan</Link>
            <Link to="/coming-soon?product=wallet">Thunder Wallet</Link>
            <Link to="/docs">ThunderScript</Link>
          </div>
          <div className="footer-col">
            <h4>Developers</h4>
            <Link to="/docs">Documentation</Link>
            <a href="https://github.com" target="_blank" rel="noopener noreferrer">GitHub</a>
            <Link to="/docs">API Reference</Link>
          </div>
          <div className="footer-col">
            <h4>Network</h4>
            <Link to="/thunderscan/testnet">Testnet Explorer</Link>
            <Link to="/thunderscan/mainnet">Mainnet Explorer</Link>
            <Link to="/coming-soon?product=faucet">Faucet</Link>
          </div>
        </div>
        <div className="footer-bottom">
          © 2026 Thunder Blockchain. All rights reserved.
        </div>
      </div>
    </footer>
  )
}

/* ── Landing Page Composition ──────────────────────────────────── */
function Landing() {
  return <><Hero /><Features /><Ecosystem /><HowItWorks /><Comparison /></>
}

/* ── Docs ──────────────────────────────────────────────────────── */
function Docs() {
  return (
    <div style={{ paddingTop: 100, minHeight: '80vh', paddingBottom: 100 }}>
      <div className="container">
        <h1 className="heading-xl">ThunderScript <span className="text-gradient">Documentation</span></h1>
        <p className="text-body" style={{ marginTop: 24, fontSize: '1.15rem', maxWidth: 700 }}>
          Welcome to the official developer portal for Thunder Blockchain. Learn how to scaffold, deploy, and interact with smart contracts natively.
        </p>

        <div className="docs-grid" style={{ marginTop: 56, display: 'grid', gridTemplateColumns: 'minmax(220px, 1fr) 3fr', gap: 32 }}>
          <div className="glass-card" style={{ padding: 24 }}>
            <h3 className="heading-md" style={{ marginBottom: 16, fontSize: '1rem' }}>Table of Contents</h3>
            <ul style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              <li><a href="#" className="text-gradient" style={{ fontWeight: 600, fontSize: '0.88rem' }}>1. Intro to ThunderScript</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)', fontSize: '0.88rem' }}>2. Creating a Wallet</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)', fontSize: '0.88rem' }}>3. Writing your First Contract</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)', fontSize: '0.88rem' }}>4. Compiling via CLI</a></li>
              <li><a href="#" style={{ color: 'var(--text-secondary)', fontSize: '0.88rem' }}>5. Deploying to Testnet</a></li>
            </ul>
          </div>
          <div className="glass-card" style={{ padding: 32 }}>
            <h2 style={{ marginBottom: 12 }}>1. Intro to ThunderScript</h2>
            <p style={{ color: 'var(--text-secondary)', lineHeight: 1.7, fontSize: '0.95rem' }}>
              ThunderScript is a stack-based smart contract language optimized specifically for <code style={{ color: 'var(--cyan)' }}>ThunderVM</code>. It forces O(1) memory mappings to prevent infinite loops and recursive depth attacks.
            </p>
            <h3 style={{ marginTop: 24, marginBottom: 8 }}>Example: Setting a basic String</h3>
            <div style={{ background: 'var(--bg-primary)', padding: 16, borderRadius: 10, marginTop: 8, border: '1px solid var(--glass-border)', fontFamily: "'JetBrains Mono', monospace", fontSize: '0.85rem', color: 'var(--cyan)' }}>
              fn init() {'{'}<br />
              &nbsp;&nbsp;PStore "MyFirstContract"<br />
              {'}'}
            </div>
            <h3 style={{ marginTop: 24, marginBottom: 8 }}>Executing via CLI</h3>
            <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem' }}>Use the built-in CLI debugging toolbox to compile and dry-run without spending gas.</p>
            <div style={{ background: 'var(--bg-primary)', padding: 16, borderRadius: 10, marginTop: 8, border: '1px solid var(--glass-border)', fontFamily: "'JetBrains Mono', monospace", fontSize: '0.85rem', color: 'var(--purple)' }}>
              $ cargo run -p thunder-cli -- contract run examples/token.thunder
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

/* ── Coming Soon ───────────────────────────────────────────────── */
function ComingSoon() {
  const query = new URLSearchParams(window.location.search)
  const product = query.get('product') || 'App'

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <motion.div
        className="glass-card"
        style={{ textAlign: 'center', maxWidth: 520, padding: 56 }}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ type: 'spring', damping: 20 }}
      >
        <div style={{ fontSize: '3.5rem', marginBottom: 20 }}>🚀</div>
        <h1 className="heading-lg">Thunder <span className="text-gradient">{product === 'scan' ? 'Scan' : 'Wallet'}</span></h1>
        <p className="text-body" style={{ margin: '20px 0' }}>
          We are currently putting the final touches on {product === 'scan' ? 'the block explorer' : 'the browser extension wallet'}.
          It will be released in Phase 12 of the roadmap.
        </p>
        <Link to="/" className="btn btn-outline" style={{ marginTop: 16 }}>← Back Home</Link>
      </motion.div>
    </div>
  )
}

export { Landing, Docs, ComingSoon, Navbar, Footer };
