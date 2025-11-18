import { motion } from 'framer-motion'
import RoundCountdown from '../components/trading/RoundCountdown'
import OrderForm from '../components/trading/OrderForm'
import EncryptedOrderCard from '../components/trading/EncryptedOrderCard'
import UserStatsCard from '../components/results/UserStatsCard'

function TradingPage() {
  const mockOrders = [
    { orderId: '1a2b3c4d', side: 'Buy' as const, isOwn: true },
    { orderId: '5e6f7g8h', side: 'Sell' as const, isOwn: false },
    { orderId: '9i0j1k2l', side: 'Buy' as const, isOwn: false },
    { orderId: '3m4n5o6p', side: 'Buy' as const, isOwn: false },
  ]

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 space-y-8">
      {/* Countdown Timer */}
      <RoundCountdown 
        endTime={Date.now() + 4 * 60 * 1000 + 23 * 1000} 
        roundId={42} 
      />

      {/* Trading Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Order Form */}
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
        >
          <OrderForm />
        </motion.div>

        {/* Order Book */}
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          className="glass rounded-3xl p-8"
        >
          <h3 className="text-3xl font-bold mb-8 gradient-text">Orderbook</h3>
          <div className="space-y-4 max-h-[500px] overflow-y-auto">
            {mockOrders.map((order, i) => (
              <motion.div
                key={order.orderId}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.1 }}
              >
                <EncryptedOrderCard
                  orderId={order.orderId}
                  side={order.side}
                  isOwn={order.isOwn}
                />
              </motion.div>
            ))}
          </div>
          <div className="mt-6 text-center">
            <div className="inline-flex items-center gap-2 text-gray-400">
              <div className="w-2 h-2 bg-veil-accent rounded-full animate-pulse" />
              <span className="text-sm">Waiting for round to complete...</span>
            </div>
          </div>
        </motion.div>
      </div>

      {/* Your Stats */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
      >
        <UserStatsCard
          ordersFilled={2}
          ordersSubmitted={2}
          surplusEarned={0}
          rank={1}
          totalPlayers={10}
        />
      </motion.div>
    </div>
  )
}

export default TradingPage
