import { Actor, HttpAgent } from '@dfinity/agent';
import type { Identity } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { sha256 } from 'js-sha256';

// ============================================================================
// VETKEYS ENCRYPTION (Temporary Implementation)
// ============================================================================
// NOTE: The @dfinity/vetkeys package is still in development
// This is a placeholder that simulates encryption for the hackathon demo
// In production, replace with actual vetKeys library when available

/**
 * Temporary encryption function for hackathon demo
 * In production, this will use the actual vetKeys library
 * For now, we just wrap the data with metadata to simulate encryption
 */
async function encryptData(
  data: Uint8Array,
  masterPublicKey: Uint8Array,
  timelockIdentity: Uint8Array
): Promise<Uint8Array> {
  console.log('🔐 Encrypting order with vetKeys (demo mode)');
  console.log('Master public key length:', masterPublicKey.length);
  console.log('Timelock identity length:', timelockIdentity.length);
  
  // For hackathon demo: Return the data as-is
  // The commitment hash provides cryptographic binding
  // In production, this would perform actual IBE encryption
  
  return data;
}

// ============================================================================
// TYPE DEFINITIONS (matching your Candid interface)
// ============================================================================

export type OrderType = { Buy: null } | { Sell: null };
export type Asset = { BTC: null } | { ETH: null };

export interface Order {
  id: bigint;
  round_id: bigint;
  owner: Principal;
  order_type: OrderType;
  asset: Asset;
  amount: bigint;
  price_limit: bigint;
  created_at: bigint;
  encrypted_payload: number[];
  commitment_hash: string;
}

export interface OrderMatch {
  order_id: bigint;
  filled: boolean;
  fill_amount: bigint;
  fill_price: bigint;
  surplus: bigint;
}

export interface ClearingResult {
  round_id: bigint;
  clearing_price: bigint;
  total_volume: bigint;
  total_surplus: bigint;
  matches: OrderMatch[];
  timestamp: bigint;
}

export interface State {
  round_id: bigint;
  round_state: RoundState;
  round_start_time: bigint;
  round_duration_ns: bigint;
  next_order_id: bigint;
  clearing_price_history: bigint[];
}

export type RoundState = 
  | { Pending: null }
  | { Active: null }
  | { Revealing: null }
  | { Clearing: null }
  | { Executing: null }
  | { Completed: null };

export interface UserStats {
  user: Principal;
  total_orders: bigint;
  filled_orders: bigint;
  total_surplus: bigint;
  rounds_participated: bigint;
}

export interface LeaderboardEntry {
  user: Principal;
  surplus: bigint;
  fill_rate: bigint;
  rank: bigint;
}

export interface OrderBookSummary {
  round_id: bigint;
  buy_orders: bigint;
  sell_orders: bigint;
  total_buy_volume: bigint;
  total_sell_volume: bigint;
}

// ============================================================================
// IDL FACTORY (Candid Interface)
// ============================================================================

