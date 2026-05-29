# Dependency Vulnerability Scanning (#554)

The Cargo dependency tree is scanned continuously for known vulnerabilities
(RustSec advisory database) and yanked crates, and kept current with
Dependabot.

See also: [Security Scanning](security-scanning.md) ·
[Security Policy](../SECURITY.md)

## Workflow

`.github/workflows/dependency-scan.yml` runs:

- on push to `main` and on PRs that touch `Cargo.toml`, `Cargo.lock`, or
  `deny.toml`
- **daily** (05:00 UTC) so freshly disclosed advisories are caught quickly
- on manual dispatch

| Job                     | Tool       | Notes                                          |
|-------------------------|------------|------------------------------------------------|
| `cargo-audit`           | cargo-audit| RustSec advisories; opens an issue on schedule |
| `cargo-deny-advisories` | cargo-deny | Second source of truth; fails the PR directly  |

## Running locally

```bash
# cargo-audit
cargo install cargo-audit --locked
cargo audit

# cargo-deny (uses deny.toml)
cargo install cargo-deny --locked
cargo deny check advisories
```

## Handling a reported vulnerability

1. **Upgrade** the affected crate — `cargo update -p <crate>` — to a patched
   version. This resolves the majority of advisories.
2. If no fix exists, evaluate whether the vulnerable code path is reachable
   from the contracts. Document the assessment.
3. As a last resort, add the advisory ID to the `[advisories].ignore` list in
   [`deny.toml`](../deny.toml) with a comment explaining the risk acceptance
   and a tracking link.

## Dependabot

[`.github/dependabot.yml`](../.github/dependabot.yml) opens weekly pull
requests (Mondays) for:

- the Rust workspace (`/`) and the standalone `api-server` crate
- GitHub Actions versions used in the workflows

Dependabot upgrades are the front line of vulnerability management: staying
current means most advisories are already fixed by the time they are disclosed.
