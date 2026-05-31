# Events System

This document covers the events system in rsm-en, which records and queries events emitted during block execution.

## Overview

The events system provides:
- **Event Recording**: Automatic emission of events on successful extrinsic execution
- **Event Querying**: Retrieve events by block number
- **Event Indexing**: Events are tagged with block number and execution phase
- **Type Safety**: Events are strongly-typed through enums

## Components

### Event Pallet

**Location**: `src/events.rs`

The events pallet manages event storage and querying.

**Storage**:
```rust
pub struct Pallet<T: Config> {
    // Stores (block_number, phase, event) tuples
    events: Vec<(BlockNumber, Phase, T::Event)>,
}
```

**Config Trait**:
```rust
pub trait Config {
    type Event;
}
```

### Event Types

**Location**: `src/event.rs`

Events are defined as an enum with variants for each event type:

```rust
pub enum Event<AccountId, Balance, Content> {
    BalanceTransfer(AccountId, AccountId, Balance),
    ClaimCreated(AccountId, Content),
    ClaimRevoked(AccountId, Content),
}
```

**Event Variants**:

| Event | Emitted When | Data |
|-------|--------------|------|
| `BalanceTransfer` | Successful token transfer | (from_account, to_account, amount) |
| `ClaimCreated` | Successful claim creation | (account, claim_content) |
| `ClaimRevoked` | Successful claim revocation | (account, claim_content) |

### Phase Types

Events are tagged with when they occurred during block execution:

```rust
pub enum Phase {
    Initialize,       // Before any extrinsics
    ApplyExtrinsic(u32), // During specific extrinsic (by index)
    Finalize,         // After all extrinsics
}
```

Currently, rsm-en only emits events during extrinsic execution (`ApplyExtrinsic`).

## Event Emission

Events are emitted in the `execute_block` function after successful extrinsic execution:

```rust
// After successful dispatch
if res.is_ok() {
    let phase = event::Phase::ApplyExtrinsic(i as u32);

    match call_clone {
        RuntimeCall::balances(balances::Call::transfer { to, amount }) => {
            self.events.deposit_event(
                block.header.block_number,
                phase,
                event::Event::BalanceTransfer(caller, to, amount),
            );
        },
        RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim { claim }) => {
            self.events.deposit_event(
                block.header.block_number,
                phase,
                event::Event::ClaimCreated(caller, claim),
            );
        },
        // ... other event types
    }
}
```

## Event Storage Flow

```
┌─────────────────────────────────────────────────────────┐
│  Block Execution Begins                                 │
│  events.clear_events()                                  │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  Extrinsic 0 Executed Successfully                      │
│  events.deposit_event(block_num, ApplyExtrinsic(0), ...)│
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  Extrinsic 1 Executed Successfully                      │
│  events.deposit_event(block_num, ApplyExtrinsic(1), ...)│
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  Block Execution Complete                               │
│  Events stored for later query                          │
└─────────────────────────────────────────────────────────┘
```

## Event Querying

### Query by Block Number

```rust
for event in runtime.events.events_at_block(1) {
    println!("{:?}", event);
}
```

**Returns**: Iterator over all events from the specified block.

**Output Format**:
```rust
(
    1,                              // Block number
    Phase::ApplyExtrinsic(0),       // Phase
    Event::BalanceTransfer(         // Event data
        [alice_public_key_bytes],
        [bob_public_key_bytes],
        30
    )
)
```

## Events Pallet API

### Functions

- `new()` - Creates a new Events pallet instance
- `deposit_event(block_number, phase, event)` - Records a new event
- `events_at_block(block_number)` - Returns iterator over events for a block
- `clear_events()` - Removes all stored events

### Usage Example

```rust
// After executing a block
runtime.execute_block(block)?;

// Query events for block 1
println!("\n=== Events in Block 1 ===");
for event in runtime.events.events_at_block(1) {
    println!("  {:?}", event);
}
```

## Adding New Event Types

To add a new event type:

1. **Add variant to Event enum** (`src/event.rs`):
```rust
pub enum Event<AccountId, Balance, Content> {
    BalanceTransfer(AccountId, AccountId, Balance),
    ClaimCreated(AccountId, Content),
    ClaimRevoked(AccountId, Content),
    NewEventType(AccountId, SomeData),  // Add new variant
}
```

2. **Update Config associated types** where needed:
```rust
impl events::Config for Runtime {
    type Event = event::Event<types::AccountId, types::Balance, types::Content>;
}
```

3. **Emit the event** in `execute_block`:
```rust
match call_clone {
    // ... existing cases
    RuntimeCall::your_pallet(your_pallet::Call::your_function { data }) => {
        self.events.deposit_event(
            block.header.block_number,
            phase,
            crate::event::Event::NewEventType(caller, data),
        );
    },
}
```

## Event Storage Considerations

### Current Implementation

Events are stored in a simple `Vec`:
- Simple to implement
- Easy to query by iterating
- Memory usage grows unbounded

### Production Considerations

In production systems, event storage typically:
- Prunes old events after a certain block height
- Uses more efficient indexing structures
- May support event subscriptions
- Often includes event metadata (topics, etc.)

## Example Output

When running `cargo run`, you'll see event output like:

```
=== Events in Block 1 ===
  (
      1,
      ApplyExtrinsic(0),
      BalanceTransfer(
          [123, 45, 67, ...],  // Alice's public key
          [89, 10, 11, ...],   // Bob's public key
          30                   // Amount
      )
  )
  (
      1,
      ApplyExtrinsic(1),
      BalanceTransfer(
          [123, 45, 67, ...],  // Alice's public key
          [21, 22, 23, ...],   // Charlie's public key
          20                   // Amount
      )
  )
```

## Events vs. Return Values

| Aspect | Return Values | Events |
|--------|---------------|---------|
| Purpose | Indicate success/failure | Record what happened |
| Scope | Per-extrinsic | Per-block (queried) |
| Persistence | Not stored | Stored indefinitely |
| Query | Immediate | By block number |
| Use Case | Error handling | Audit trails, UI updates |

## Testing with Events

Events are useful for testing:
```rust
// Execute block
runtime.execute_block(block).unwrap();

// Verify expected events were emitted
let block_events: Vec<_> = runtime.events.events_at_block(1).collect();

// Assertions
assert_eq!(block_events.len(), 2);
assert!(matches!(block_events[0].2, Event::BalanceTransfer(_, _, 30)));
```
