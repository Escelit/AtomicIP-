# Security Scanning (#553)

Automated security scanning runs in CI/CD to catch vulnerable dependencies,
banned licenses, leaked secrets, and risky code before it reaches `main`.

See also: [Dependency Scanning](dependency-scanning.md) ·
[Security Audit Checklist](security-audit-checklist.md) ·
[Security Policy](../SECURITY.md)

## Workflow

`.github/workflows/security.yml` runs on every push to `main`, every pull
request, weekly (Monday 06:00 UTC), and on manual dispatch.

| Job               | Tool                | What it checks                                       |
|-------------------|---------------------|------------------------------------------------------|
| `cargo-deny`      | cargo-deny          | Advisories, banned crates, license policy, sources   |
| `secret-scan`     | gitleaks            | Committed secrets/keys across full history           |
| `static-analysis` | clippy `-D warnings`| Lint/static-analysis gate, warnings fail the build   |

## cargo-deny

Policy lives in [`deny.toml`](../deny.toml) at the repo root. It enforces:

- **advisories** — RustSec vulnerability and unmaintained-crate database
- **bans** — disallowed crates and duplicate-version warnings
- **licenses** — only the permissive licenses in the `allow` list
- **sources** — crates may only come from crates.io (no unknown registries/git)

Run locally before pushing:

```bash
cargo install cargo-deny --locked
cargo deny check                 # all checks
cargo deny check advisories      # just vulnerabilities
cargo deny check bans licenses sources
```

To accept a specific advisory or license, add it to the relevant `ignore` /
`allow` / `exceptions` section in `deny.toml` with a comment explaining why.

## Secret scanning

`gitleaks` scans the full git history for credentials. If it flags a real
secret: rotate the credential immediately, then scrub history. Never commit
real keys — use the patterns in [`.env.example`](../.env.example) and GitHub
Actions secrets instead.

## Triage

A failing security job blocks merge. Investigate the report, then either fix
the issue (upgrade the dependency, remove the secret) or, for false positives,
record an explicit exception in `deny.toml` with justification.
