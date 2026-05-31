# Genesis Configuration

## Overview

The Genesis Configuration system provides a formal way to configure the initial state of the blockchain when creating a new runtime. Instead of manually calling `set_balance()` and other initialization functions, you can define a `GenesisConfig` struct that captures all initial state in a declarative manner.

## Purpose

Before genesis configuration, initial blockchain state was set up manually:

```rust
// Old way - manual initialization
runtime.balances.set_balance(alice_account, 100);
runtime.balances.set_balance(bob_account, 500);
runtime.proof_of_existence.create_claim(alice_account, "my_document")?;
```

With genesis configuration, this becomes:

```rust
// New way - declarative genesis configuration
let genesis = GenesisConfig::builder()
    .add_balance(alice_account, 100)
    .add_balance(bob_account, 500)
    .add_claim("my_document", alice_account)
    .build();
```

## GenesisConfig Struct

The `GenesisConfig` struct is defined in `src/genesis.rs` and contains all initial state:

```rust
pub struct GenesisConfig {
    /// Initial account balances
    pub balances: BTreeMap<AccountId, Balance>,

    /// Initial claims for proof of existence
    pub claims: BTreeMap<Content, AccountId>,

    /// Initial block number (defaults to 0)
    pub block_number: BlockNumber,

    /// Initial account nonces (defaults to empty)
    pub nonces: BTreeMap<AccountId, Nonce>,
}
```

### Fields Explained

| Field | Type | Description |
|-------|------|-------------|
| `balances` | `BTreeMap<AccountId, Balance>` | Initial token balances for accounts |
| `claims` | `BTreeMap<Content, AccountId>` | Pre-populated proof of existence claims |
| `block_number` | `BlockNumber` | Starting block number (typically 0) |
| `nonces` | `BTreeMap<AccountId, Nonce>` | Initial transaction nonces (typically empty) |

## GenesisBuilder

The `GenesisBuilder` provides a fluent API for constructing genesis configurations:

### Basic Usage

```rust
use crate::genesis::GenesisConfig;

let genesis = GenesisConfig::builder()
    .add_balance(alice_account, 1000)
    .build();
```

### Adding Multiple Balances

```rust
let genesis = GenesisConfig::builder()
    .add_balance(alice_account, 1000)
    .add_balance(bob_account, 500)
    .add_balance(charlie_account, 250)
    .build();
```

### Batch Adding Balances

```rust
let balances = vec![
    (alice_account, 1000),
    (bob_account, 500),
    (charlie_account, 250),
];

let genesis = GenesisConfig::builder()
    .add_balances(balances)
    .build();
```

### Adding Claims

```rust
let genesis = GenesisConfig::builder()
    .add_claim("alice_document.pdf", alice_account)
    .add_claim("bob_data.txt", bob_account)
    .build();
```

### Setting Block Number

```rust
let genesis = GenesisConfig::builder()
    .with_block_number(1)  // Start from block 1 instead of 0
    .build();
```

### Complete Example

```rust
let genesis = GenesisConfig::builder()
    .add_balance(alice_account, 1000)
    .add_balance(bob_account, 500)
    .add_claim("genesis_config.md", alice_account)
    .with_block_number(0)
    .build();
```

## Integration with Storage

The genesis configuration integrates with the storage system through the `load_state_or_create` method:

```rust
// Create genesis configuration
let genesis = GenesisConfig::builder()
    .add_balance(alice_account, 100)
    .build();

// Load existing state or create new runtime with genesis config
let mut runtime = storage.load_state_or_create(Some(genesis))
    .expect("Failed to load or create runtime state");
```

If no genesis configuration is provided (`None`), an empty runtime is created:

```rust
let mut runtime = storage.load_state_or_create(None)
    .expect("Failed to load or create runtime state");
```

### State Loading Flow

```
┌─────────────────────────────────────┐
│  Open Storage Database              │
└──────────────┬──────────────────────┘
               │
               ▼
       ┌───────────────┐
       │ Does State    │──── Yes ───► Load Existing State
       │ Exist?        │                 (ignore genesis)
       └───────┬───────┘
               │ No
               ▼
┌─────────────────────────────────────┐
│  Create New Runtime                 │
│  Runtime::new()                      │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Genesis Config Provided?           │
└──────────────┬──────────────────────┘
               │
         Yes ──┴── No
         │         │
         ▼         ▼
    Apply      Return
    Genesis    Empty
    Config     Runtime
```

## Applying Genesis Configuration

