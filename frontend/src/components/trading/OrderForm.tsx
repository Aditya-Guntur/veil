import React, { useState } from 'react';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Card } from '../ui/Card';
import { TrendingUp, TrendingDown, Lock } from 'lucide-react';
import { motion } from 'framer-motion';
import toast from 'react-hot-toast';
import { canisterService } from '../../services/canister';
import { useAuth } from '../../hooks/useAuth';

export const OrderForm: React.FC = () => {
  const { isAuthenticated, login } = useAuth();
  const [side, setSide] = useState<'Buy' | 'Sell'>('Buy');
  const [amount, setAmount] = useState('');
  const [price, setPrice] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async () => {
    // Check authentication
    if (!isAuthenticated) {
      toast.error('Please connect your wallet first');
      try {
        await login();
      } catch (error) {
        return;
      }
    }

    // Validation
    if (!amount || !price) {
      toast.error('Please fill in all fields');
      return;
    }

    const amountNum = parseFloat(amount);
    const priceNum = parseFloat(price);

    if (amountNum <= 0 || priceNum <= 0) {
      toast.error('Amount and price must be positive');
      return;
    }

    setIsSubmitting(true);

    try {
      // Convert to base units
      // ETH: 1 ETH = 10^18 wei
      const amountInWei = Math.floor(amountNum * Math.pow(10, 18));
      
      // Price in USD cents (e.g., $3,100.00 = 310000 cents)
      const priceInCents = Math.floor(priceNum * 100);

      console.log(`Submitting order: ${side} ${amountNum} ETH @ $${priceNum}`);
      console.log(`Converting: ${amountInWei} wei @ ${priceInCents} cents`);

      // Show encrypting toast
      const encryptingToast = toast.loading('🔐 Encrypting order with vetKeys...');

      // Submit encrypted order
      const orderId = await canisterService.submitOrder(
        side,
        'ETH',
        amountInWei,
        priceInCents
      );

      toast.dismiss(encryptingToast);
      toast.success(
        `Order #${orderId.toString()} encrypted and submitted! 🎉`,
        { duration: 5000 }
      );

      // Reset form
      setAmount('');
      setPrice('');

    } catch (error: any) {
      console.error('Order submission error:', error);
      
      if (error.message.includes('not accepting orders')) {
        toast.error('Round is not active. Wait for next round.');
      } else if (error.message.includes('Anonymous users')) {
        toast.error('Please connect your wallet');
      } else {
        toast.error(`Failed to submit order: ${error.message}`);
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card className="max-w-md">
      <div className="space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-bold">Submit Order</h3>
          <Lock className="text-veil-accent" size={24} />
        </div>

        {/* Buy/Sell Toggle */}
        <div className="flex gap-2">
          <Button
            variant={side === 'Buy' ? 'success' : 'secondary'}
            className="flex-1"
            onClick={() => setSide('Buy')}
            disabled={isSubmitting}
          >
            <TrendingUp className="mr-2" size={18} />
            Buy
          </Button>
          <Button
            variant={side === 'Sell' ? 'danger' : 'secondary'}
            className="flex-1"
            onClick={() => setSide('Sell')}
            disabled={isSubmitting}
          >
            <TrendingDown className="mr-2" size={18} />
            Sell
          </Button>
        </div>

        {/* Amount Input */}
        <Input
          type="number"
          label="Amount"
          placeholder="0.00"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          icon={<span className="font-mono text-sm">ETH</span>}
          disabled={isSubmitting}
          step="0.01"
          min="0"
        />

        {/* Price Input */}
        <Input
          type="number"
          label={`${side === 'Buy' ? 'Maximum' : 'Minimum'} Price`}
          placeholder="0.00"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          icon={<span className="font-mono text-sm">USD</span>}
          disabled={isSubmitting}
          step="0.01"
          min="0"
        />

        {/* Info Box */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="glass p-2.2 rounded-lg text-sm text-gray-300"
        >
          <p className="mb-2">💡 <strong>How it works:</strong></p>
          <ul className="list-disc list-inside space-y-1 text-xs">
            <li>Your order will be encrypted using vetKeys timelock encryption</li>
            <li>Nobody can see your order until the round ends</li>
            <li>All orders reveal simultaneously when round timer hits 00:00</li>
            <li>Everyone trades at the same fair clearing price</li>
            <li>You earn surplus if your limit is better than clearing price</li>
          </ul>
        </motion.div>

        {/* Estimated Surplus */}
        {amount && price && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="glass p-3 rounded-lg"
          >
            <div className="text-xs text-gray-400 mb-1">Estimated Order Value</div>
            <div className="text-lg font-bold gradient-text">
              ${(parseFloat(amount) * parseFloat(price)).toFixed(2)}
            </div>
          </motion.div>
        )}

        {/* Submit Button */}
        <Button
          variant="primary"
          size="lg"
          glow
          className="w-full"
          onClick={handleSubmit}
          loading={isSubmitting}
          disabled={!amount || !price || isSubmitting}
        >
          {isSubmitting ? (
            <>
              <Lock className="mr-2 animate-pulse" size={18} />
              Encrypting...
            </>
          ) : (
            <>
              <Lock className="mr-2" size={18} />
              Encrypt & Submit Order
            </>
          )}
        </Button>

        {/* Connection Warning */}
        {!isAuthenticated && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="text-xs text-yellow-400 text-center flex items-center justify-center gap-2"
          >
            <span>⚠️</span>
            <span>Wallet must be connected to submit orders</span>
          </motion.div>
        )}
      </div>
    </Card>
  );
};

export default OrderForm;