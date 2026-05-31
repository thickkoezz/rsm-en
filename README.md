# rsm-en

A simplified blockchain runtime implementation in Rust, designed for educational purposes to demonstrate the core concepts of blockchain architecture, pallet-based modularity, and transaction processing.

## Overview

`rsm-en` implements a minimal blockchain runtime with multiple pallets (modules), cryptographic signing, and event emission. It showcases how modern blockchains like Polkadot/Substrate are structured at a fundamental level.

## Implemented Features

Based on the feature roadmap in [`.docs/feature-suggestions.txt`](.docs/feature-suggestions.txt):

| # | Feature | Status | Description |
|---|---------|--------|-------------|
| 1 | Cryptographic Signatures | ✅ Implemented | Ed25519 signature verification, signed transactions, nonce-based replay protection |
| 2 | Events/System Logs | ✅ Implemented | Event emission from pallets, queryable event log by block number |
| 3 | Transaction Fee Mechanism | ✅ Implemented | Flat fee per transaction, fee deduction before execution, insufficient balance rejection |
| 4 | Persistent Storage | ✅ Implemented | Embedded key-value storage (sled), state serialization to disk, automatic recovery |
| 5 | Genesis Configuration | ✅ Implemented | Formal genesis state struct with builder pattern |
| 6 | Simple CLI | ⏳ Pending | Command-line interface for transactions |
| 7 | Basic P2P Networking | ⏳ Pending | libp2p node connections and gossip |
| 8 | Consensus Lightweight | ⏳ Pending | Raft or Proof-of-Work consensus |

## Technical Features

- **Modular Pallet Architecture**: Composable blockchain logic through independent pallets
- **Cryptographic Security**: Ed25519 signature verification for transaction authorization
- **Replay Attack Prevention**: Nonce-based transaction ordering
- **Event System**: Per-block event tracking and querying
- **Procedural Macros**: Custom `#[runtime]` and `#[call]` macros for automatic code generation
- **Persistent Storage**: Embedded key-value database for state persistence across program restarts
- **Genesis Configuration**: Formal genesis state struct with builder pattern for initial blockchain setup

## Pallets

| Pallet | Description |
|--------|-------------|
| **System** | Manages block numbers, account nonces, and system-level state |
| **Balances** | Handles account balances and token transfers |
| **Proof of Existence** | Allows users to create and revoke claims for data existence proofs |
| **Events** | Records and queries events emitted during block execution |
| **Fees** | Manages transaction fee collection and total fees tracking |

## Project Structure

```
rsm-en/
├── Cargo.toml           # Main project dependencies
├── macros/              # Procedural macros for code generation
│   ├── src/call/       # #[call] macro for pallet functions
│   └── src/runtime/    # #[runtime] macro for runtime generation
├── src/
│   ├── main.rs         # Entry point with example usage
│   ├── support.rs      # Core types (Block, Header, Extrinsic, Dispatch)
│   ├── crypto.rs       # Cryptographic primitives (Ed25519 wrappers)
│   ├── transaction.rs  # Transaction builder for signed extrinsics
│   ├── genesis.rs      # Genesis configuration and builder
│   ├── storage.rs     # Persistent storage implementation
│   ├── system.rs       # System pallet implementation
│   ├── balances.rs     # Balances pallet implementation
│   ├── fees.rs         # Fees pallet implementation
│   ├── proof_of_existence.rs # PoE pallet implementation
│   ├── events.rs       # Events pallet implementation
│   └── event.rs        # Event type definitions
├── db/                  # Persistent storage directory (created at runtime)
└── .docs/              # Additional documentation
```

## Quick Start

### Prerequisites

- Rust 2024 edition or later
- Cargo

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

The example in `main.rs` demonstrates:
- **Persistent Storage**: Automatic state loading on startup, saving after each block
- **Generating Ed25519 keypairs** for accounts (Alice, Bob, Charlie)
- **Creating and executing blocks** with signed transfers
- **Creating claims** in the Proof of Existence pallet
- **Querying events** by block number
- **Replay attack prevention** through nonce verification
- **Transaction fee mechanism** with insufficient balance handling

### Persistence Demonstration

Run the program twice to see state persistence in action:

```bash
# First run - creates blockchain and saves state
cargo run

# Second run - loads and displays saved state
cargo run
```

To start fresh, remove the storage directory:
```bash
rm -rf db
```

## Key Concepts

### Exinsics

An extrinsic represents an external transaction submitted to the blockchain. Each extrinsic contains:
- **caller**: The account submitting the transaction (derived from public key)
- **call**: The pallet function to execute
- **signature**: Ed25519 signature authorizing the transaction
- **nonce**: Sequential number preventing replay attacks

### Runtime Configuration

Each pallet is configured through a `Config` trait that defines associated types:

```rust
impl system::Config for Runtime {
    type AccountId = types::AccountId;
    type BlockNumber = types::BlockNumber;
    type Nonce = types::Nonce;
}
```

### Event System

Events are emitted during successful extrinsic execution and can be queried by block number:

```rust
for event in runtime.events.events_at_block(1) {
    println!("{:?}", event);
}
```

### Genesis Configuration

The blockchain uses a formal genesis configuration system for initializing the runtime state. Instead of manually calling `set_balance()` and other initialization functions, you can define a `GenesisConfig` struct that captures all initial state:

```rust
use crate::genesis::GenesisConfig;

// Create genesis configuration using the builder pattern
let genesis = GenesisConfig::builder()
    .add_balance(alice_account, 1000)
    .add_balance(bob_account, 500)
    .add_claim("my_document", alice_account)
    .with_block_number(0)
    .build();

// Apply genesis configuration when creating a new runtime
let mut runtime = storage.load_state_or_create(Some(genesis))?;
```

The genesis configuration supports:
- **Initial balances**: Set starting balances for accounts
- **Initial claims**: Pre-populate proof of existence claims
- **Block number**: Start from a specific block (defaults to 0)
- **Nonces**: Set initial nonces for accounts (typically empty at genesis)

## Cryptography

This project uses:
- **Ed25519** for digital signatures via `ed25519-dalek`
- **SHA-512** for hashing payloads before signing
- **32-byte public keys** as account identifiers

See `.docs/crypto.md` for detailed documentation on cryptographic primitives.

## Dependencies

- `ed25519-dalek` - Ed25519 signature scheme
- `sha2` - SHA-512 hashing
- `serde`/`serde_json` - Serialization for call encoding
- `rand` - Cryptographically secure random number generation
- `num` - Numeric traits for checked arithmetic
- `sled` - Embedded key-value database for persistent storage
- `bincode` - Binary serialization for state persistence
- `hex` - Hex encoding for binary data serialization

## Documentation

Additional documentation is available in the `.docs/` directory:
- `architecture.md` - Detailed architecture explanation
- `pallets.md` - Pallet system documentation
- `crypto.md` - Cryptographic primitives and transaction signing
- `events.md` - Event system documentation
- `storage.md` - Persistent storage implementation details
- `genesis.md` - Genesis configuration system documentation

## License

This is an educational project. Use and modify freely for learning purposes.
