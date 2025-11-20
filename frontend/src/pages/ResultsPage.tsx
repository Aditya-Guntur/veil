import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';
import ClearingPriceReveal from '../components/results/ClearingPriceReveal';
import OrderBreakdown from '../components/results/OrderBreakdown';
import UserStatsCard from '../components/results/UserStatsCard';
import { canisterService} from '../services/canister';
import type { ClearingResult, UserStats, LeaderboardEntry } from '../services/canister';

import { useAuth } from '../hooks/useAuth';

function ResultsPage() {
  const { principal } = useAuth();
  const [clearingResult, setClearingResult] = useState<ClearingResult | null>(null);
  const [userStats, setUserStats] = useState<UserStats | null>(null);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadResults();
  }, [principal]);

  const loadResults = async () => {
    try {
      // Get current round result
      const result = await canisterService.getCurrentRoundResult();
      setClearingResult(result);

      // Get user stats if authenticated
      if (principal) {
        const stats = await canisterService.getUserStats(principal);
        setUserStats(stats);
      }

      // Get leaderboard
      if (result) {
        const board = await canisterService.getRoundLeaderboard(result.round_id);
        setLeaderboard(board);
      }
    } catch (error) {
      console.error('Failed to load results:', error);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="text-center text-gray-400">
          <div className="animate-spin w-8 h-8 border-2 border-veil-accent border-t-transparent rounded-full mx-auto mb-4" />
          Loading results...
        </div>
      </div>
    );
  }

  if (!clearingResult) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="text-center">
          <div className="text-6xl mb-4">⏳</div>
          <h2 className="text-3xl font-bold gradient-text mb-4">
            Round in Progress
          </h2>
          <p className="text-gray-400">
            Results will appear once the round is completed and cleared
          </p>
        </div>
      </div>
    );
  }

  // Calculate user's position in leaderboard
  const userRank = principal 
    ? leaderboard.findIndex(entry => entry.user.toText() === principal.toText()) + 1
    : 0;

  // Get user's orders from clearing result
  const userOrders = principal && userStats
    ? clearingResult.matches
        .filter(match => {
          // TODO: Filter by user's orders
          // This requires matching order IDs with user
          return false;
        })
        .map(match => ({
          id: match.order_id.toString(),
          side: 'Buy' as const, // TODO: Get from order data
          amount: Number(match.fill_amount) / Math.pow(10, 18),
          limitPrice: Number(match.fill_price) / 100,
          filled: match.filled,
          surplus: Number(match.surplus) / 100,
        }))
    : [];

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 space-y-8">
      {/* Page Title */}
      <motion.h1
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-4xl font-bold text-center gradient-text mb-8"
      >
        Round #{clearingResult.round_id.toString()} Results
      </motion.h1>

      {/* Clearing Price Reveal */}
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2 }}
      >
        <ClearingPriceReveal
          price={Number(clearingResult.clearing_price) / 100}
          volume={Number(clearingResult.total_volume) / Math.pow(10, 18)}
        />
      </motion.div>

      {/* User Stats */}
      {principal && userStats && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
        >
          <UserStatsCard
            ordersFilled={Number(userStats.filled_orders)}
            ordersSubmitted={Number(userStats.total_orders)}
            surplusEarned={Number(userStats.total_surplus) / 100}
            rank={userRank || 1}
            totalPlayers={leaderboard.length}
          />
        </motion.div>
      )}

      {/* Leaderboard */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="glass rounded-3xl p-8"
      >
        <h3 className="text-3xl font-bold mb-6 gradient-text">Leaderboard</h3>
        
        <div className="space-y-3">
          {leaderboard.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              No players this round
            </div>
          ) : (
            leaderboard.slice(0, 10).map((entry, index) => (
              <motion.div
                key={entry.user.toText()}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.7 + index * 0.05 }}
                className={`
                  glass p-4 rounded-xl flex items-center justify-between
                  ${principal && entry.user.toText() === principal.toText() 
                    ? 'ring-2 ring-veil-accent' 
                    : ''
                  }
                `}
              >
                <div className="flex items-center gap-4">
                  {/* Rank Badge */}
                  <div className={`
                    w-10 h-10 rounded-full flex items-center justify-center font-bold
                    ${Number(entry.rank) === 1 ? 'bg-yellow-500 text-black' : ''}
                    ${Number(entry.rank) === 2 ? 'bg-gray-300 text-black' : ''}
                    ${Number(entry.rank) === 3 ? 'bg-amber-600 text-white' : ''}
                    ${Number(entry.rank) > 3 ? 'bg-gray-700 text-gray-300' : ''}
                  `}>
                    {Number(entry.rank) <= 3 && Number(entry.rank) === 1 && '🥇'}
                    {Number(entry.rank) <= 3 && Number(entry.rank) === 2 && '🥈'}
                    {Number(entry.rank) <= 3 && Number(entry.rank) === 3 && '🥉'}
                    {Number(entry.rank) > 3 && `#${entry.rank}`}
                  </div>

                  {/* User Info */}
                  <div>
                    <div className="font-mono text-sm text-gray-400">
                      {entry.user.toText().slice(0, 10)}...{entry.user.toText().slice(-6)}
                    </div>
                    <div className="text-xs text-gray-500">
                      Fill Rate: {Number(entry.fill_rate)}%
                    </div>
                  </div>
                </div>

                {/* Surplus */}
                <div className="text-right">
                  <div className="text-2xl font-bold text-green-400">
                    ${(Number(entry.surplus) / 100).toFixed(2)}
                  </div>
                  <div className="text-xs text-gray-500">surplus earned</div>
                </div>
              </motion.div>
            ))
          )}
        </div>
      </motion.div>

      {/* Total Surplus Info */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.8 }}
        className="glass p-6 rounded-2xl text-center"
      >
        <div className="text-sm text-gray-400 mb-2">Total Surplus This Round</div>
        <div className="text-4xl font-bold gradient-text">
          ${(Number(clearingResult.total_surplus) / 100).toFixed(2)}
        </div>
        <div className="text-xs text-gray-500 mt-2">
          Distributed among {clearingResult.matches.filter(m => m.filled).length} filled orders
        </div>
      </motion.div>
    </div>
  );
}

export default ResultsPage;