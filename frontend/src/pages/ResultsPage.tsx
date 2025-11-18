import { motion } from 'framer-motion'
import ClearingPriceReveal from '../components/results/ClearingPriceReveal'
import OrderBreakdown from '../components/results/OrderBreakdown'
import UserStatsCard from '../components/results/UserStatsCard'

function ResultsPage() {
  const mockOrders = [
    {
      id: '1',
      side: 'Buy' as const,
      amount: 10,
      limitPrice: 3100,
      filled: true,
      surplus: 50,
    },
    {
      id: '2',
      side: 'Buy' as const,
      amount: 5,
      limitPrice: 3000,
      filled: true,
      surplus: 25,
    },
  ]

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 space-y-8">
      {/* Page Title */}
      <motion.h1
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-4xl font-bold text-center gradient-text mb-8"
      >
        Round #42 Results
      </motion.h1>

      {/* Clearing Price Reveal */}
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2 }}
      >
        <ClearingPriceReveal price={3050} volume={150} />
      </motion.div>

      {/* User Stats */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
      >
        <UserStatsCard
          ordersFilled={2}
          ordersSubmitted={2}
          surplusEarned={75}
          rank={3}
          totalPlayers={25}
        />
      </motion.div>

      {/* Order Breakdown */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
      >
        <OrderBreakdown orders={mockOrders} clearingPrice={3050} />
      </motion.div>
    </div>
  )
}

export default ResultsPage