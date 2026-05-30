# Swap Partial Payment (Installments) — #466

Allow buyers to pay for high-value IP in installments rather than a single upfront payment.

## Overview

The installment payment feature lets a seller offer a patent swap where the buyer pays in multiple partial payments. The swap stays in `Pending` state while payments accumulate; once the full price is paid the swap automatically transitions to `Accepted`, at which point the seller can reveal the decryption key.

## API

### `initiate_swap_installment`

```rust
pub fn initiate_swap_installment(
    env: Env,
    token: Address,
    ip_id: u64,
    seller: Address,
    price: i128,
    buyer: Address,
    num_installments: u32,
) -> u64
```

Seller creates an installment swap. `num_installments` is a hint for the buyer indicating the expected number of payments (must be ≥ 1). Returns the swap ID.

### `submit_installment_payment`

```rust
pub fn submit_installment_payment(
    env: Env,
    swap_id: u64,
    payment_amount: i128,
)
```

Buyer submits a partial payment. Tokens are transferred to escrow immediately. When `paid_amount >= price` the swap transitions to `Accepted`.

Panics if:
- Swap is not an installment swap
- Swap is not in `Pending` state
- `payment_amount` is zero or negative
- Payment would exceed the remaining balance (overpayment rejected)

### `get_installment_status`

```rust
pub fn get_installment_status(env: Env, swap_id: u64) -> (i128, i128, i128)
```

Returns `(paid_amount, total_price, remaining)`.

## Flow

```
Seller: initiate_swap_installment(price=300, num_installments=3)
  → swap created: Pending, is_installment=true, paid_amount=0

Buyer: submit_installment_payment(100)  → paid=100, remaining=200
Buyer: submit_installment_payment(100)  → paid=200, remaining=100
Buyer: submit_installment_payment(100)  → paid=300, remaining=0 → status=Accepted

Seller: reveal_key(secret, blinding)    → status=Completed, payment released
```

## SwapRecord Fields

| Field | Type | Description |
|-------|------|-------------|
| `is_installment` | `bool` | `true` for installment swaps |
| `paid_amount` | `i128` | Cumulative amount paid so far |
| `quantity` | `u32` | Stores `num_installments` hint |

## Events

- `swap_init` — emitted when the installment swap is created
- `inst_pay` — emitted on each installment payment: `(swap_id, payment_amount, paid_amount, price)`
- `swap_acpt` — emitted when the final payment completes the full price
