# Architecture

This document provides a detailed explanation of the rsm-en blockchain architecture.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Runtime                             │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌───────────────-─┐              │
│  │ System  │  │ Events  │  │ RuntimeCall Enum│              │
│  │ Pallet  │  │ Pallet  │  │                 │              │
│  └─────────┘  └─────────┘  │ - balances(...) │              │
│                            │ - proof_of_...  │              │
|                            |                 |              |
│  ┌─────────┐  ┌───────────┐└────────────────-┘              │
│  │Balances │  │  Proof    │                                 │
│  │ Pallet  │  │ of Exist. │                                 │
│  └─────────┘  └───────────┘                                 │
│                                                             │
│  Dispatch Trait Implementation                              │
└─────────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌──────────┐        ┌─────────---─┐        ┌──────────┐
    │   Block  │        │Extrinsic    │        │  Event   │
    │ Execution│        │ Verification│        │  Query   │
    └──────────┘        └──────────---┘        └──────────┘
```

## Core Components

### 1. Runtime

The `Runtime` struct is the central orchestrator that contains all pallets and implements the dispatch logic:

```rust
#[macros::runtime]
pub struct Runtime {
    system: system::Pallet<Runtime>,
    events: events::Pallet<Runtime>,
    balances: balances::Pallet<Runtime>,
    fees: fees::Pallet<Runtime>,
    proof_of_existence: proof_of_existence::Pallet<Runtime>,
}
```

**Generated Code**: The `#[runtime]` procedural macro automatically generates:
- `Runtime::new()` - Constructor initializing all pallets
- `Runtime::execute_block()` - Block execution logic
- `RuntimeCall` enum - Enum of all callable pallet functions
- `Dispatch` trait implementation - Call routing logic

**Genesis Configuration**: When creating a new blockchain runtime, you can provide a `GenesisConfig` struct to initialize the state:
- `GenesisConfig::builder()` - Builder pattern for creating genesis configuration
- `genesis.apply_to(runtime)` - Applies genesis state to a runtime
- Supports initial balances, claims, block number, and nonces
- See `src/genesis.rs` for details

### 2. Pallet System

A "pallet" is a modular unit of blockchain logic. Each pallet:
- Encapsulates specific functionality (balances, claims, etc.)
- Manages its own storage
- Implements the `Dispatch` trait for callable functions
- Is configured through a `Config` trait

**Pallet Structure**:
```rust
pub struct Pallet<T: Config> {
    // Storage items specific to this pallet
}

impl<T: Config> Pallet<T> {
    // Public functions callable from the runtime
}

impl<T: Config> Dispatch for Pallet<T> {
    // Routing logic for calls
}
```

### 3. Support Types

Located in `support.rs`, these provide the foundational types:

#### Block
```rust
pub struct Block<Header, Extrinsic> {
    pub header: Header,
    pub extrinsics: Vec<Extrinsic>,
}
```
Represents a full block containing metadata and a list of transactions.

#### Header
```rust
pub struct Header<BlockNumber> {
    pub block_number: BlockNumber,
}
```
Contains block metadata. Currently minimal with only block number.

#### Extrinsic
```rust
pub struct Extrinsic<Caller, Call> {
    pub caller: Caller,
    pub call: Call,
    pub signature: crate::crypto::SignatureWrapper,
    pub nonce: u32,
}
```
Represents a single signed transaction with caller information, the function call, signature, and nonce.

### 4. Dispatch Trait

The core abstraction that enables pallet modularity:

```rust
pub trait Dispatch {
    type Caller;
    type Call;
    fn dispatch(&mut self, caller: Self::Caller, call: Self::Call) -> DispatchResult;
}
```

Each pallet implements this trait to define how its calls are executed.

## Block Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    execute_block(block)                     │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Clear previous   │
        │ events           │
        └─────────┬────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Increment block  │
        │ number           │
        └─────────┬────────┘
                  │
                  ▼
        ┌────────────-──────┐    Verify    ┌──────────────┐
        │ For each extrinsic│──────────►   │Nonce matches │
        │    in block       │              │expected value│
        └─────────┬───-─────┘              └──────────────┘
                  │
                  ▼
        ┌──────────────────┐    Verify    ┌──────────────┐
        │                  │──────────►   │Signature is  │
        │ Reconstruct      │              │valid         │
        │ signed payload   │              └──────────────┘
        └─────────┬────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Increment nonce  │
        │ for caller       │
        └─────────┬────────┘
                  │
                  ▼
        ┌──────────────────┐    Check     ┌─────────────┐
        │ Calculate fee    │──────────►   │Caller has   │
        │ and deduct       │              │sufficient   │
        │ from balance     │              │balance      │
        └─────────┬────────┘              └─────────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Emit FeePaid     │
        │ event            │
        └─────────┬────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Dispatch call to │
        │ appropriate      │
        │ pallet           │
        └─────────┬────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Emit event on    │
        │ success          │
        └─────────┬────────┘
                  │
                  ▼
        ┌──────────────────┐
        │ Save state to    │
        │ persistent      │
        │ storage         │
        └──────────────────┘
