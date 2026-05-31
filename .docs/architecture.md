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
    proof_of_existence: proof_of_existence::Pallet<Runtime>,
}
```

**Generated Code**: The `#[runtime]` procedural macro automatically generates:
- `Runtime::new()` - Constructor initializing all pallets
- `Runtime::execute_block()` - Block execution logic
- `RuntimeCall` enum - Enum of all callable pallet functions
- `Dispatch` trait implementation - Call routing logic

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

## Storage Model

Each pallet manages its own storage using simple Rust collections:

- **System**: `HashMap<AccountId, Nonce>` for account nonces
- **Balances**: `HashMap<AccountId, Balance>` for account balances
- **Proof of Existence**: `HashMap<Content, AccountId>` for claims
- **Events**: `Vec<(BlockNumber, Phase, Event)>` for event storage

This is a simplified model. Production runtimes use more sophisticated storage with Merkle trees for state verification.
