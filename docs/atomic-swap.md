# Atomic Swap Flow

This document describes the trustless patent sale mechanism in AtomicIP.

## Overview

An **atomic swap** allows a seller to exchange an IP decryption key for payment in a single transaction — if the key is invalid, the payment fails automatically. No escrow, no intermediary, no counterparty risk.

---

## Swap Lifecycle

```
┌─────────┐       ┌─────────┐       ┌──────────┐       ┌───────────┐
│ Pending │  -->  │Accepted │  -->  │Completed │       │ Cancelled │
└─────────┘       └─────────┘       └──────────┘       └───────────┘
     │                 │                                      ▲
     │                 └──────────────────────────────────────┘
     └────────────────────────────────────────────────────────┘
```

| State | Description |
|---|---|
| **Pending** | Seller has initiated the swap; buyer has not yet accepted |
| **Accepted** | Buyer has sent payment; waiting for seller to reveal key |
| **Completed** | Seller revealed valid key; payment released; IP transferred |
| **Cancelled** | Swap aborted by seller (if Pending) or buyer (if Accepted + expired) |

---

## Sequence Diagram

```
Seller                  AtomicSwap Contract              IpRegistry              Buyer
  │                            │                            │                      │
  │ 1. initiate_swap()         │                            │                      │
  ├───────────────────────────>│                            │                      │
  │                            │ verify IP ownership        │                      │
  │                            ├───────────────────────────>│                      │
  │                            │<───────────────────────────┤                      │
  │                            │ create SwapRecord          │                      │
  │                            │ status = Pending           │                      │
  │<───────────────────────────┤                            │                      │
  │                            │                            │                      │
  │                            │         2. accept_swap()   │                      │
  │                            │<───────────────────────────┼──────────────────────┤
  │                            │ transfer payment to contract                      │
  │                            │ status = Accepted          │                      │
  │                            ├────────────────────────────┼──────────────────────>│
  │                            │                            │                      │
  │ 3. reveal_key()            │                            │                      │
  ├───────────────────────────>│                            │                      │
  │                            │ verify_commitment()        │                      │
  │                            ├───────────────────────────>│                      │
  │                            │<───────────────────────────┤                      │
  │                            │ if valid:                  │                      │
  │                            │   transfer payment to seller                      │
  │                            │   transfer IP to buyer     │                      │
  │                            │   status = Completed       │                      │
  │<───────────────────────────┤                            │                      │
  │                            │                            │                      │
  │                            │ if invalid:                │                      │
  │                            │   refund buyer             │                      │
  │                            │   status = Cancelled       │                      │
  │                            ├────────────────────────────┼──────────────────────>│
```

---

## Step-by-Step Flow

### 1. Seller Initiates Swap

```rust
let swap_id = atomic_swap.initiate_swap(
    token,        // Payment token address (e.g., XLM)
    ip_id,        // The IP to sell
    seller,       // Seller's address (requires auth)
    price,        // Price in stroops (1 XLM = 10^7 stroops)
    buyer,        // Buyer's address
);
```

**Checks:**
- Seller must own the IP (`IpRegistry.get_ip(ip_id).owner == seller`)
- IP must not be revoked
- No other active swap exists for this `ip_id`
- Price must be > 0

**Result:**
- Swap created with `status = Pending`
- Expiry set to ~7 days from now

---

### 2. Buyer Accepts Swap

```rust
atomic_swap.accept_swap(swap_id);
```

**Checks:**
- Swap must be in `Pending` state
- Buyer must authorize the transaction
- Buyer must have sufficient token balance

**Result:**
- Payment transferred from buyer to contract
- Swap status updated to `Accepted`
- `accept_timestamp` recorded

---

### 3. Seller Reveals Key

```rust
atomic_swap.reveal_key(swap_id, secret, blinding_factor);
```

**Checks:**
- Swap must be in `Accepted` state
- Only seller can call this
- `verify_commitment(ip_id, secret, blinding_factor)` must return `true`

**Result if key is valid:**
- Payment released to seller
- IP ownership transferred to buyer
- Swap status updated to `Completed`

**Result if key is invalid:**
- Payment refunded to buyer
- Swap status updated to `Cancelled`

---

### 4. Cancellation Paths

#### Seller Cancels (Pending Only)

```rust
atomic_swap.cancel_swap(swap_id);
```

Only allowed if swap is still `Pending` (buyer has not yet accepted).

