# Pallets Documentation

This document describes each pallet in the rsm-en runtime.

## Pallet System Overview

A pallet is a modular unit of blockchain logic. Each pallet:
- Manages its own storage
- Exposes callable functions via the `Call` enum
- Is configured through a `Config` trait
- Implements the `Dispatch` trait

## System Pallet

**Location**: `src/system.rs`

### Purpose
Provides system-level functionality including:
- Block number tracking
- Account nonce management for replay attack prevention

### Config
```rust
pub trait Config {
    type AccountId;
    type BlockNumber;
    type Nonce;
}
```

### Storage
- `block_number: BlockNumber` - Current block number
- `nonce: HashMap<AccountId, Nonce>` - Transaction count per account

### Functions
- `new()` - Creates a new System pallet instance
- `block_number()` - Returns the current block number
- `inc_block_number()` - Increments the block number by 1
- `nonce(account_id)` - Returns the nonce for an account
- `inc_nonce(account_id)` - Increments the nonce for an account
- `verify_nonce(account_id, expected_nonce)` - Verifies nonce matches expected value

### Usage Example
```rust
// Get current nonce for an account
let nonce = runtime.system.nonce(&alice_account);

// Increment nonce after transaction
runtime.system.inc_nonce(&alice_account);
```

---

## Balances Pallet

**Location**: `src/balances.rs`

### Purpose
Manages account balances and token transfers.

### Config
```rust
pub trait Config {
    type Balance;
}
```

### Storage
- `balances: HashMap<AccountId, Balance>` - Account balances

### Call Types
```rust
pub enum Call<T: Config> {
    transfer { to: AccountId, amount: Balance },
}
```

### Functions
- `new()` - Creates a new Balances pallet instance
- `set_balance(account_id, balance)` - Sets the balance for an account (for initialization)
- `balance(account_id)` - Returns the balance for an account
- `transfer(caller, to, amount)` - Transfers amount from caller to recipient

### Transfer Rules
1. Caller must have sufficient balance
2. Balance is deducted from caller
3. Balance is added to recipient
4. Returns error if insufficient funds

### Usage Example
```rust
// Set initial balance
runtime.balances.set_balance(alice, 100);

// Transfer tokens
runtime.balances.transfer(alice, bob, 30)?;
```

---

## Proof of Existence Pallet

**Location**: `src/proof_of_existence.rs`

### Purpose
Allows users to prove ownership/existence of data by creating claims. Later claimers can revoke existing claims.

### Config
```rust
pub trait Config {
    type Content;
}
```

### Storage
- `claims: HashMap<Content, AccountId>` - Mapping of content to owner

### Call Types
```rust
pub enum Call<T: Config> {
    create_claim { claim: Content },
    revoke_claim { claim: Content },
}
```

### Functions
- `new()` - Creates a new Proof of Existence pallet instance
- `create_claim(caller, claim)` - Creates a new claim owned by caller
- `revoke_claim(caller, claim)` - Revokes an existing claim owned by caller
- `get_claim(claim)` - Returns the owner of a claim

### Claim Rules
1. **create_claim**: Succeeds only if claim doesn't exist
2. **revoke_claim**: Succeeds only if claim exists and caller is the owner

### Usage Example
```rust
// Create a claim
runtime.proof_of_existence.create_claim(alice, "my_document")?;

// Revoke a claim
runtime.proof_of_existence.revoke_claim(alice, "my_document")?;
```

---

## Events Pallet

**Location**: `src/events.rs`

### Purpose
Records and indexes events emitted during block execution for later querying.

### Config
```rust
pub trait Config {
    type Event;
}
```

### Storage
- `events: Vec<(BlockNumber, Phase, Event)>` - Indexed list of events

### Event Types
Defined in `src/event.rs`:
```rust
pub enum Event<AccountId, Balance, Content> {
    BalanceTransfer(AccountId, AccountId, Balance),
    ClaimCreated(AccountId, Content),
    ClaimRevoked(AccountId, Content),
}
```

### Phase Types
```rust
pub enum Phase {
    Initialize,
    ApplyExtrinsic(u32),
    Finalize,
}
```

### Functions
- `new()` - Creates a new Events pallet instance
- `deposit_event(block_number, phase, event)` - Records a new event
- `events_at_block(block_number)` - Returns all events for a specific block
- `clear_events()` - Removes all events (called at start of each block)

### Event Emission
Events are automatically emitted by the runtime after successful extrinsic execution:
1. Successful balance transfer → `BalanceTransfer` event
2. Successful claim creation → `ClaimCreated` event
3. Successful claim revocation → `ClaimRevoked` event

### Usage Example
```rust
// Query events for block 1
for event in runtime.events.events_at_block(1) {
    println!("{:?}", event);
}
```

### Event Storage Flow
```
┌─────────────────────────────────────┐
│  Block Execution Starts             │
│  clear_events() called              │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Extrinsic Executed Successfully    │
│  deposit_event(...) called          │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Event Stored with:                 │
│  - Block Number                     │
│  - Phase (Extrinsic Index)          │
│  - Event Data                       │
└─────────────────────────────────────┘
```

---

## Adding a New Pallet

To add a new pallet to the runtime:

1. **Create the pallet module** (`src/my_pallet.rs`):

```rust
use crate::support::Dispatch;

pub struct Pallet<T: Config> {
    // Your storage here
}

pub trait Config {
    // Associated types here
}

#[macros::call]
impl<T: Config> Pallet<T> {
    // Your callable functions here
}

impl<T: Config> Dispatch for Pallet<T> {
    type Caller = T::AccountId;
    type Call = Call<T>;
    // dispatch implementation here
}
```

2. **Add module declaration** in `main.rs`:
```rust
mod my_pallet;
```

3. **Add pallet to Runtime struct**:
```rust
#[macros::runtime]
pub struct Runtime {
    // ... existing pallets
    my_pallet: my_pallet::Pallet<Runtime>,
}
```

4. **Implement Config trait**:
```rust
impl my_pallet::Config for Runtime {
    // Configure associated types
}
```

The `#[runtime]` macro will automatically generate the necessary dispatch code.
