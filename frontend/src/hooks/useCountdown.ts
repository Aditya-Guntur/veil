import { useState, useEffect } from 'react';

export const useCountdown = (endTime: number) => {
  const [timeLeft, setTimeLeft] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      const remaining = Math.max(0, endTime - now);
      setTimeLeft(remaining);

      if (remaining === 0) {
        clearInterval(interval);
      }
    }, 100); // Update every 100ms for smooth animation

    return () => clearInterval(interval);
  }, [endTime]);

  const minutes = Math.floor(timeLeft / 60000);
  const seconds = Math.floor((timeLeft % 60000) / 1000);
  const progress = ((60000 - timeLeft) / 60000) * 100;
  const isUrgent = timeLeft < 10000; // Last 10 seconds

  return {
    minutes,
    seconds,
    progress,
    isUrgent,
    timeLeft,
  };
};