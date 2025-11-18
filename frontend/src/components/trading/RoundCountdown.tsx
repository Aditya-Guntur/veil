import { motion } from 'framer-motion'
import { Clock, Lock, TrendingUp } from 'lucide-react'
import { useCountdown } from '../../hooks/useCountdown'

interface RoundCountdownProps {
  endTime: number
  roundId: number
}

const RoundCountdown = ({ endTime, roundId }: RoundCountdownProps) => {
  const { minutes, seconds, progress, isUrgent } = useCountdown(endTime)

  return (
    <motion.div
      initial={{ scale: 0.9, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      className={`glass-strong rounded-3xl p-12 text-center glow-box ${
        isUrgent ? 'ring-4 ring-red-500 animate-pulse' : ''
      }`}
    >
      <div className="text-sm text-gray-400 uppercase tracking-wider mb-4">
        Round #{roundId}
      </div>
      
      <div className="flex items-center justify-center gap-6 mb-6">
        <Clock className="text-veil-accent animate-pulse" size={40} />
        <div className="text-8xl font-bold font-mono gradient-text tabular-nums">
          {String(minutes).padStart(2, '0')}:{String(seconds).padStart(2, '0')}
        </div>
      </div>
      
      <div className="relative w-full h-4 bg-black/50 rounded-full overflow-hidden mb-4">
        <motion.div
          className={`absolute inset-y-0 left-0 rounded-full ${
            isUrgent
              ? 'bg-gradient-to-r from-red-500 to-orange-500'
              : 'bg-gradient-to-r from-veil-accent via-veil-cyan to-veil-light'
          }`}
          style={{ width: `${progress}%` }}
          transition={{ duration: 0.3 }}
        />
      </div>
      
      <div className="flex items-center justify-center gap-6 text-sm text-gray-400">
        <span className="flex items-center gap-2">
          <Lock size={16} />
          4 orders encrypted
        </span>
        <span className="flex items-center gap-2">
          <TrendingUp size={16} />
          {isUrgent ? '⚠ REVEALING NOW!' : 'Revealing soon...'}
        </span>
      </div>
    </motion.div>
  )
}

export default RoundCountdown
