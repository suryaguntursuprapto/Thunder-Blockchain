import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import './App.css'
import { Landing, Docs, ComingSoon, Navbar, Footer } from './pages/Landing'
import { ThunderScanTestnet, ThunderScanMainnet, ScanNavbar, ScanFooter } from './pages/Explorer'

// ── Main App ─────────────────────────────────────────────────── */
function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<><Navbar /><Landing /><Footer /></>} />
        <Route path="/docs" element={<><Navbar /><Docs /><Footer /></>} />
        <Route path="/coming-soon" element={<><Navbar /><ComingSoon /><Footer /></>} />

        <Route path="/thunderscan/testnet" element={<><ScanNavbar /><ThunderScanTestnet /><ScanFooter /></>} />
        <Route path="/thunderscan/mainnet" element={<><ScanNavbar /><ThunderScanMainnet /><ScanFooter /></>} />
      </Routes>
    </Router>
  )
}

export default App
