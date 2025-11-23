import React from 'react';
import { motion } from 'framer-motion';
import { Card } from '../ui/Card';
import { TrendingUp, TrendingDown, CheckCircle, XCircle } from 'lucide-react';

interface Order {
  id: string;
  side: 'Buy' | 'Sell';
  amount: number;
  limitPrice: number;
  filled: boolean;
  surplus: number;
}

interface OrderBreakdownProps {
  orders: Order[];
  clearingPrice: number;
}

export const OrderBreakdown: React.FC<OrderBreakdownProps> = ({
  orders,
  clearingPrice,
}) => {
  return (
    <Card>
      <h3 className="text-2xl font-bold mb-4">Order Breakdown</h3>
      
      <div className="space-y-3">
        {orders.map((order, index) => (
          <motion.div
            key={order.id}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className={`p-4 rounded-lg ${
              order.filled 
                ? 'bg-green-500/10 border border-green-500/30' 
                : 'bg-gray-800/50 border border-gray-700'
            }`}
          >
            <div className="flex items-center justify-between">
              {/* Order Info */}
              <div className="flex items-center gap-3">
                {/* Icon */}
                <div className={`p-2 rounded-lg ${
                  order.side === 'Buy' ? 'bg-green-500/20' : 'bg-red-500/20'
                }`}>
                  {order.side === 'Buy' ? (
                    <TrendingUp className="text-green-400" size={20} />
                  ) : (
                    <TrendingDown className="text-red-400" size={20} />
                  )}
                </div>

                {/* Details */}
                <div>
                  <div className="font-semibold">
                    {order.side} {order.amount} ETH
                  </div>
                  <div className="text-sm text-gray-400">
                    Your Limit: ${order.limitPrice.toFixed(2)}
                    {order.filled && (
                      <span className="text-gray-300">
                        {' '}→ Filled at ${clearingPrice.toFixed(2)}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              {/* Status & Surplus */}
              <div className="text-right">
                {order.filled ? (
                  <>
                    <div className="flex items-center gap-2 text-green-400 font-semibold">
                      <CheckCircle size={18} />
                      Filled
                    </div>
                    <div className="text-lg font-bold text-green-400 mt-1">
                      +${order.surplus.toFixed(2)}
                    </div>
                  </>
                ) : (
                  <div className="flex items-center gap-2 text-gray-400">
                    <XCircle size={18} />
                    Not Filled
                  </div>
                )}
              </div>
            </div>

            {/* Explanation */}
            {order.filled && (
              <div className="mt-2 text-xs text-gray-400 border-t border-gray-700 pt-2">
                {order.side === 'Buy' ? (
                  <>
                    You were willing to pay up to ${order.limitPrice.toFixed(2)}, 
                    but only paid ${clearingPrice.toFixed(2)} — 
                    saved ${(order.limitPrice - clearingPrice).toFixed(2)} per ETH!
                  </>
                ) : (
                  <>
                    You were willing to sell at ${order.limitPrice.toFixed(2)}, 
                    but sold at ${clearingPrice.toFixed(2)} — 
                    earned ${(clearingPrice - order.limitPrice).toFixed(2)} extra per ETH!
                  </>
                )}
              </div>
            )}
          </motion.div>
        ))}
      </div>

      {/* Total Summary */}
      <div className="mt-6 p-4 glass rounded-lg">
        <div className="flex items-center justify-between">
          <div className="text-sm text-gray-400">
            Total Surplus This Round
          </div>
          <div className="text-2xl font-bold gradient-text">
            +${orders.reduce((sum, o) => sum + o.surplus, 0).toFixed(2)}
          </div>
        </div>
      </div>
    </Card>
  );
};

export default OrderBreakdown;