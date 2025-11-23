import React from 'react';
import { motion } from 'framer-motion';
import { Lock, TrendingUp, TrendingDown } from 'lucide-react';

interface EncryptedOrderCardProps {
  orderId: string;
  side: 'Buy' | 'Sell';
  isOwn: boolean;
}

export const EncryptedOrderCard: React.FC<EncryptedOrderCardProps> = ({
  orderId,
  side,
  isOwn,
}) => {
  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      className={`
        glass p-4 rounded-lg shimmer
        ${isOwn ? 'ring-2 ring-accent/50' : ''}
      `}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          {/* Lock Icon */}
          <div className="p-2 bg-primary-dark rounded-lg">
            <Lock className="text-accent" size={20} />
          </div>

          {/* Order Info */}
          <div>
            <div className="flex items-center gap-2">
              {side === 'Buy' ? (
                <TrendingUp className="text-success" size={16} />
              ) : (
                <TrendingDown className="text-error" size={16} />
              )}
              <span className="font-semibold">{side} Order</span>
              {isOwn && (
                <span className="text-xs bg-accent/20 text-accent px-2 py-0.5 rounded">
                  Yours
                </span>
              )}
            </div>
            <div className="text-xs text-text-secondary font-mono">
              ID: {orderId.slice(0, 8)}...
            </div>
          </div>
        </div>

        {/* Encrypted Badge */}
        <div className="text-xs text-text-secondary">
          🔒 Encrypted
        </div>
      </div>
    </motion.div>
  );
};

export default EncryptedOrderCard; 