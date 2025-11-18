import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

interface RevealAnimationProps {
  onComplete: () => void;
  orderCount: number;
}

export const RevealAnimation: React.FC<RevealAnimationProps> = ({
  onComplete,
  orderCount,
}) => {
  const [decryptedCount, setDecryptedCount] = useState(0);
  const [stage, setStage] = useState<'decrypting' | 'clearing' | 'complete'>('decrypting');

  useEffect(() => {
    // Stage 1: Decrypt orders one by one
    const decryptInterval = setInterval(() => {
      setDecryptedCount(prev => {
        if (prev >= orderCount) {
          clearInterval(decryptInterval);
          setStage('clearing');
          return prev;
        }
        return prev + 1;
      });
    }, 300);

    // Stage 2: After all decrypted, show clearing
    const clearingTimeout = setTimeout(() => {
      setStage('complete');
      setTimeout(onComplete, 1000);
    }, orderCount * 300 + 2000);

    return () => {
      clearInterval(decryptInterval);
      clearTimeout(clearingTimeout);
    };
  }, [orderCount, onComplete]);

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-lg"
      >
        <div className="text-center space-y-8">
          {stage === 'decrypting' && (
            <>
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                className="text-6xl"
              >
                🔓
              </motion.div>
              
              <div className="space-y-4">
                <h2 className="text-4xl font-bold gradient-text">
                  Decrypting Orders...
                </h2>
                
                <div className="flex items-center justify-center gap-2">
                  <div className="text-2xl font-bold text-veil-accent">
                    {decryptedCount}
                  </div>
                  <div className="text-2xl text-gray-400">
                    / {orderCount}
                  </div>
                </div>

                {/* Progress Bar */}
                <div className="w-64 h-3 bg-gray-800 rounded-full overflow-hidden mx-auto">
                  <motion.div
                    className="h-full bg-gradient-to-r from-veil-light to-veil-cyan"
                    initial={{ width: 0 }}
                    animate={{ width: `${(decryptedCount / orderCount) * 100}%` }}
                    transition={{ duration: 0.3 }}
                  />
                </div>
              </div>

              {/* Particle effect */}
              <div className="relative h-20">
                {[...Array(5)].map((_, i) => (
                  <motion.div
                    key={i}
                    className="absolute left-1/2 top-0 w-2 h-2 bg-veil-accent rounded-full"
                    animate={{
                      x: [0, Math.random() * 200 - 100],
                      y: [0, Math.random() * 100],
                      opacity: [1, 0],
                    }}
                    transition={{
                      duration: 1,
                      repeat: Infinity,
                      delay: i * 0.2,
                    }}
                  />
                ))}
              </div>
            </>
          )}

          {stage === 'clearing' && (
            <motion.div
              initial={{ scale: 0, rotate: -180 }}
              animate={{ scale: 1, rotate: 0 }}
              transition={{ type: 'spring', duration: 0.8 }}
              className="space-y-4"
            >
              <div className="text-6xl">⚖️</div>
              <h2 className="text-4xl font-bold gradient-text">
                Calculating Clearing Price...
              </h2>
              <div className="flex items-center justify-center gap-2">
                <div className="w-3 h-3 bg-veil-accent rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                <div className="w-3 h-3 bg-veil-cyan rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                <div className="w-3 h-3 bg-veil-light rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
              </div>
            </motion.div>
          )}

          {stage === 'complete' && (
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ type: 'spring' }}
              className="text-6xl"
            >
              ✨
            </motion.div>
          )}
        </div>
      </motion.div>
    </AnimatePresence>
  );
};