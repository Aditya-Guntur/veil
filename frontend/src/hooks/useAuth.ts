import { useState, useEffect } from 'react';
import { AuthClient } from '@dfinity/auth-client';
import type { Identity } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { canisterService } from '../services/canister';

// export const useAuth = () => {
//   const [authClient, setAuthClient] = useState<AuthClient | null>(null);
//   const [identity, setIdentity] = useState<Identity | null>(null);
//   const [principal, setPrincipal] = useState<Principal | null>(null);
//   const [isAuthenticated, setIsAuthenticated] = useState(false);
//   const [isLoading, setIsLoading] = useState(true);

//   useEffect(() => {
//     initAuth();
//   }, []);

//   const initAuth = async () => {
//     try {
//       const client = await AuthClient.create();
//       setAuthClient(client);

//       const isAuth = await client.isAuthenticated();
//       setIsAuthenticated(isAuth);

//       if (isAuth) {
//         const id = client.getIdentity();
//         const prin = id.getPrincipal();
        
//         setIdentity(id);
//         setPrincipal(prin);

//         // Initialize canister service with authenticated identity
//         await canisterService.initialize(id);
//       } else {
//         // Initialize canister service without authentication (for queries)
//         await canisterService.initialize();
//       }
//     } catch (error) {
//       console.error('Auth initialization failed:', error);
//     } finally {
//       setIsLoading(false);
//     }
//   };

//   const login = async () => {
//     if (!authClient) {
//       throw new Error('Auth client not initialized');
//     }

//     return new Promise<void>((resolve, reject) => {
//       authClient.login({
//         identityProvider: 
//           import.meta.env.VITE_INTERNET_IDENTITY_URL || 
//           'https://identity.ic0.app',
//         maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000), // 7 days
//         onSuccess: async () => {
//           const id = authClient.getIdentity();
//           const prin = id.getPrincipal();
          
//           setIdentity(id);
//           setPrincipal(prin);
//           setIsAuthenticated(true);

//           // Re-initialize canister with authenticated identity
//           await canisterService.initialize(id);
          
//           resolve();
//         },
//         onError: (error) => {
//           console.error('Login failed:', error);
//           reject(error);
//         },
//       });
//     });
//   };

//   const logout = async () => {
//     if (!authClient) return;

//     await authClient.logout();
//     setIdentity(null);
//     setPrincipal(null);
//     setIsAuthenticated(false);

//     // Re-initialize canister without authentication
//     await canisterService.initialize();
//   };

//   return {
//     authClient,
//     identity,
//     principal,
//     isAuthenticated,
//     isLoading,
//     login,
//     logout,
//   };
// };

const [mockUser, setMockUser] = useState<string | null>(null);

export const useAuth = () => {
  const isDemo = true; // hardcode for hackathon demo

  return {
    isAuthenticated: !!mockUser,
    principal: mockUser,
    login: async () => {}, // no-op
    setMockUser,
  };
};
