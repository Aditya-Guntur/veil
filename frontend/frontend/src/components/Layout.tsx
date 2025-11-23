import React from 'react';
import { AnimatedBackground } from './AnimatedBackground';

const Layout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <div className="min-h-screen font-sans relative">
      <AnimatedBackground />
      <div className="fixed inset-0 opacity-[0.015] pointer-events-none"
           style={{ backgroundImage: 'url("data:image/svg+xml,%3Csvg viewBox=\'0 0 400 400\' xmlns=\'http://www.w3.org/200/svg\'%3E%3Cfilter id=\'noiseFilter\'%3E%3CfeTurbulence type=\'fractalNoise\' baseFrequency=\'0.9\' numOctaves=\'4\' /%3E%3C/filter%3E%3Crect width=\'100%25\' height=\'100%25\' filter=\'url(%23noiseFilter)\' /%3E%3C/svg%3E")' }} />
      
      <header className="fixed top-0 left-0 w-full z-10 bg-transparent p-4">
        <div className="container mx-auto flex justify-between items-center">
          <h1 className="text-2xl font-bold gradient-text">VEIL</h1>
          {/* Navigation can go here */}
        </div>
      </header>
      <main className="container mx-auto pt-20 pb-10 px-4 relative z-0">{children}</main>
      <footer className="text-center p-4 text-text-secondary">
        <p>&copy; 2025 Veil. All rights reserved.</p>
      </footer>
    </div>
  );
};

export default Layout;