#### Buyer Cancels (Accepted + Expired)

```rust
atomic_swap.cancel_swap(swap_id);
```

Only allowed if:
- Swap is in `Accepted` state
- Current time > `expiry` timestamp
- Seller has not called `reveal_key`

This protects buyers from sellers who accept payment but never reveal the key.

---

## Security Properties

| Property | Enforcement |
|---|---|
| **Atomicity** | Payment and key exchange happen in the same transaction — no partial completion |
| **Trustlessness** | Smart contract verifies the key; no human arbitrator needed |
| **No Escrow Risk** | Payment held by contract, not a third party |
| **Expiry Protection** | Buyers can reclaim funds if seller abandons the swap |
| **Invalid Key Refund** | If `verify_commitment` fails, buyer is automatically refunded |

---

## Example: Full Swap Execution

```rust
// 1. Seller initiates
let swap_id = swap_contract.initiate_swap(
    xlm_token_address,
    ip_id,
    seller_address,
    100_000_000, // 10 XLM
    buyer_address,
);

// 2. Buyer accepts (sends 10 XLM to contract)
swap_contract.accept_swap(swap_id);

// 3. Seller reveals key
swap_contract.reveal_key(swap_id, secret, blinding_factor);

// If key is valid:
//   - Seller receives 10 XLM
//   - Buyer receives IP ownership
//   - Swap status = Completed
```

---

## Common Failure Scenarios

| Scenario | Outcome |
|---|---|
| Seller reveals invalid key | Buyer refunded; swap cancelled |
| Seller never reveals key | Buyer cancels after expiry; refunded |
| Buyer never accepts | Seller cancels; no payment involved |
| IP is revoked before swap completes | `initiate_swap` panics; swap cannot be created |

---

## Gas Optimization

- Use `initiate_swap` once per IP sale (not per negotiation attempt)
- Batch multiple IP sales if selling to the same buyer
- Cancel pending swaps promptly to free storage

---

## Related Documentation

- [Commitment Scheme](commitment-scheme.md) — How to construct valid secrets
- [Security Considerations](security.md) — Best practices for key management
- [Threat Model](threat-model.md) — Attack vectors and mitigations

---

## Price Oracle Integration (#470)

The atomic swap contract supports dynamic pricing via an on-chain price oracle.

### Overview

Instead of the seller specifying a fixed price at initiation time, the price can be fetched from a trusted oracle contract at the moment the swap is created. This enables:

- Market-rate pricing for IP assets
- Automatic price discovery without off-chain coordination
- Slippage protection via min/max price bounds

### Oracle Interface

The oracle contract must expose a single function:

```rust
fn get_price(token: Address) -> i128
```

Returns the current price in stroops (1 XLM = 10^7 stroops) for the given payment token.

### Admin Setup

The contract admin sets the oracle address once:

```rust
atomic_swap.set_oracle(
    admin,          // must be the contract admin
    oracle_address, // address of the oracle contract
    true,           // enabled
);
```

### Oracle-Priced Swap

```rust
let swap_id = atomic_swap.initiate_swap_with_oracle_price(
    token,              // payment token
    ip_id,              // IP to sell
    seller,             // seller address (requires auth)
    buyer,              // buyer address
    0,                  // required_approvals
    None,               // referrer
    0,                  // collateral_amount
    false,              // insurance_enabled
    100_000_000,        // min_price: reject if oracle < 10 XLM
    500_000_000,        // max_price: reject if oracle > 50 XLM (0 = no bound)
);
```

The price used in the swap is the oracle price at call time. If the oracle price falls outside `[min_price, max_price]`, the call panics and no swap is created.

### Query Oracle Price

Off-chain clients can preview the current oracle price before initiating:

```rust
let price = atomic_swap.get_oracle_price(token);
```

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| 46 | `OracleNotConfigured` | No oracle set, or oracle is disabled |
| 47 | `OraclePriceInvalid` | Oracle returned a price ≤ 0 |
| 48 | `OraclePriceBelowMin` | Oracle price is below the caller's `min_price` bound |
| 49 | `OraclePriceAboveMax` | Oracle price is above the caller's `max_price` bound |

### Events

| Topic | Payload | When |
|-------|---------|------|
| `oracle` | `OracleConfigSetEvent { oracle_address, enabled }` | Admin calls `set_oracle` |
| `ora_price` | `OraclePriceUsedEvent { swap_id, oracle_price }` | Oracle-priced swap initiated |
