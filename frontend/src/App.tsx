import { useState } from 'react'
import { AnimatedBackground } from './components/AnimatedBackground'
import LandingPage from './pages/LandingPage'
import TradingPage from './pages/TradingPage'
import ResultsPage from './pages/ResultsPage'
import './index.css'

function App() {
  const [currentPage, setCurrentPage] = useState<'landing' | 'trading' | 'results'>('landing')
  const [connected, setConnected] = useState(false)

  const handleConnect = () => {
    setConnected(true)
    setCurrentPage('trading')
  }

  return (
    <>
      <AnimatedBackground />
      <div className="min-h-screen relative">
        {/* Noise Texture Overlay */}
        <div className="fixed inset-0 opacity-[0.015] pointer-events-none"
          style={{ backgroundImage: 'url("data:image/svg+xml,%3Csvg viewBox=\'0 0 400 400\' xmlns=\'http://www.w3.org/2000/svg\'%3E%3Cfilter id=\'noiseFilter\'%3E%3CfeTurbulence type=\'fractalNoise\' baseFrequency=\'0.9\' numOctaves=\'4\' /%3E%3C/filter%3E%3Crect width=\'100%25\' height=\'100%25\' filter=\'url(%23noiseFilter)\' /%3E%3C/svg%3E")' }} />

        {/* Header */}
        <header className="relative z-10 backdrop-blur-xl">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between items-center py-6">
              {/* Logo */}
              <div 
                className="flex items-center space-x-3 cursor-pointer"
                onClick={() => setCurrentPage('landing')}
              >
                <div className="text-4xl">⛓</div>
                <h1 className="text-3xl font-bold gradient-text">VEIL</h1>
              </div>

              {/* Navigation */}
              <div className="flex items-center gap-4">
                {currentPage === 'landing' && (
                  <>
                    <button className="px-6 py-3 text-gray-400 hover:text-white transition-colors font-semibold">
                      DOCS
                    </button>
                    <button className="px-6 py-3 text-gray-400 hover:text-white transition-colors font-semibold">
                      GITHUB
                    </button>
                  </>
                )}
                
                {currentPage === 'trading' && (
                  <>
                    <button 
                      onClick={() => setCurrentPage('results')}
                      className="px-6 py-3 glass rounded-xl font-semibold hover:bg-white/10 transition-all"
                    >
                      Leaderboard
                    </button>
                    <button 
                      onClick={() => setCurrentPage('landing')}
                      className="px-6 py-3 glass rounded-xl font-semibold hover:bg-white/10 transition-all"
                    >
                      Home
                    </button>
                  </>
                )}

                {currentPage === 'results' && (
                  <button 
                    onClick={() => setCurrentPage('trading')}
                    className="px-6 py-3 glass rounded-xl font-semibold hover:bg-white/10 transition-all"
                  >
                    Back to Trading
                  </button>
                )}
                
                <button 
                  onClick={handleConnect}
                  className="px-6 py-3 bg-veil-primary text-white rounded-xl font-semibold hover:opacity-90 transition-opacity"
                >
                  {connected ? (
                    <span className="flex items-center gap-2">
                      <div className="w-2 h-2 bg-veil-accent rounded-full animate-pulse" />
                      0x1a2b...3c4d
                    </span>
                  ) : (
                    'connect wallet'
                  )}
                </button>
              </div>
            </div>
          </div>
        </header>

        {/* Main Content */}
        <main className="relative z-10">
          {currentPage === 'landing' && <LandingPage onEnter={handleConnect} />}
          {currentPage === 'trading' && <TradingPage />}
          {currentPage === 'results' && <ResultsPage />}
        </main>

        {/* Footer */}
        <footer className="relative z-10 border-t border-white/5 mt-20">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
            <div className="text-center space-y-4">
              <div className="flex items-center justify-center gap-8 text-sm text-gray-500">
                <span className="flex items-center gap-2">
                  Built on Internet Computer
                </span>
                <span>•</span>
                <span>Powered by vetKeys</span>
                <span>•</span>
                <span>Threshold Signatures</span>
              </div>
              <div className="text-xl font-semibold gradient-text">
                © 2025 Veil. All rights reserved.
              </div>
            </div>
          </div>
        </footer>
      </div>
    </>
  )
}

export default App