# Cryptography & Transactions

This document covers the cryptographic primitives and transaction signing mechanism used in rsm-en.

## Cryptographic Primitives

**Location**: `src/crypto.rs`

### Overview

rsm-en uses Ed25519 (Edwards-curve Digital Signature Algorithm) for cryptographic operations. Ed25519 provides:
- Fast signature verification
- Strong security guarantees
- Deterministic signatures
- 32-byte public keys and 64-byte signatures

### Key Types

#### PublicKeyWrapper

```rust
#[derive(Debug, Clone, Eq)]
pub struct PublicKeyWrapper(pub VerifyingKey);
```

A 32-byte public key used for:
- Account identification
- Signature verification

**Conversions**:
- `TryFrom<[u8; 32]>` - Create from bytes
- `Into<[u8; 32]>` - Convert to bytes

#### KeypairWrapper

```rust
#[derive(Debug, Clone)]
pub struct KeypairWrapper(pub SigningKey);
```

Contains both the private signing key and can derive the public verifying key.

**Methods**:
- `generate()` - Creates a new random keypair using OsRng
- `public()` - Returns the PublicKeyWrapper
- `sign(message)` - Signs a message and returns SignatureWrapper

#### SignatureWrapper

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureWrapper(pub Signature);
```

A 64-byte Ed25519 signature.

**Conversions**:
- `TryFrom<[u8; 64]>` - Create from bytes
- `Into<[u8; 64]>` - Convert to bytes

### Signed Payload

```rust
pub struct SignedPayload {
    pub call_data: Vec<u8>,
    pub nonce: u32,
}
```

Represents the data that is signed. The payload includes both the call data and nonce to prevent signature replay across different transactions.

**Methods**:
- `new(call_data, nonce)` - Creates a new payload
- `encode()` - Returns the byte representation (call_data || nonce)
- `hash()` - Returns SHA-512 hash of the encoded payload

**Why hash before signing?**
The payload is hashed using SHA-512 before signing because Ed25519 signatures operate on fixed-size messages. The hash provides a deterministic, fixed-size representation of the variable-length call data.

### Verification Function

```rust
pub fn verify(
    public_key: &PublicKeyWrapper,
    signature: &SignatureWrapper,
    message: &[u8],
) -> Result<(), String>
```

Verifies that a signature is valid for a given message and public key.

## Transaction Building

**Location**: `src/transaction.rs`

### TransactionBuilder

Helper for creating signed extrinsics:

```rust
pub struct TransactionBuilder;
```

### Creating a Signed Extrinsic

```rust
pub fn signed_extrinsic<Call>(
    keypair: &KeypairWrapper,
    call: Call,
    nonce: u32,
) -> Extrinsic<[u8; 32], Call>
```

**Process Flow**:

```
┌─────────────────────────────────────────────────────────┐
│  1. Extract public key from keypair                     │
│     let account_id = keypair.public().into()            │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  2. Encode the call data                                │
│     let call_data = serde_json::to_vec(&call)           │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  3. Create signed payload                               │
│     let payload = SignedPayload::new(call_data, nonce)  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  4. Hash the payload                                    │
│     let hash = payload.hash()  // SHA-512               │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  5. Sign the hash                                       │
│     let signature = keypair.sign(&hash)                 │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  6. Create and return Extrinsic                         │
│     Extrinsic { caller, call, signature, nonce }        │
└─────────────────────────────────────────────────────────┘
```

## Extrinsic Verification in Block Execution

During block execution, each extrinsic is verified before processing:

```rust
// 1. Verify nonce
self.system.verify_nonce(&caller, nonce)?;

// 2. Reconstruct the signed payload
let call_bytes = serde_json::to_vec(&call)?;
let payload = SignedPayload::new(call_bytes, nonce);
let payload_hash = payload.hash();

// 3. Verify signature
let public_key = PublicKeyWrapper::try_from(caller)?;
verify(&public_key, &signature, &payload_hash)?;

// 4. Increment nonce
self.system.inc_nonce(&caller);

// 5. Dispatch the call
self.dispatch(caller, call)?;
```

## Security Properties

### 1. Nonce-Based Replay Prevention

Each account maintains a monotonically increasing nonce:
- First transaction: nonce 0
- Second transaction: nonce 1
- etc.

The runtime verifies:
```rust
if provided_nonce != expected_nonce {
    return Err("Nonce mismatch");
}
```

This prevents replay attacks where an attacker would resubmit a previously signed transaction.

### 2. Signature Binding

The signature binds together:
- **Caller's public key** - Proves who authorized the transaction
- **Call data** - Prevents tampering with the function call
- **Nonce** - Prevents signature reuse across different transactions

### 3. Payload Hashing

By hashing the call_data || nonce before signing:
- We get a fixed-size message for Ed25519
- We prevent signature malleability attacks
- We ensure integrity of both call data and nonce

## Example Usage

### Generating Keys
```rust
let keypair = KeypairWrapper::generate();
let public_key = keypair.public();
let account_id: [u8; 32] = public_key.into();
```

### Creating a Transfer
```rust
let call = RuntimeCall::balances(balances::Call::transfer {
    to: bob_account,
    amount: 100,
});

let nonce = runtime.system.nonce(&alice_account);

let extrinsic = TransactionBuilder::signed_extrinsic(
    &alice_keypair,
    call,
    nonce,
);
```

### Verification Failure Cases

1. **Wrong Nonce**:
```rust
// Using nonce 0 when expected nonce is 3
let result = runtime.execute_block(block);
// Returns: Err("Nonce verification failed")
```

2. **Invalid Signature**:
```rust
// Using different keypair to sign
let wrong_extrinsic = TransactionBuilder::signed_extrinsic(
    &different_keypair,
    call,
    nonce,
);
let result = runtime.execute_block(block);
// Returns: Err("Invalid signature")
```

3. **Tampered Call Data**:
```rust
// Modifying call after signing
let mut tampered = signed_extrinsic;
tampered.call = different_call;
let result = runtime.execute_block(block);
// Returns: Err("Invalid signature")
```

## Encoding Format

Currently, rsm-en uses JSON encoding for call data:

```rust
pub fn encode_call<Call>(call: &Call) -> Vec<u8>
where
    Call: serde::Serialize,
{
    serde_json::to_vec(call).expect("Failed to encode call")
}
```

**Note**: Production blockchains like Polkadot use SCALE (Simple Concatenated Aggregate Little-Endian) codec for more efficient binary encoding.

## Future Improvements

1. **SCALE Codec**: Replace JSON with SCALE for compact binary encoding
2. **Multi-sig**: Support for multi-signature accounts
3. **Batch Transactions**: Allow multiple calls in one extrinsic
4. **Tip/Fee**: Add optional tip/fee field to extrinsic
5. **Mortal/Immortal**: Add era information for transaction mortality
