import { AuthClient } from "@dfinity/auth-client";
import type { Identity } from '@dfinity/agent';
import { canisterService } from "../services/canister";

export interface WalletInfo {
  id: string;
  name: string;
  address: string;          // principal string for ICP
  chain: string;
  connected: boolean;
  identity?: Identity;       // ICP identity object
}

export interface WalletConnectionResult {
  success: boolean;
  wallet?: WalletInfo;
  error?: string;
}

class WalletManager {
  private connectedWallets: Map<string, WalletInfo> = new Map();

  async connectMetaMask(): Promise<WalletConnectionResult> {
    try {
      const eth = (window as any).ethereum;
      if (!eth) {
        return { success: false, error: "MetaMask not installed" };
      }

      // This must be directly triggered
      const accounts: string[] = await eth.request({
        method: "eth_requestAccounts"
      });

      if (!accounts || accounts.length === 0) {
        return { success: false, error: "No accounts returned" };
      }

      const wallet: WalletInfo = {
        id: "metamask",
        name: "MetaMask",
        address: accounts[0],
        chain: "EVM",
        connected: true
      };

      this.connectedWallets.set("metamask", wallet);
      console.log("✅ MetaMask authorized:", wallet);

      return { success: true, wallet };
    } catch (err: any) {
      return {
        success: false,
        error: err?.message || "User rejected MetaMask"
      };
    }
  }

  async connectBitcoin(): Promise<WalletConnectionResult> {
    try {
      // Check if a Bitcoin wallet is available (e.g., Xverse, Unisat, etc.)
      const bitcoinWallet = (window as Window & { unisat?: unknown })?.unisat;
      if (!bitcoinWallet) {
        return { success: false, error: 'Bitcoin wallet not detected. Please install a Bitcoin wallet like Unisat or Xverse.' };
      }

      const accounts = await (bitcoinWallet as { requestAccounts: () => Promise<string[]> }).requestAccounts();
      if (accounts.length === 0) {
        return { success: false, error: 'No Bitcoin accounts found' };
      }

      const address = accounts[0];
      const wallet: WalletInfo = {
        id: 'bitcoin',
        name: 'Bitcoin',
        address,
        chain: 'Bitcoin',
        connected: true
      };

      this.connectedWallets.set('bitcoin', wallet);
      return { success: true, wallet };
    } catch (error: unknown) {
      return { success: false, error: error instanceof Error ? error.message : 'Failed to connect Bitcoin wallet' };
    }
  }

  async connectInternetIdentity(): Promise<WalletConnectionResult> {
    try {
      console.log("Starting ICP login");

      const authClient = await AuthClient.create();

      await authClient.login({
        identityProvider: (import.meta.env.VITE_INTERNET_IDENTITY_URL || "http://127.0.0.1:4943/?canisterId=uxrrr-q7777-77774-qaaaq-cai") as string,
        windowOpenerFeatures: "width=500,height=700,left=100,top=100",
        onSuccess: () => {
          console.log("✅ ICP login success");
        },
      });

      const identity = await authClient.getIdentity();
      await canisterService.initialize(identity);
      const principal = identity.getPrincipal();

      const wallet: WalletInfo = {
        id: "icp",
        name: "Internet Identity",
        address: principal.toText(),
        chain: "ICP",
        connected: true,
        identity,
      };

      this.connectedWallets.set("icp", wallet);

      return { success: true, wallet };
    } catch (error) {
      console.error("ICP connect failed:", error);
      return {
        success: false,
        error: "Failed to connect Internet Identity",
      };
    }
  }

  async connectWallet(walletId: string): Promise<WalletConnectionResult> {
    switch (walletId) {
      case 'metamask':
        return this.connectMetaMask();
      case 'bitcoin':
        return this.connectBitcoin();
      case 'icp':
        return this.connectInternetIdentity();
      default:
        return { success: false, error: 'Unknown wallet type' };
    }
  }

  getICIdentity(): unknown | null {
    const wallet = this.getWallet("icp");
    return wallet?.identity ?? null;
  }

  disconnectWallet(walletId: string): void {
    this.connectedWallets.delete(walletId);
  }

  getConnectedWallets(): WalletInfo[] {
    return Array.from(this.connectedWallets.values());
  }

  getWallet(walletId: string): WalletInfo | undefined {
    return this.connectedWallets.get(walletId);
  }

  isWalletConnected(walletId: string): boolean {
    return this.connectedWallets.has(walletId);
  }

  getPrimaryWallet(): WalletInfo | undefined {
    // Return the first connected wallet as primary
    return this.getConnectedWallets()[0];
  }
}

export const walletManager = new WalletManager();