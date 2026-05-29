# Contract Invariants

This document defines the invariants that must hold for the Atomic Patent smart contracts.

## IP Registry Invariants

### I1: Commitment Uniqueness
- **Definition**: Each IP commitment hash must be globally unique — no two records may share the same hash, regardless of owner.
- **Enforcement**: The contract rejects duplicate commitments from any owner.
- **Tests**: `invariant_i1_commitment_uniqueness`, `invariant_i1_commitment_uniqueness_cross_owner`

### I2: Timestamp Monotonicity
- **Definition**: IP IDs are monotonically increasing, which implies timestamp ordering because the ledger timestamp is non-decreasing.
- **Enforcement**: New commitments always receive a higher ID than all prior commitments.
- **Tests**: `invariant_i2_id_monotonicity`

### I3: Owner Immutability
- **Definition**: An IP record's owner cannot change after creation.
- **Enforcement**: The contract stores owner immutably at commitment time.
- **Tests**: `invariant_i3_owner_immutability`

### I4: Commitment Verification
- **Definition**: `verify_commitment` returns `true` only for the exact `(secret, blinding_factor)` pair used at commit time.
- **Enforcement**: SHA-256 preimage binding — any wrong input produces a different hash.
- **Tests**: `invariant_i4_correct_secret_verifies`, `invariant_i4_wrong_secret_fails`

### I5: Zero Hash Rejection
- **Definition**: The all-zeros commitment hash is always rejected.
- **Enforcement**: Explicit guard in `commit_ip`.
- **Tests**: `invariant_i5_zero_hash_rejected`

### I6: Owner List Consistency
- **Definition**: Every committed IP ID appears in `list_ip_by_owner` for that owner, and the list length equals the number of commits.
- **Enforcement**: Owner index is updated atomically with each commit.
- **Tests**: `invariant_i6_owner_list_consistency`

### I7: Revoked Record Preservation
- **Definition**: Revoking an IP sets `revoked = true` but does not delete the record.
- **Enforcement**: `revoke_ip` updates the flag; `get_ip` still returns the record.
- **Tests**: `invariant_i7_revoked_ip_record_preserved`

## Atomic Swap Invariants

### S1: Fee Accounting
- **Definition**: Total fees collected = sum of all swap fees.
- **Enforcement**: Each swap records its fee; total is auditable.

### S2: Payment Atomicity
- **Definition**: Payment and key reveal must occur together or not at all.
- **Enforcement**: Escrow holds payment until key is revealed; refund if timeout.

### S3: Swap State Transitions
- **Definition**: Swaps follow valid state transitions: `Pending → Accepted → Completed/Cancelled`.
- **Enforcement**: State machine validates transitions.
- **Tests**: `chaos_double_accept_rejected`, `chaos_cancel_after_completion_rejected`, `chaos_accept_after_cancel_rejected`

### S4: Key Validity
- **Definition**: A revealed key must match the commitment hash stored at swap initiation.
- **Enforcement**: Key validation happens before payment release.
- **Tests**: `chaos_wrong_key_rejected`

### S5: Reveal Requires Accepted State
- **Definition**: `reveal_key` is only valid after the buyer has called `accept_swap`.
- **Enforcement**: State guard on `reveal_key`.
- **Tests**: `chaos_reveal_before_accept_rejected`

## Running Invariant Tests

```bash
# IP Registry invariants (property-based)
cargo test invariant_ -p ip_registry

# Atomic Swap chaos/invariant tests
cargo test chaos_ -p atomic_swap
```

## Testing Strategy

### Invariant Checks After Each Operation

1. **After `commit_ip`**: Verify I1, I2, I3, I6
2. **After `verify_commitment`**: Verify I4
3. **After `revoke_ip`**: Verify I7
4. **After `initiate_swap`**: Verify S3
5. **After `accept_swap`**: Verify S3
6. **After `reveal_key`**: Verify S4, S3
7. **After `cancel_swap`**: Verify S3

### Property-Based Testing

Invariants I1–I6 are verified with `proptest` over randomised inputs. Each test runs 256 cases by default. The `proptest!` macro shrinks failing inputs to a minimal reproducer automatically.

## Monitoring

Invariant violations should trigger:
1. Alert to operations team
2. Contract pause (if critical)
3. Forensic analysis of transaction history
4. Potential rollback to last known good state
