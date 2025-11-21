import { motion } from 'framer-motion'
import { Lock, Shield, Zap } from 'lucide-react'

interface LandingPageProps {
  onEnter: () => void
}

function LandingPage({ onEnter }: LandingPageProps) {
  return (
    <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-20 min-h-[calc(100vh-200px)] flex items-center">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        className="glass-strong rounded-3xl p-12 md:p-16 text-center glow-box w-full relative overflow-hidden"
      >
        {/* Subtle gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-br from-white/[0.02] to-transparent pointer-events-none" />
        
        {/* Content */}
        <div className="relative z-10">
          {/* Title */}
          <motion.h2 
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="text-5xl md:text-7xl font-bold mb-4 gradient-text"
          >
            Dark Pool, Fair Price
          </motion.h2>
          
          <motion.p 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.3 }}
            className="text-gray-400 mb-8 text-lg md:text-xl max-w-2xl mx-auto leading-relaxed"
          >
            Encrypt&nbsp;&nbsp;.&nbsp;&nbsp;Submit&nbsp;&nbsp;.&nbsp;&nbsp;Reveal
          </motion.p>

          <motion.p 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.3 }}
            className="text-gray-400 mb-8 text-lg md:text-xl max-w-2xl mx-auto leading-relaxed"
          >
            WHERE STRATEGY BEATS SPEED
          </motion.p>

          {/* Mempool Chess Hook */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.4 }}
            className="mb-16 inline-flex items-center gap-2 px-6 py-3 glass rounded-full border border-white/10"
          >
            <Shield className="text-veil-accent" size={18} />
            <span className="text-sm text-gray-300 font-semibold">
              Win at mempool chess - Play your move in secret
            </span>
            <Zap className="text-veil-accent" size={18} />
          </motion.div>

          {/* Visual Section - Technical Encryption Flow */}
          <motion.div
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ delay: 0.5, duration: 0.8 }}
            className="mb-16 py-12 relative"
          >
            {/* Technical grid background */}
            <div className="absolute inset-0 opacity-5">
              <div className="grid grid-cols-12 gap-1 h-full">
                {[...Array(144)].map((_, i) => (
                  <motion.div
                    key={i}
                    className="border border-white/20"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: [0.1, 0.3, 0.1] }}
                    transition={{
                      duration: 4,
                      repeat: Infinity,
                      delay: i * 0.02,
                    }}
                  />
                ))}
              </div>
            </div>

            {/* Main Tech Flow Visualization */}
            <div className="relative max-w-4xl mx-auto">
              {/* Encrypted Input Orders */}
              <div className="flex justify-between items-start mb-8">
                {[
                  { side: 'BUY', amount: '5.2 ckETH', price: '█████', color: 'text-green-400' },
                  { side: 'SELL', amount: '3.8 ckETH', price: '█████', color: 'text-red-400' },
                  { side: 'BUY', amount: '10.5 ckETH', price: '█████', color: 'text-green-400' },
                ].map((order, i) => (
                  <motion.div
                    key={`input-${i}`}
                    initial={{ x: -50, opacity: 0 }}
                    animate={{ x: 0, opacity: 1 }}
                    transition={{ delay: 0.6 + i * 0.1, duration: 0.5 }}
                    className="flex-1 mx-2"
                  >
                    <div className="glass p-4 rounded-xl border border-white/10 relative">
                      <div>
                        <div className="flex items-center justify-between mb-3">
                          <div className="flex items-center gap-2">
                            <Lock className="text-gray-500" size={14} />
                            <span className="text-xs text-gray-600 font-mono">ORDER_{i + 1}</span>
                          </div>
                          <span className={`text-xs font-bold ${order.color}`}>{order.side}</span>
                        </div>
                        
                        <div className="space-y-2 text-xs">
                          <div className="flex justify-between items-center">
                            <span className="text-gray-600">Amount:</span>
                            <span className="font-mono text-white">{order.amount}</span>
                          </div>
                          <div className="flex justify-between items-center">
                            <span className="text-gray-600">Price:</span>
                            <span className="font-mono text-gray-500">{order.price}</span>
                          </div>
                          <div className="flex justify-between items-center">
                            <span className="text-gray-600">Status:</span>
                            <span className="flex items-center gap-1 text-yellow-400">
                              <motion.div
                                animate={{ opacity: [0.3, 1, 0.3] }}
                                transition={{ duration: 1.5, repeat: Infinity }}
                                className="w-1.5 h-1.5 bg-yellow-400 rounded-full"
                              />
                              ENCRYPTED
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </motion.div>
                ))}
              </div>

              {/* Flow arrows down */}
              <div className="flex justify-around mb-4">
                {[0, 1, 2].map((i) => (
                  <motion.div
                    key={`arrow-down-${i}`}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: [0.3, 0.8, 0.3], y: [0, 5, 0] }}
                    transition={{ duration: 1.5, repeat: Infinity, delay: i * 0.2 }}
                    className="text-2xl text-gray-700"
                  >
                    ↓
                  </motion.div>
                ))}
              </div>

              {/* Central Processing Unit - vetKeys */}
              <motion.div
                initial={{ scale: 0.8, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                transition={{ delay: 0.9, type: "spring" }}
                className="relative mb-6"
              >
                <div className="glass-strong p-8 rounded-2xl border border-white/20 relative overflow-hidden">
                  {/* Animated circuit lines */}
                  <div className="absolute inset-0 opacity-20">
                    {[...Array(6)].map((_, i) => (
                      <motion.div
                        key={`circuit-${i}`}
                        className="absolute h-px bg-gradient-to-r from-transparent via-white to-transparent"
                        style={{ 
                          top: `${20 + i * 15}%`,
                          left: 0,
                          right: 0
                        }}
                        animate={{ 
                          opacity: [0.2, 0.6, 0.2],
                          scaleX: [0.8, 1, 0.8]
                        }}
                        transition={{ 
                          duration: 2,
                          repeat: Infinity,
                          delay: i * 0.3
                        }}
                      />
                    ))}
                  </div>

                  {/* Central content */}
                  <div className="relative z-10 text-center">
                    <div className="flex items-center justify-center gap-3 mb-3">
                      <Shield className="text-gray-400" size={24} />
                      <span className="text-sm font-mono text-gray-400 tracking-wider">vetKeys ENCRYPTION</span>
                      <Shield className="text-gray-400" size={24} />
                    </div>
                    
                    {/* Hexadecimal display */}
                    <div className="flex justify-center gap-1 mb-3 font-mono text-xs text-gray-600">
                      {['0x', 'A7', '4F', '92', 'E3', 'B8'].map((hex, i) => (
                        <motion.span
                          key={i}
                          animate={{ opacity: [0.3, 1, 0.3] }}
                          transition={{ duration: 1.5, repeat: Infinity, delay: i * 0.1 }}
                        >
                          {hex}
                        </motion.span>
                      ))}
                    </div>

                    {/* Processing indicator */}
                    <div className="flex items-center justify-center gap-2">
                      <motion.div
                        animate={{ rotate: 360 }}
                        transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
                        className="w-4 h-4 border-2 border-gray-600 border-t-white rounded-full"
                      />
                      <span className="text-xs text-gray-500">PROCESSING BATCH</span>
                    </div>
                  </div>
                </div>

                {/* Side indicators */}
                <motion.div
                  animate={{ opacity: [0.5, 1, 0.5] }}
                  transition={{ duration: 1.5, repeat: Infinity }}
                  className="absolute -left-3 top-1/2 -translate-y-1/2 w-2 h-2 bg-green-400 rounded-full shadow-lg shadow-green-400/50"
                />
                <motion.div
                  animate={{ opacity: [0.5, 1, 0.5] }}
                  transition={{ duration: 1.5, repeat: Infinity, delay: 0.5 }}
                  className="absolute -right-3 top-1/2 -translate-y-1/2 w-2 h-2 bg-green-400 rounded-full shadow-lg shadow-green-400/50"
                />
              </motion.div>

              {/* Flow arrows up */}
              <div className="flex justify-around mb-4">
                {[0, 1, 2].map((i) => (
                  <motion.div
                    key={`arrow-up-${i}`}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: [0.3, 0.8, 0.3], y: [0, -5, 0] }}
                    transition={{ duration: 1.5, repeat: Infinity, delay: i * 0.2 + 0.5 }}
                    className="text-2xl text-gray-700"
                  >
                    ↓
                  </motion.div>
                ))}
              </div>

              {/* Output - Simultaneous Reveal */}
              <motion.div
                initial={{ y: 50, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 1.1, duration: 0.6 }}
                className="glass p-6 rounded-2xl border border-white/20 bg-white/5"
              >
                <div className="text-center">
                  <div className="flex items-center justify-center gap-2 mb-3">
                    <motion.div
                      animate={{ scale: [1, 1.2, 1] }}
                      transition={{ duration: 2, repeat: Infinity }}
                    >
                      <Zap className="text-gray-400" size={20} />
                    </motion.div>
                    <span className="text-sm font-mono text-gray-400">SIMULTANEOUS REVEAL</span>
                    <motion.div
                      animate={{ scale: [1, 1.2, 1] }}
                      transition={{ duration: 2, repeat: Infinity, delay: 0.5 }}
                    >
                      <Zap className="text-gray-400" size={20} />
                    </motion.div>
                  </div>
                  <div className="text-xs text-gray-600 font-mono">
                    00:00:00 • ALL ORDERS VISIBLE • CLEARING PRICE CALCULATED
                  </div>
                </div>
              </motion.div>
            </div>
          </motion.div>

          {/* Enter Button - Redesigned */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7 }}
            className="space-y-6"
          >
            <button
              onClick={onEnter}
              className="group relative px-20 py-4 bg-white text-black rounded-xl font-bold text-lg overflow-hidden transition-all duration-300 hover:scale-[1.02] border-2 border-white/20 hover:border-white shadow-lg hover:shadow-2xl"
            >
              <span className="relative z-10 tracking-wide">
                ENTER VEIL
              </span>
              
              {/* Hover gradient effect */}
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-gray-100 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
            </button>

            {/* Note */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.9 }}
              className="text-xs text-gray-500 flex items-center justify-center gap-2"
            >
              <div className="w-1 h-1 bg-gray-600 rounded-full" />
                -----------------------
              <div className="w-1 h-1 bg-gray-600 rounded-full" />
            </motion.div>
          </motion.div>
        </div>
      </motion.div>
    </div>
  )
}

export default LandingPage