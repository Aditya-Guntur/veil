# Mempool Chess - Privacy-Preserving Batch Auction on ICP

A decentralized batch auction platform built on the Internet Computer Protocol (ICP) that uses vetKeys timelock encryption to prevent MEV and ensure fair price discovery.

## Overview

Mempool Chess implements a privacy-preserving batch auction where:
- Orders are encrypted using vetKeys timelock encryption
- All orders reveal simultaneously when the round ends
- Everyone trades at the same fair clearing price
- Traders earn surplus when their limit price is better than the clearing price

## Architecture

### Backend (Rust Canisters)
- **mempool_chess_backend**: Main auction logic, order management, clearing algorithm
- **vetkeys_engine**: Handles timelock encryption and key derivation
- **internet_identity**: User authentication

### Frontend (React + TypeScript)
- React 19 with TypeScript
- Vite for bundling
- TailwindCSS for styling
- Integration with ICP canisters via @dfinity/agent

## Prerequisites

- **dfx**: Internet Computer SDK v0.15.0 or higher
- **Rust**: 1.70.0 or higher with wasm32-unknown-unknown target
- **Node.js**: v18 or higher
- **npm** or **yarn**

## Installation & Setup

### 1. Install dfx

```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

### 2. Install Rust with WebAssembly target

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### 3. Clone and Navigate

```bash
cd /path/to/mempool_chess
```

## Backend Deployment

### Build vetkeys_engine

```bash
cd backend/vetkeys_engine
cargo build --target wasm32-unknown-unknown --release
cd ../..
```

### Start Local Replica

```bash
cd backend
dfx start --clean --background
```

### Deploy All Canisters

```bash
dfx deploy
```

This will deploy:
- `internet_identity` - Authentication
- `vetkeys_engine` - Encryption engine
- `mempool_chess_backend` - Main application

### Get Canister IDs

```bash
dfx canister id mempool_chess_backend
dfx canister id internet_identity
```

## Frontend Setup

### Install Dependencies

```bash
cd frontend
npm install
```

### Configure Environment

Create `.env` file:

```bash
# Replace with your actual canister IDs from dfx canister id commands
VITE_CANISTER_ID=<your-backend-canister-id>
VITE_INTERNET_IDENTITY_URL=http://127.0.0.1:4943/?canisterId=<your-ii-canister-id>
VITE_DFX_NETWORK=local
```

### Start Development Server

```bash
npm run dev
```

Open http://localhost:5173

## Usage

### Admin Functions (via dfx)

```bash
# Start a new round
dfx canister call mempool_chess_backend admin_start_round

# Check round state
dfx canister call mempool_chess_backend get_round_state

# Run clearing
dfx canister call mempool_chess_backend admin_run_clearing

# View results
dfx canister call mempool_chess_backend get_current_round_result
```

### Submit Orders (via CLI)

```bash
# Create test identities
dfx identity new trader1 --storage-mode=plaintext
dfx identity new trader2 --storage-mode=plaintext

# Switch to trader1
dfx identity use trader1

# Submit buy order (BTC, 200000 satoshis @ $10000)
dfx canister call mempool_chess_backend submit_order \
  '(variant { Buy }, variant { BTC }, 200000, 10000, vec {}, "")'

# Switch to trader2
dfx identity use trader2

# Submit sell order
dfx canister call mempool_chess_backend submit_order \
  '(variant { Sell }, variant { BTC }, 200000, 9000, vec {}, "")'

# Switch back to default
dfx identity use default
```

## Features

### Batch Auction
- Uniform-price clearing mechanism
- Maximizes matched volume
- Fair price discovery for all participants

### Privacy
- Orders encrypted with vetKeys timelock encryption
- Nobody can see order details until round ends
- Simultaneous reveal prevents frontrunning

### Surplus Rewards
- Traders earn surplus when limit price beats clearing price
- Buy orders: (clearing_price - limit_price) × volume
- Sell orders: (limit_price - clearing_price) × volume

### Assets Supported
- BTC (Bitcoin)
- ETH (Ethereum)

## Build Modes

### Demo Mode
```bash
# Build with demo feature (mock encryption)
cargo build --target wasm32-unknown-unknown --release --no-default-features --features demo
```

### VetKeys Mode
```bash
# Build with real vetKeys encryption
cargo build --target wasm32-unknown-unknown --release --no-default-features --features vetkeys
```

## Project Structure

```
mempool_chess/
├── backend/
│   ├── src/
│   │   ├── lib.rs           # Main canister logic
│   │   ├── auction.rs       # Clearing algorithm
│   │   ├── encryption.rs    # VetKeys integration
│   │   ├── ethereum.rs      # Ethereum integration
│   │   ├── queries.rs       # Query functions
│   │   ├── timers.rs        # Round timers
│   │   └── types.rs         # Data structures
│   ├── vetkeys_engine/      # Encryption canister
│   ├── dfx.json             # DFX configuration
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── components/      # React components
│   │   ├── pages/           # Page components
│   │   ├── services/        # API services
│   │   └── utils/           # Utilities
│   └── package.json
└── README.md
```

## Testing

### Backend Tests

```bash
cd backend
cargo test
```

### Run Demo Script

```bash
cd backend
./demo.sh
```

This script:
1. Starts clean replica
2. Deploys all canisters
3. Creates test identities
4. Submits sample orders
5. Runs clearing
6. Shows results

## Troubleshooting

### Canister Not Found
```bash
dfx canister create --all
dfx deploy
```

### Build Errors
```bash
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

### Frontend Connection Issues
- Verify canister IDs in `.env` match deployed canisters
- Check dfx is running: `dfx ping`
- Hard refresh browser: `Ctrl+Shift+R`

## Technology Stack

**Backend:**
- Rust (Canister development)
- ic-cdk (Internet Computer SDK)
- ic-stable-structures (Persistent storage)
- ethers-core (Ethereum types)
- k256 (Cryptography)

**Frontend:**
- React 19
- TypeScript
- Vite
- TailwindCSS
- @dfinity/agent (ICP integration)
- @dfinity/vetkeys (Encryption)

## License

MIT License

## Built For

ICP Bitcoin DeFi Hackathon 2025
