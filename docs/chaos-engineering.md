# Chaos Engineering Tests (#550)

## Overview

Chaos tests verify that the Atomic Swap contract handles fault conditions
gracefully — invalid state transitions, wrong credentials, duplicate operations,
and large ledger time jumps.

```bash
cargo test chaos_ -p atomic_swap
```

## Fault Scenarios

| Test | Fault Injected | Expected Outcome |
|------|---------------|-----------------|
| `chaos_double_accept_rejected` | Accept same swap twice | Second call panics (`#6 NotPending`) |
| `chaos_reveal_before_accept_rejected` | Reveal key before buyer accepts | Panics (`#8 NotAccepted`) |
| `chaos_cancel_after_completion_rejected` | Cancel a completed swap | Panics |
| `chaos_wrong_key_rejected` | Wrong decryption key on accepted swap | Panics (`#2 InvalidKey`) |
| `chaos_zero_price_rejected` | Initiate swap with price = 0 | Panics |
| `chaos_accept_after_cancel_rejected` | Accept a cancelled swap | Panics (`#6 NotPending`) |
| `chaos_reveal_by_non_seller_rejected` | Third party reveals key | Panics |
| `chaos_duplicate_active_swap_rejected` | Two active swaps for same IP | Second initiation panics |
| `chaos_state_consistent_after_time_jump` | 30-day ledger advance | Swap remains `Pending` |
| `chaos_repeated_full_lifecycle` | 5 sequential full lifecycles | No state leaks between swaps |

## Design Principles

- Each test targets a single fault in isolation.
- `#[should_panic]` tests assert the contract rejects the invalid operation.
- Non-panic tests assert state consistency after the fault.
- `env.mock_all_auths()` is used so auth failures don't mask contract logic errors.

## Adding New Chaos Tests

1. Identify a state transition or input that should be rejected.
2. Add a `#[test]` prefixed with `chaos_` in `contracts/atomic_swap/src/chaos_tests.rs`.
3. Use `#[should_panic(expected = "Error(Contract, #N)")]` where `N` is the error code, or plain `#[should_panic]` if the exact code is not important.
4. Document the fault and expected outcome in the table above.
