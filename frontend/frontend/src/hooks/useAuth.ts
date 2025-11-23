import { useState, useEffect } from 'react';
import type { Identity } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { MOCK_USERS } from '../utils/mockUsers';
import { Ed25519KeyIdentity } from '@dfinity/identity';


export function useAuth() {
  // A single, stable, static identity for all transactions.
  // This is generated once and never changes.
  const [identity, setIdentity] = useState<Identity | null>(null);
  // The principal string that changes for UI purposes.
  const [displayPrincipal, setDisplayPrincipal] = useState<string | null>(null);

  // Create the static identity once on mount.
  useEffect(() => {
    // This creates a new, random, non-anonymous identity.
    const staticIdentity = Ed25519KeyIdentity.generate();
    setIdentity(staticIdentity);
    // Set the initial display principal to Alice
    setDisplayPrincipal(MOCK_USERS[0].principal.toString());
  }, []); // Run only once.

  // This function ONLY changes the string used for display.
  const setMockUser = (principalId: string) => {
    setDisplayPrincipal(principalId);
  };

  // Dummy login function for compatibility.
  const login = async () => {
    setMockUser(MOCK_USERS[0].principal.toString());
  };

  const isAuthenticated = !!identity;

  return { 
    identity, // The static identity for transactions
    principal: displayPrincipal, // The changeable principal for UI
    setMockUser,
    isAuthenticated,
    login
  };
}
