import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';
import RoundCountdown from '../components/trading/RoundCountdown';
import OrderForm from '../components/trading/OrderForm';
import EncryptedOrderCard from '../components/trading/EncryptedOrderCard';
import UserStatsCard from '../components/results/UserStatsCard';
import { canisterService } from '../services/canister';
import type { State, OrderBookSummary, UserStats } from '../services/canister';
import { useAuth } from '../hooks/useAuth';
import { MOCK_USERS } from '../utils/mockUsers';
import { Principal } from "@dfinity/principal";

function TradingPage() {
  const { principal } = useAuth();
  const [roundState, setRoundState] = useState<State | null>(null);
  const [orderBook, setOrderBook] = useState<OrderBookSummary | null>(null);
  const [userStats, setUserStats] = useState<UserStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { setMockUser } = useAuth();

  useEffect(() => {
    loadData();
    
    // Poll every 5 seconds
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, [principal]);

  const loadData = async () => {
    try {
      // Load round state
      const state = await canisterService.getRoundState();
      setRoundState(state);

      // Load order book summary
      const book = await canisterService.getOrderBookSummary();
      setOrderBook(book);

      // Load user stats if authenticated
      if (principal) {
        try {
          const p = Principal.fromText(principal);
          const stats = await canisterService.getUserStats(p);
          setUserStats(stats);
        } catch (e) {
          console.error("Invalid principal:", principal);
        }
      }
    } catch (error) {
      console.error('Failed to load data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="text-center text-gray-400">
          <div className="animate-spin w-8 h-8 border-2 border-veil-accent border-t-transparent rounded-full mx-auto mb-4" />
          Loading...
        </div>
      </div>
    );
  }

  const roundEndTime = roundState 
    ? Number(roundState.round_start_time) / 1_000_000 + Number(roundState.round_duration_ns) / 1_000_000
    : Date.now() + 60000;

  const totalOrders = orderBook 
    ? Number(orderBook.buy_orders) + Number(orderBook.sell_orders)
    : 0;

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 space-y-8">


      {/* Countdown Timer */}
      <RoundCountdown
        endTime={roundEndTime}
        roundId={roundState ? Number(roundState.round_id) : 0}
        state={roundState ? canisterService.getRoundStateString(roundState.round_state) : 'Unknown'}
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

          {/* Order Book Stats */}
          <div className="grid grid-cols-2 gap-4 mb-6">
            <div className="glass p-4 rounded-xl">
              <div className="text-sm text-gray-400 mb-1">Buy Orders</div>
              <div className="text-2xl font-bold text-green-400">
                {orderBook ? Number(orderBook.buy_orders) : 0}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {orderBook 
                  ? canisterService.formatAmount(orderBook.total_buy_volume)
                  : '0.0000'} ETH
              </div>
            </div>

            <div className="glass p-4 rounded-xl">
              <div className="text-sm text-gray-400 mb-1">Sell Orders</div>
              <div className="text-2xl font-bold text-red-400">
                {orderBook ? Number(orderBook.sell_orders) : 0}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {orderBook 
                  ? canisterService.formatAmount(orderBook.total_sell_volume)
                  : '0.0000'} ETH
              </div>
            </div>
          </div>

          {/* Encrypted Orders List */}
          <div className="space-y-4 max-h-[400px] overflow-y-auto">
            {totalOrders === 0 ? (
              <div className="text-center py-12 text-gray-500">
                <div className="text-4xl mb-4">📦</div>
                <p>No orders yet this round</p>
                <p className="text-sm mt-2">Be the first to submit!</p>
              </div>
            ) : (
              // Show encrypted order placeholders
              Array.from({ length: totalOrders }).map((_, i) => (
                <motion.div
                  key={i}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: i * 0.05 }}
                >
                  <EncryptedOrderCard
                    orderId={`order_${i + 1}`}
                    side={i % 2 === 0 ? 'Buy' : 'Sell'}
                    isOwn={false}
                  />
                </motion.div>
              ))
            )}
          </div>

          <div className="mt-6 text-center">
            <div className="inline-flex items-center gap-2 text-gray-400">
              <div className="w-2 h-2 bg-veil-accent rounded-full animate-pulse" />
              <span className="text-sm">
                {roundState && 'Active' in roundState.round_state
                  ? 'Accepting encrypted orders...'
                  : 'Waiting for round to start...'}
              </span>
            </div>
          </div>
        </motion.div>
      </div>

      {/* Your Stats */}
      {principal && userStats && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
        >
          <UserStatsCard
            ordersFilled={Number(userStats.filled_orders)}
            ordersSubmitted={Number(userStats.total_orders)}
            surplusEarned={Number(userStats.total_surplus) / 100} // Convert cents to dollars
            rank={1} // TODO: Calculate rank from leaderboard
            totalPlayers={10} // TODO: Get from platform stats
          />
        </motion.div>
      )}
    </div>
  );
}

export default TradingPage;