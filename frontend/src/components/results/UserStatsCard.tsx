import { motion } from 'framer-motion'
import { TrendingUp, CheckCircle, DollarSign } from 'lucide-react'

interface UserStatsCardProps {
  ordersFilled: number
  ordersSubmitted: number
  surplusEarned: number
  rank: number
  totalPlayers: number
}

const UserStatsCard = ({
  ordersFilled,
  ordersSubmitted,
  surplusEarned,
  rank,
  totalPlayers,
}: UserStatsCardProps) => {
  const fillRate = (ordersFilled / ordersSubmitted) * 100
  const isWinner = surplusEarned > 0

  return (
    <div className={`glass-strong rounded-3xl p-8 ${isWinner ? 'glow-box' : ''}`}>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-bold gradient-text">Personal stats</h3>
          {isWinner && (
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ type: 'spring', delay: 0.2 }}
            >
              <span className="text-3xl">🏆</span>
            </motion.div>
          )}
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-3 gap-6">
          {/* Orders Filled */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="text-center p-6 bg-black/30 rounded-xl"
          >
            <CheckCircle className="mx-auto mb-2 text-green-400" size={32} />
            <div className="text-3xl font-bold gradient-text">
              {ordersFilled}/{ordersSubmitted}
            </div>
            <div className="text-xs text-gray-400 mt-1">Orders Filled</div>
            <div className={`text-sm font-semibold mt-1 ${
              fillRate === 100 ? 'text-green-400' : 'text-yellow-400'
            }`}>
              {fillRate.toFixed(0)}%
            </div>
          </motion.div>

          {/* Surplus Earned */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="text-center p-6 bg-black/30 rounded-xl"
          >
            <DollarSign className={`mx-auto mb-2 ${
              surplusEarned > 0 ? 'text-green-400' : 'text-gray-400'
            }`} size={32} />
            <div className={`text-3xl font-bold ${
              surplusEarned > 0 ? 'text-green-400' : 'text-gray-400'
            }`}>
              ${Math.abs(surplusEarned).toFixed(2)}
            </div>
            <div className="text-xs text-gray-400 mt-1">Surplus Earned</div>
          </motion.div>

          {/* Rank */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="text-center p-6 bg-black/30 rounded-xl"
          >
            <TrendingUp className="mx-auto mb-2 text-veil-cyan" size={32} />
            <div className="text-3xl font-bold gradient-text">
              #{rank}
            </div>
            <div className="text-xs text-gray-400 mt-1">Rank</div>
            <div className="text-sm text-gray-300 mt-1">
              of {totalPlayers} players
            </div>
          </motion.div>
        </div>
      </div>
    </div>
  )
}

export default UserStatsCard