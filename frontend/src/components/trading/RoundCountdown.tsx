import { motion } from 'framer-motion';
import { Clock, Lock, TrendingUp, Eye, Zap } from 'lucide-react';
import { useCountdown } from '../../hooks/useCountdown';

interface RoundCountdownProps {
  endTime: number;
  roundId: number;
  state: string;
}

const RoundCountdown = ({ endTime, roundId, state }: RoundCountdownProps) => {
  const { minutes, seconds, progress, isUrgent } = useCountdown(endTime);

  // Determine icon and message based on state
  const getStateInfo = () => {
    switch (state) {
      case 'Active':
        return {
          icon: <Lock className="text-veil-accent animate-pulse" size={40} />,
          message: 'Submitting encrypted orders',
          color: 'from-veil-accent via-veil-cyan to-veil-light',
        };
      case 'Revealing':
        return {
          icon: <Eye className="text-yellow-400 animate-pulse" size={40} />,
          message: 'Revealing orders...',
          color: 'from-yellow-500 to-orange-500',
        };
      case 'Clearing':
        return {
          icon: <Zap className="text-purple-400 animate-pulse" size={40} />,
          message: 'Calculating clearing price...',
          color: 'from-purple-500 to-pink-500',
        };
      case 'Executing':
        return {
          icon: <TrendingUp className="text-green-400 animate-pulse" size={40} />,
          message: 'Executing settlements...',
          color: 'from-green-500 to-emerald-500',
        };
      case 'Completed':
        return {
          icon: <span className="text-4xl">✅</span>,
          message: 'Round completed! Starting next round...',
          color: 'from-green-400 to-blue-400',
        };
      case 'Pending':
        return {
          icon: <Clock className="text-gray-400" size={40} />,
          message: 'Waiting for round to start...',
          color: 'from-gray-500 to-gray-600',
        };
      default:
        return {
          icon: <Clock className="text-veil-accent" size={40} />,
          message: 'Loading...',
          color: 'from-veil-accent to-veil-cyan',
        };
    }
  };

  const stateInfo = getStateInfo();

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
        {stateInfo.icon}
        
        {state === 'Active' && (
          <div className="text-8xl font-bold font-mono gradient-text tabular-nums">
            {String(minutes).padStart(2, '0')}:{String(seconds).padStart(2, '0')}
          </div>
        )}
      </div>

      {state === 'Active' && (
        <div className="relative w-full h-4 bg-black/50 rounded-full overflow-hidden mb-4">
          <motion.div
            className={`absolute inset-y-0 left-0 rounded-full ${
              isUrgent
                ? 'bg-gradient-to-r from-red-500 to-orange-500'
                : `bg-gradient-to-r ${stateInfo.color}`
            }`}
            style={{ width: `${progress}%` }}
            transition={{ duration: 0.3 }}
          />
        </div>
      )}

      <div className="flex items-center justify-center gap-2 text-sm text-gray-400">
        <span>{stateInfo.message}</span>
      </div>

      {/* State-specific warnings */}
      {state === 'Active' && isUrgent && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="mt-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg"
        >
          <span className="text-red-400 font-semibold">
            ⚠️ Round ending soon! Submit your orders now!
          </span>
        </motion.div>
      )}

      {state === 'Revealing' && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="mt-4 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg"
        >
          <span className="text-yellow-400 text-sm">
            🔓 Decrypting all orders simultaneously...
          </span>
        </motion.div>
      )}

      {state === 'Clearing' && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="mt-4 p-3 bg-purple-500/10 border border-purple-500/30 rounded-lg"
        >
          <span className="text-purple-400 text-sm">
            ⚖️ Finding fair clearing price for all traders...
          </span>
        </motion.div>
      )}
    </motion.div>
  );
};

export default RoundCountdown;