export const idlFactory = ({ IDL }: any) => {
  const Asset = IDL.Variant({ BTC: IDL.Null, ETH: IDL.Null });
  const OrderType = IDL.Variant({ Buy: IDL.Null, Sell: IDL.Null });
  const RoundState = IDL.Variant({
    Pending: IDL.Null,
    Active: IDL.Null,
    Revealing: IDL.Null,
    Clearing: IDL.Null,
    Executing: IDL.Null,
    Completed: IDL.Null,
  });
  
  const State = IDL.Record({
    round_id: IDL.Nat64,
    round_state: RoundState,
    round_start_time: IDL.Nat64,
    round_duration_ns: IDL.Nat64,
    next_order_id: IDL.Nat64,
    clearing_price_history: IDL.Vec(IDL.Nat64),
  });

  const Order = IDL.Record({
    id: IDL.Nat64,
    round_id: IDL.Nat64,
    owner: IDL.Principal,
    order_type: OrderType,
    asset: Asset,
    amount: IDL.Nat64,
    price_limit: IDL.Nat64,
    created_at: IDL.Nat64,
    encrypted_payload: IDL.Vec(IDL.Nat8),
    commitment_hash: IDL.Text,
  });

  const OrderMatch = IDL.Record({
    order_id: IDL.Nat64,
    filled: IDL.Bool,
    fill_amount: IDL.Nat64,
    fill_price: IDL.Nat64,
    surplus: IDL.Nat64,
  });

  const ClearingResult = IDL.Record({
    round_id: IDL.Nat64,
    clearing_price: IDL.Nat64,
    total_volume: IDL.Nat64,
    total_surplus: IDL.Nat64,
    matches: IDL.Vec(OrderMatch),
    timestamp: IDL.Nat64,
  });

  const UserStats = IDL.Record({
    user: IDL.Principal,
    total_orders: IDL.Nat64,
    filled_orders: IDL.Nat64,
    total_surplus: IDL.Nat64,
    rounds_participated: IDL.Nat64,
  });

  const LeaderboardEntry = IDL.Record({
    user: IDL.Principal,
    surplus: IDL.Nat64,
    fill_rate: IDL.Nat64,
    rank: IDL.Nat64,
  });

  const OrderBookSummary = IDL.Record({
    round_id: IDL.Nat64,
    buy_orders: IDL.Nat64,
    sell_orders: IDL.Nat64,
    total_buy_volume: IDL.Nat64,
    total_sell_volume: IDL.Nat64,
  });

  const ResultOrder = IDL.Variant({
    Ok: IDL.Nat64,
    Err: IDL.Text,
  });

  const ResultBytes = IDL.Variant({
    Ok: IDL.Vec(IDL.Nat8),
    Err: IDL.Text,
  });

  return IDL.Service({
    // State queries
    get_round_state: IDL.Func([], [State], ['query']),
    get_order_count: IDL.Func([], [IDL.Nat64], ['query']),
    get_current_round_orders: IDL.Func([], [IDL.Nat64], ['query']),
    get_time_remaining: IDL.Func([], [IDL.Nat64], ['query']),
    
    // Order submission
    submit_order: IDL.Func(
      [OrderType, Asset, IDL.Nat64, IDL.Nat64, IDL.Vec(IDL.Nat8), IDL.Text],
      [ResultOrder],
      []
    ),
    
    // Encryption
    get_encryption_public_key: IDL.Func([], [ResultBytes], []),
    get_round_timelock_identity: IDL.Func([IDL.Nat64], [IDL.Vec(IDL.Nat8)], ['query']),
    
    // User queries
    get_user_orders: IDL.Func([IDL.Principal], [IDL.Vec(Order)], ['query']),
    get_user_current_round_orders: IDL.Func([IDL.Principal], [IDL.Vec(Order)], ['query']),
    get_user_stats: IDL.Func([IDL.Principal], [IDL.Opt(UserStats)], ['query']),
    
    // Round queries
    get_round_result: IDL.Func([IDL.Nat64], [IDL.Opt(ClearingResult)], ['query']),
    get_current_round_result: IDL.Func([], [IDL.Opt(ClearingResult)], ['query']),
    get_round_orders: IDL.Func([IDL.Nat64], [IDL.Vec(Order)], ['query']),
    get_price_history: IDL.Func([], [IDL.Vec(IDL.Nat64)], ['query']),
    
    // Leaderboard
    get_round_leaderboard: IDL.Func([IDL.Nat64], [IDL.Vec(LeaderboardEntry)], ['query']),
    get_global_leaderboard: IDL.Func([], [IDL.Vec(LeaderboardEntry)], ['query']),
    
    // Order book
    get_order_book_summary: IDL.Func([], [OrderBookSummary], ['query']),
    
    // Admin
    admin_start_round: IDL.Func([], [IDL.Text], []),
    admin_run_clearing: IDL.Func([], [IDL.Text], []),
  });
};

// ============================================================================
// CANISTER SERVICE CLASS
// ============================================================================

class CanisterService {
  private actor: any = null;
  private agent: HttpAgent | null = null;
  private identity: Identity | null = null;
  private masterPublicKey: Uint8Array | null = null;
  
  getActor() {
  return this.actor;
  }

  // Canister ID - set this after deployment
  private canisterId = import.meta.env.VITE_CANISTER_ID || 'rrkah-fqaaa-aaaaa-aaaaq-cai';

