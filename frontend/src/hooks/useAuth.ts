import { useState, useEffect } from 'react';
import { AuthClient } from '@dfinity/auth-client';
import type { Identity } from '@dfinity/agent';

export const useAuth = () => {
  const [authClient, setAuthClient] = useState<AuthClient | null>(null);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    initAuth();
  }, []);

  const initAuth = async () => {
    const client = await AuthClient.create();
    setAuthClient(client);

    const isAuth = await client.isAuthenticated();
    setIsAuthenticated(isAuth);

    if (isAuth) {
      const identity = client.getIdentity();
      setIdentity(identity);
    }

    setIsLoading(false);
  };

  const login = async () => {
    if (!authClient) return;

    await authClient.login({
      identityProvider: import.meta.env.VITE_REACT_APP_INTERNET_IDENTITY_URL || 
        'https://identity.ic0.app',
      onSuccess: () => {
        const identity = authClient.getIdentity();
        setIdentity(identity);
        setIsAuthenticated(true);
      },
    });
  };

  const logout = async () => {
    if (!authClient) return;

    await authClient.logout();
    setIdentity(null);
    setIsAuthenticated(false);
  };

  return {
    identity,
    isAuthenticated,
    isLoading,
    login,
    logout,
  };
};