The `apply_to` method applies the genesis configuration to a runtime:

```rust
pub fn apply_to(self, runtime: &mut crate::Runtime) {
    // Set the initial block number
    runtime.system.block_number = self.block_number;

    // Set initial nonces
    for (account_id, nonce) in self.nonces {
        runtime.system.nonce.insert(account_id, nonce);
    }

    // Set initial balances
    for (account_id, balance) in self.balances {
        runtime.balances.balances.insert(account_id, balance);
    }

    // Set initial claims
    for (content, account_id) in self.claims {
        runtime.proof_of_existence.claims.insert(content, account_id);
    }
}
```

This is called automatically when using `load_state_or_create(Some(genesis))`.

## Default Configuration

An empty `GenesisConfig` can be created using `default()` or `new()`:

```rust
// Both create an empty genesis configuration
let genesis1 = GenesisConfig::default();
let genesis2 = GenesisConfig::new();
```

This is equivalent to:

```rust
let genesis = GenesisConfig {
    balances: BTreeMap::new(),
    claims: BTreeMap::new(),
    block_number: 0,
    nonces: BTreeMap::new(),
};
```

## Builder Methods Reference

| Method | Parameters | Description |
|--------|------------|-------------|
| `new()` | - | Create a new builder with default values |
| `add_balance(account_id, balance)` | Account ID, Balance | Add a single account balance |
| `add_balances(iterable)` | Iterator of (AccountId, Balance) | Add multiple balances at once |
| `add_claim(content, account_id)` | Content, Account ID | Add a single claim |
| `add_claims(iterable)` | Iterator of (Content, AccountId) | Add multiple claims at once |
| `with_block_number(block_number)` | BlockNumber | Set the initial block number |
| `add_nonce(account_id, nonce)` | Account ID, Nonce | Add an initial nonce for an account |
| `build()` | - | Build the final `GenesisConfig` |

## Testing

The genesis module includes comprehensive unit tests:

```rust
#[test]
fn test_default_genesis() {
    let genesis = GenesisConfig::default();
    assert!(genesis.balances.is_empty());
    assert!(genesis.claims.is_empty());
    assert_eq!(genesis.block_number, 0);
    assert!(genesis.nonces.is_empty());
}

#[test]
fn test_builder_comprehensive() {
    let alice: AccountId = [1u8; 32];
    let bob: AccountId = [2u8; 32];

    let genesis = GenesisConfig::builder()
        .add_balance(alice, 1000)
        .add_balance(bob, 500)
        .add_claim("doc1", alice)
        .add_claim("doc2", bob)
        .with_block_number(1)
        .add_nonce(alice, 0)
        .build();

    assert_eq!(genesis.balances.len(), 2);
    assert_eq!(genesis.balances.get(&alice), Some(&1000));
    assert_eq!(genesis.claims.len(), 2);
    assert_eq!(genesis.block_number, 1);
}
```

Run tests with:
```bash
cargo test genesis
```

## Use Cases

### 1. Development Testing

Set up consistent test scenarios:

```rust
let genesis = GenesisConfig::builder()
    .add_balance(alice, 1000000)
    .add_balance(bob, 500000)
    .build();
```

### 2. Production Launch

Configure initial token distribution:

```rust
let genesis = GenesisConfig::builder()
    .add_balances(vec![
        (treasury_account, 10_000_000),
        (foundation_account, 5_000_000),
        (early_adopter_1, 100_000),
        (early_adopter_2, 100_000),
    ])
    .build();
```

### 3. Network Initialization

Pre-populate governance state:

```rust
let genesis = GenesisConfig::builder()
    .add_balance(council_member_1, 1000)
    .add_balance(council_member_2, 1000)
    .add_claim("constitution", council_member_1)
    .with_block_number(0)
    .build();
```

## Design Benefits

1. **Declarative**: All initial state is defined in one place
2. **Type-Safe**: Builder pattern prevents invalid configurations
3. **Testable**: Easy to create consistent test scenarios
4. **Maintainable**: Single source of truth for initial state
5. **Flexible**: Supports all pallet state initialization

## Future Enhancements

Potential improvements to the genesis system:

1. **JSON Configuration**: Load genesis from JSON/TOML files
2. **Genesis Validation**: Validate genesis constraints (e.g., total supply)
3. **Genesis Presets**: Common configurations for different networks
4. **Merkle Root**: Compute genesis state root for verification
5. **Genesis Hash**: Unique identifier for the genesis configuration
