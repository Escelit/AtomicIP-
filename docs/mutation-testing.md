# Mutation Testing (#378)

## Overview

Mutation testing verifies that the test suite actually catches logic errors.
A *mutant* is a copy of the source with one small change (e.g. `>` → `>=`,
`true` → `false`). If all tests still pass for a mutant, the test suite has a
gap.

## Tool

[cargo-mutants](https://mutants.rs) — install with:

```bash
cargo install cargo-mutants
```

## Running

```bash
# Both contracts
cargo mutants -p ip_registry -p atomic_swap

# Single contract
cargo mutants -p ip_registry
```

Configuration is in `.cargo-mutants.toml` at the repo root.

## Mutation-Catching Tests

Dedicated tests live in `src/mutation_tests.rs` for each contract.
They are designed to kill the most common mutant classes:

| Mutant class                  | Killed by                                      |
|-------------------------------|------------------------------------------------|
| Remove zero-hash check        | `zero_hash_is_rejected`                        |
| Off-by-one in ID counter      | `ids_are_sequential_starting_at_one`           |
| Remove duplicate-hash check   | `duplicate_hash_is_rejected`                   |
| Flip `revoked = true`         | `revoked_flag_is_set_after_revoke`             |
| Skip owner-index append       | `owner_index_contains_committed_ids`           |
| Always-true verify            | `verify_commitment_rejects_wrong_secret`       |
| Always-false verify           | `verify_commitment_accepts_correct_secret`     |
| Skip status → Pending         | `initiate_swap_sets_pending_status`            |
| Skip status → Accepted        | `accept_swap_sets_accepted_status`             |
| Skip status → Completed       | `reveal_key_sets_completed_status`             |
| Skip commitment verification  | `reveal_key_rejects_wrong_secret`              |
| Remove price > 0 check        | `zero_price_is_rejected`                       |
| Allow double-accept           | `accept_swap_twice_is_rejected`                |
| Allow reveal on Pending       | `reveal_key_on_pending_swap_is_rejected`       |

## Interpreting Results

- **Killed** — the mutant caused at least one test to fail. Good.
- **Survived** — no test caught the mutation. Add a test targeting that line.
- **Timeout** — the mutant caused an infinite loop. Treated as killed.
- **Unviable** — the mutant did not compile. Ignored.

## Baseline Results

Mutation testing was run against the current codebase. All mutants in the
core validation paths (`require_non_zero_commitment`, `require_unique_commitment`,
`require_positive_price`, status transition assignments) are killed by the
`mutation_tests` module.

## CI/CD Integration (#556)

Mutation testing runs automatically via `.github/workflows/mutation.yml`:

| Trigger              | Scope                              |
|----------------------|------------------------------------|
| Pull request         | Only lines changed in the PR diff (`--in-diff`) — fast |
| Weekly schedule (Mon)| Full sweep over both contracts     |
| Manual dispatch      | Full sweep                         |

The PR-scoped run keeps feedback fast while ensuring new/changed logic is
covered by mutation-killing tests. The full report is uploaded as the
`mutants-report` build artifact (`mutants.out/`).

If a PR introduces a **surviving** mutant, add a test (typically in
`src/mutation_tests.rs`) that fails for that mutation, then re-run.
