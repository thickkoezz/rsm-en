# Persistent Storage

## Overview

The persistent storage feature enables the blockchain to save and restore its state across program restarts. This is a critical feature for any production blockchain, as it ensures that:

1. All account balances are preserved
2. Transaction history (nonces) is maintained
3. Claims and events persist
4. The blockchain can resume from where it left off

## Implementation Details

### Storage Backend

We use **sled** as the embedded key-value storage engine. Sled is:
- Written in pure Rust
- Thread-safe and performant
- ACID-compliant (atomic, consistent, isolated, durable)
- Easy to integrate with existing code

### State Serialization

State is serialized using **bincode** (a binary serialization format) and stored in a single entry in the database. This includes:

1. **System State**
   - Current block number
   - Account nonces (transaction counters)

2. **Balances State**
   - Account ID → Balance mapping

3. **Proof of Existence State**
   - Content → Owner Account mapping

4. **Events State**
   - (Block Number, Extrinsic Index) → Event Record mapping

5. **Fees State**
   - Total fees collected

### Key Design Decisions

1. **Hex Encoding for Binary Keys**: Account IDs are `[u8; 32]` arrays, which can't be directly used as keys in many serialization formats. We convert them to hex strings for serialization.

2. **String Leaking for Static Content**: The proof of existence pallet uses `&'static str` for content. When loading from storage, we leak strings to convert them to `'static` lifetime. This is safe because the strings live for the program's entire duration.

3. **Single Entry Storage**: All state is stored in a single database entry keyed by `"state"`. This simplifies the implementation and ensures atomic consistency - either all state loads or none does.

## Usage

### Basic Usage

```rust
// Open or create the storage database
let storage = storage::Storage::open("db")
    .expect("Failed to open storage database");

// Create genesis configuration for initial blockchain state
let genesis = crate::genesis::GenesisConfig::builder()
    .add_balance(alice_account, 1000)
    .build();

// Load existing state or create a new runtime with genesis config
let mut runtime = storage.load_state_or_create(Some(genesis))
    .expect("Failed to load or create runtime state");

// Execute blocks...

// Save state after execution
storage.save_state(&runtime).expect("Failed to save state");
```

**Note**: If you don't need a custom genesis configuration, you can pass `None`:
```rust
let mut runtime = storage.load_state_or_create(None)
    .expect("Failed to load or create runtime state");
```

### In-Memory Storage (Testing)

```rust
// Create an in-memory database that doesn't persist to disk
let storage = storage::Storage::in_memory()
    .expect("Failed to create in-memory storage");
```

### Checking for Existing State

```rust
if storage.has_state() {
    println!("Found existing blockchain state");
} else {
    println!("No existing state - will create new blockchain");
}
```

### Clearing State

```rust
// Remove all persisted state (useful for resetting the blockchain)
storage.clear().expect("Failed to clear state");
```

## File Structure

The state is stored in a directory named `db` in the current working directory. The directory structure:

```
db/
├── state                 # Serialized blockchain state
├── conf                  # Sled configuration
└── ...                   # Other sled internal files
```

## Error Handling

The storage module uses `Result` types with `Box<dyn std::error::Error>` for flexible error handling:

```rust
pub fn save_state(&self, runtime: &Runtime) -> Result<(), Box<dyn std::error::Error>>
pub fn load_state_or_create(&self) -> Result<Runtime, Box<dyn std::error::Error>>
```

## Integration with Runtime

The storage module integrates with the main runtime through:

1. **Initialization**: Load state when the program starts
2. **Persistence**: Save state after each block execution
3. **Recovery**: Automatically restore full state on restart

## Testing

The storage module includes comprehensive unit tests:

- `test_in_memory_storage`: Verifies in-memory database creation
- `test_save_and_load`: Tests saving and loading state
- `test_load_without_state`: Tests default runtime creation
- `test_clear`: Tests state clearing

Run tests with:
```bash
cargo test storage
```

## Future Enhancements

Potential improvements to the storage system:

1. **Incremental Updates**: Save only changed state instead of full state
2. **Snapshots**: Create periodic state snapshots for rollback capability
3. **Pruning**: Remove old events to limit storage growth
4. **Merkle Tree**: Implement a Merkle tree for state verification
5. **Multiple Backends**: Support different storage engines (RocksDB, etc.)

## Security Considerations

1. **Access Control**: The `db` directory should be protected with appropriate file permissions
2. **Backup**: Regular backups of the `db` directory are recommended
3. **Consistency**: The ACID properties of sled ensure state consistency even after crashes

## Performance Notes

- sled is optimized for SSD storage but works on HDDs
- Write operations are batched for efficiency
- The single-entry design ensures atomicity but may become a bottleneck for very large state
- Consider splitting state into multiple entries for better performance at scale
