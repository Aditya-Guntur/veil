import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Diamond } from 'lucide-react';
import confetti from 'canvas-confetti';

interface ClearingPriceRevealProps {
  price: number;
  volume: number;
}

export const ClearingPriceReveal: React.FC<ClearingPriceRevealProps> = ({
  price,
  volume,
}) => {
  const [displayPrice, setDisplayPrice] = useState(0);

  useEffect(() => {
    const duration = 1000;
    const steps = 50;
    const increment = price / steps;
    let current = 0;
    let step = 0;

    const interval = setInterval(() => {
      current += increment;
      step++;
      setDisplayPrice(current);

      if (step >= steps) {
        setDisplayPrice(price);
        clearInterval(interval);

        confetti({
          particleCount: 100,
          spread: 70,
          origin: { y: 0.6 },
          colors: ['#6c5ce7', '#00cec9', '#00ff88'],
        });
      }
    }, duration / steps);

    return () => clearInterval(interval);
  }, [price]);

  return (
    <motion.div
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      transition={{ type: 'spring', duration: 0.8 }}
      className="relative"
    >
      {/* Background Glow */}
      <div className="absolute inset-0 -z-10">
        <div className="absolute inset-0 bg-gradient-to-r from-veil-light to-veil-cyan rounded-3xl blur-3xl opacity-30 animate-pulse" />
      </div>

      {/* Main Card */}
      <div className="glass rounded-3xl p-12 text-center">

        {/* Label */}
        <motion.div
          initial={{ y: -20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          className="text-sm text-gray-400 uppercase tracking-wider"
        >
          Clearing Price
        </motion.div>

        {/* Price Display */}
        <motion.div
          initial={{ scale: 0.5, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ delay: 0.4, type: 'spring' }}
          className="relative inline-block"
        >
          {/* Diamond Icon */}
          <motion.div
            animate={{
              rotate: [0, 360],
              scale: [1, 1.2, 1],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: 'easeInOut',
            }}
            className="absolute -top-8 left-1/2 -translate-x-1/2"
          >
            <Diamond className="text-veil-accent" size={40} />
          </motion.div>

          <div className="text-8xl font-bold font-mono gradient-text">
            ${displayPrice.toFixed(2)}
          </div>
        </motion.div>

        {/* Volume Info */}
        <motion.div
          initial={{ y: 20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.6 }}
          className="mt-6 text-lg text-gray-300"
        >
          <span className="font-semibold text-veil-accent">
            {volume.toFixed(2)} ETH
          </span>{' '}
          traded at this price
        </motion.div>

        {/* Chart Visualization */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.8 }}
          className="mt-8"
        >
          <div className="text-xs text-gray-400 mb-2">
            Supply & Demand Intersection
          </div>

          <svg width="100%" height="150" viewBox="0 0 400 150">
            {/* Supply Line */}
            <motion.path
              d="M 50 30 L 200 90 L 350 150"
              stroke="url(#supplyGradient)"
              strokeWidth="3"
              fill="none"
              initial={{ pathLength: 0 }}
              animate={{ pathLength: 1 }}
              transition={{ duration: 1, delay: 0.8 }}
            />

            {/* Demand Line */}
            <motion.path
              d="M 50 150 L 200 90 L 350 30"
              stroke="url(#demandGradient)"
              strokeWidth="3"
              fill="none"
              initial={{ pathLength: 0 }}
              animate={{ pathLength: 1 }}
              transition={{ duration: 1, delay: 0.8 }}
            />

            {/* Intersection Point */}
            <motion.circle
              cx="200"
              cy="90"
              r="8"
              fill="#00ff88"
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ delay: 1.8, type: 'spring' }}
            />

            <defs>
              <linearGradient id="supplyGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="#6c5ce7" />
                <stop offset="100%" stopColor="#a29bfe" />
              </linearGradient>
              <linearGradient id="demandGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="#00cec9" />
                <stop offset="100%" stopColor="#0984e3" />
              </linearGradient>
            </defs>
          </svg>
        </motion.div>
      </div>
    </motion.div>
  );
};

export default ClearingPriceReveal;