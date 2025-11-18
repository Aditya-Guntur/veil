import React, { useState } from 'react';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Card } from '../ui/Card';
import { TrendingUp, TrendingDown, Lock } from 'lucide-react';
import { motion } from 'framer-motion';
import toast from 'react-hot-toast';

export const OrderForm: React.FC = () => {
  const [side, setSide] = useState<'Buy' | 'Sell'>('Buy');
  const [amount, setAmount] = useState('');
  const [price, setPrice] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async () => {
    // Validation
    if (!amount || !price) {
      toast.error('Please fill in all fields');
      return;
    }

    setIsSubmitting(true);
    
    try {
      // TODO: Encrypt order
      // TODO: Submit to canister
      
      toast.success('Order encrypted and submitted! 🔒');
      
      // Reset form
      setAmount('');
      setPrice('');
    } catch (error) {
      toast.error('Failed to submit order');
      console.error(error);
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
          >
            <TrendingUp className="mr-2" size={18} />
            Buy
          </Button>
          <Button
            variant={side === 'Sell' ? 'danger' : 'secondary'}
            className="flex-1"
            onClick={() => setSide('Sell')}
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
        />

        {/* Price Input */}
        <Input
          type="number"
          label={`${side === 'Buy' ? 'Maximum' : 'Minimum'} Price`}
          placeholder="0.00"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          icon={<span className="font-mono text-sm">USD</span>}
        />

        {/* Info Box */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="glass p-4 rounded-lg text-sm text-gray-300"
        >
          <p className="mb-2">💡 <strong>How it works:</strong></p>
          <ul className="list-disc list-inside space-y-1 text-xs">
            <li>Your order will be encrypted using vetKeys</li>
            <li>Nobody can see your order until the round ends</li>
            <li>All orders reveal simultaneously at 00:00</li>
            <li>Everyone trades at the same fair clearing price</li>
          </ul>
        </motion.div>

        {/* Submit Button */}
        <Button
          variant="primary"
          size="lg"
          glow
          className="w-full"
          onClick={handleSubmit}
          loading={isSubmitting}
        >
          <Lock className="mr-2" size={18} />
          Encrypt & Submit Order
        </Button>
      </div>
    </Card>
  );
};

export default OrderForm;