  async initialize(identity?: Identity) {
    this.identity = identity || null;
    
    const host = import.meta.env.VITE_DFX_NETWORK === 'ic' 
      ? 'https://ic0.app'
      : 'http://127.0.0.1:4943';

    this.agent = new HttpAgent({ 
      host,
      identity: this.identity || undefined,
    });

    // Fetch root key for local development
    if (import.meta.env.VITE_DFX_NETWORK !== 'ic') {
      await this.agent.fetchRootKey();
    }

    this.actor = Actor.createActor(idlFactory, {
      agent: this.agent,
      canisterId: this.canisterId,
    });

    console.log("Agent initialized:", !!this.agent);
    console.log("Actor initialized:", !!this.actor);
  }

  // ============================================================================
  // ENCRYPTION METHODS
  // ============================================================================

  async getMasterPublicKey(): Promise<Uint8Array> {
    if (this.masterPublicKey) {
      return this.masterPublicKey;
    }

    const result = await this.actor.get_encryption_public_key();
    
    if ('Err' in result) {
      throw new Error(result.Err);
    }

    this.masterPublicKey = new Uint8Array(result.Ok);
    
    // Cache in localStorage
    localStorage.setItem('vetkd_master_key', JSON.stringify({
      key: Array.from(this.masterPublicKey),
      timestamp: Date.now(),
    }));

    return this.masterPublicKey;
  }

  async getTimelockIdentity(roundId: bigint): Promise<Uint8Array> {
    const identity = await this.actor.get_round_timelock_identity(roundId);
    return new Uint8Array(identity);
  }

  private formatOrderData(
    orderType: 'Buy' | 'Sell',
    asset: 'BTC' | 'ETH',
    amount: number,
    priceLimit: number
  ): string {
    return JSON.stringify({
      order_type: orderType,
      asset: asset,
      amount: amount,
      price_limit: priceLimit,
      timestamp: Date.now() * 1_000_000, // nanoseconds
    });
  }

  private generateCommitmentHash(orderData: string): string {
    return sha256(orderData);
  }

  // ============================================================================
  // ORDER SUBMISSION
  // ============================================================================

  // async submitOrder(
  //   side: 'Buy' | 'Sell',
  //   asset: 'BTC' | 'ETH',
  //   amount: number, // in base units (satoshis/wei)
  //   priceLimit: number // in USD cents
  // ): Promise<bigint> {
  //   if (!this.actor) {
  //     throw new Error('Canister not initialized');
  //   }

  //   // Get current round state
  //   const state = await this.getRoundState();
    
  //   if (!('Active' in state.round_state)) {
  //     throw new Error('Round is not accepting orders');
  //   }

  //   const roundId = state.round_id;

  //   // Format order data
  //   const orderData = this.formatOrderData(side, asset, amount, priceLimit);
    
  //   // Generate commitment hash BEFORE encryption
  //   const commitmentHash = this.generateCommitmentHash(orderData);

  //   // Get master public key and timelock identity
  //   const masterPubKey = await this.getMasterPublicKey();
  //   const timelockIdentity = await this.getTimelockIdentity(roundId);

  //   // Encrypt order using vetKeys
  //   const encryptedPayload = await encryptData(
  //     new TextEncoder().encode(orderData),
  //     masterPubKey,
  //     timelockIdentity
  //   );

  //   // Submit to canister
  //   const orderTypeVariant: OrderType = side === 'Buy' ? { Buy: null } : { Sell: null };
  //   const assetVariant: Asset = asset === 'BTC' ? { BTC: null } : { ETH: null };

  //   const result = await this.actor.submit_order(
  //     orderTypeVariant,
  //     assetVariant,
  //     BigInt(amount),
  //     BigInt(priceLimit),
  //     Array.from(encryptedPayload),
  //     commitmentHash
  //   );

  //   if ('Err' in result) {
  //     throw new Error(result.Err);
  //   }

  //   return result.Ok;
  // }
  
