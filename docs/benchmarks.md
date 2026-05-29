# Performance Benchmarking Suite (#551)

## Overview

Soroban contracts are metered by the host via a **CPU instruction budget** and a
**memory bytes budget**. These budgets are the closest proxy to on-chain gas cost
available in the test environment.

Benchmarks live alongside the contract source and run with `cargo test`:

```bash
cargo test bench_ -p ip_registry
cargo test bench_ -p atomic_swap
```

## Regression Targets

### IP Registry

| Function            | CPU limit (instructions) |
|---------------------|--------------------------|
| `commit_ip`         | 500 000                  |
| `verify_commitment` | 200 000                  |
| `get_ip`            | 100 000                  |
| `list_ip_by_owner`  | 150 000                  |

### Atomic Swap

| Function        | CPU limit (instructions) |
|-----------------|--------------------------|
| `initiate_swap` | 800 000                  |
| `accept_swap`   | 600 000                  |
| `reveal_key`    | 600 000                  |
| `cancel_swap`   | 400 000                  |
| `get_swap`      | 100 000                  |

These limits are conservative upper bounds measured on the current implementation.
A test failure means a change has introduced a performance regression and must be
investigated before merging.

## How to Read Results

`env.budget().cpu_instruction_count()` returns the total CPU instructions consumed
since the last `env.budget().reset_default()` call. The value is deterministic
for a given Soroban SDK version and contract binary — it does not depend on wall
clock time or hardware.

## Adding New Benchmarks

1. Add a `#[test]` function prefixed with `bench_` in the relevant `src/benchmarks.rs` file.
2. Call `env.budget().reset_default()` immediately before the operation under test.
3. Assert against a documented limit with a descriptive panic message.
4. Update the regression targets table above.