```

## Procedural Macros

### `#[runtime]` Macro

Expands the `Runtime` struct definition to generate:

1. **RuntimeCall Enum**: Enum with variants for each callable pallet
2. **Dispatch Implementation**: Routes calls to appropriate pallets
3. **Block Execution**: Handles extrinsic processing with signature/nonce verification

### `#[call]` Macro

Expands pallet definitions to generate:

1. **Call Enum**: Enum with variants for each callable function
2. **Dispatch Implementation**: Routes function calls to actual implementations

## Type System

The runtime uses extensive type parameters for configurability:

```rust
mod types {
    pub type AccountId = [u8; 32];
    pub type Balance = u128;
    pub type BlockNumber = u32;
    pub type Nonce = u32;
    pub type Extrinsic = support::Extrinsic<AccountId, crate::RuntimeCall>;
    pub type Header = support::Header<BlockNumber>;
    pub type Block = support::Block<Header, Extrinsic>;
    pub type Content = &'static str;
}
```

This allows easy modification of fundamental types without changing pallet code.

## Security Considerations

### 1. Replay Attack Prevention
- Each account maintains a nonce (transaction counter)
- Every extrinsic must include the expected nonce
- Nonce is verified BEFORE processing the transaction
- Nonce is incremented after successful execution

### 2. Signature Verification
- Every extrinsic must be signed by the caller's private key
- The signature covers both the call data and the nonce
- Verification happens before any state changes
- Uses Ed25519 for cryptographic security

### 3. Block Number Consistency
- Blocks must have sequential, incrementing numbers
- Block number is verified before processing
- Prevents processing blocks out of order

### 4. Transaction Fees
- Every transaction pays a flat fee before execution
- Fee is deducted AFTER verification but BEFORE dispatch
- Insufficient balance causes transaction to fail
- Fee is NOT refunded even if subsequent dispatch fails
- Invalid signatures and replay attacks don't cost fees (verified before fee deduction)

## Storage Model

### In-Memory Storage

Each pallet manages its own storage using simple Rust collections:

- **System**: `BTreeMap<AccountId, Nonce>` for account nonces
- **Balances**: `BTreeMap<AccountId, Balance>` for account balances
- **Fees**: `Balance` for total fees collected
- **Proof of Existence**: `BTreeMap<Content, AccountId>` for claims
- **Events**: `BTreeMap<(BlockNumber, u32), EventRecord>` for event storage

### Persistent Storage

All blockchain state is automatically persisted to disk using sled (an embedded key-value database). The storage system:

1. **State Extraction**: Collects all pallet state after each block execution
2. **Serialization**: Converts complex types (AccountId arrays, &'static str) to serializable formats
3. **Persistence**: Saves complete state atomically to the `db/` directory
4. **Recovery**: On startup, loads existing state or creates fresh runtime with genesis configuration

**Genesis Configuration**: When creating a new runtime (no existing state), you can optionally provide a `GenesisConfig` to initialize the blockchain state. This replaces manual `set_balance()` calls with a formal, declarative configuration.

**Storage Flow**:
```
┌─────────────────────────────────────┐
│     Block Execution Completes       │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│     Extract All Pallet State        │
│  - System (block number, nonces)    │
│  - Balances (account balances)       │
│  - Events (event records)            │
│  - Claims (proof of existence)       │
│  - Fees (total collected)            │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│     Serialize to Binary Format      │
│  (bincode with hex encoding)        │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│     Write to sled Database          │
│  (atomically to db/state)           │
└─────────────────────────────────────┘
```

**Key Design Decisions**:
- **BTreeMap vs HashMap**: Ordered maps provide deterministic iteration
- **Hex Encoding**: Account IDs serialized as hex strings
- **String Leaking**: &'static str claims restored via Box::leak (safe for program duration)
- **Single Entry**: All state in one database entry ensures atomicity

This is a simplified model. Production runtimes use more sophisticated storage with Merkle trees for state verification.