  async submitOrder(
  side: 'Buy' | 'Sell',
  asset: 'BTC' | 'ETH',
  amount: number,
  priceLimit: number
): Promise<bigint> {
  if (!this.actor) {
    throw new Error('Canister not initialized');
  }

  // 1. Get round state
  const state = await this.getRoundState();
  if (!('Active' in state.round_state)) {
    throw new Error('Round is not accepting orders');
  }

  const roundId = state.round_id;

  // 2. Prepare plaintext order JSON
  const orderData = JSON.stringify({
    side,
    asset,
    amount,
    priceLimit,
    roundId
  });

  // 3. Commitment hash (unchanged)
  const commitmentHash = this.generateCommitmentHash(orderData);

  // ✅ 4. Mock encryption instead of vetKeys
  const encryptedPayload = new TextEncoder().encode(
    `MOCK_ENCRYPTED::${orderData}`
  );

  // 5. Variants
  const orderTypeVariant: OrderType = side === 'Buy' ? { Buy: null } : { Sell: null };
  const assetVariant: Asset = asset === 'BTC' ? { BTC: null } : { ETH: null };

  // ✅ 6. Call PocketIC-compatible method
  const result = await this.actor.pocketic_submit_order(
    BigInt(roundId),
    Array.from(encryptedPayload),
    commitmentHash
  );

  if ('Err' in result) {
    throw new Error(result.Err);
  }

  return result.Ok;
}


  // ============================================================================
  // STATE QUERIES
  // ============================================================================

  async getRoundState(): Promise<State> {
  if (!this.actor) throw new Error("Actor not initialized");
  return await this.actor.get_round_state();;
  }

  async getOrderCount(): Promise<bigint> {
    return await this.actor.get_order_count();
  }

  async getCurrentRoundOrders(): Promise<bigint> {
    return await this.actor.get_current_round_orders();
  }

  async getTimeRemaining(): Promise<bigint> {
    return await this.actor.get_time_remaining();
  }

  async getOrderBookSummary(): Promise<OrderBookSummary> {
    return await this.actor.get_order_book_summary();
  }

  // ============================================================================
  // USER QUERIES
  // ============================================================================

  async getUserOrders(principal: Principal): Promise<Order[]> {
    return await this.actor.get_user_orders(principal);
  }

  async getUserCurrentRoundOrders(principal: Principal): Promise<Order[]> {
    return await this.actor.get_user_current_round_orders(principal);
  }

  async getUserStats(principal: Principal): Promise<UserStats | null> {
    const result = await this.actor.get_user_stats(principal);
    return result.length > 0 ? result[0] : null;
  }

  // ============================================================================
  // ROUND QUERIES
  // ============================================================================

  async getCurrentRoundResult(): Promise<ClearingResult | null> {
    const result = await this.actor.get_current_round_result();
    return result.length > 0 ? result[0] : null;
  }

  async getRoundResult(roundId: bigint): Promise<ClearingResult | null> {
    const result = await this.actor.get_round_result(roundId);
    return result.length > 0 ? result[0] : null;
  }

  async getPriceHistory(): Promise<bigint[]> {
    return await this.actor.get_price_history();
  }

  // ============================================================================
  // LEADERBOARD
  // ============================================================================

  async getGlobalLeaderboard(): Promise<LeaderboardEntry[]> {
    return await this.actor.get_global_leaderboard();
  }

  async getRoundLeaderboard(roundId: bigint): Promise<LeaderboardEntry[]> {
    return await this.actor.get_round_leaderboard(roundId);
  }

  // ============================================================================
  // ADMIN (for testing)
  // ============================================================================

  async adminStartRound(): Promise<string> {
    return await this.actor.admin_start_round();
  }

  async adminRunClearing(): Promise<string> {
    return await this.actor.admin_run_clearing();
  }

  // ============================================================================
  // UTILITIES
  // ============================================================================

  formatPrice(cents: bigint): string {
    return `$${(Number(cents) / 100).toFixed(2)}`;
  }

  formatAmount(wei: bigint, decimals: number = 18): string {
    return (Number(wei) / Math.pow(10, decimals)).toFixed(4);
  }

  getRoundStateString(state: RoundState): string {
    if ('Pending' in state) return 'Pending';
    if ('Active' in state) return 'Active';
    if ('Revealing' in state) return 'Revealing';
    if ('Clearing' in state) return 'Clearing';
    if ('Executing' in state) return 'Executing';
    if ('Completed' in state) return 'Completed';
    return 'Unknown';
  }
}

// Export singleton instance
export const canisterService = new CanisterService();