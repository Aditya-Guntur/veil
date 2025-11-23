import React from 'react';
import { motion } from 'framer-motion';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  hover?: boolean;
  glow?: boolean;
}

export const Card: React.FC<CardProps> = ({
  children,
  className = '',
  hover = false,
  glow = false,
}) => {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className={`
        glass rounded-2xl p-6
        ${hover ? 'card-hover' : ''}
        ${glow ? 'ring-2 ring-accent/50' : ''}
        ${className}
      `}
    >
      {children}
    </motion.div>
  );
};
