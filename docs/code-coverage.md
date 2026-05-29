# Code Coverage Enforcement (#555)

Test coverage is measured with [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
and **enforced** in CI: a build fails if coverage drops below the threshold.

## Workflow

`.github/workflows/coverage.yml` runs on every push to `main`, every pull
request, and on manual dispatch.

It runs tarpaulin over the workspace and fails the build via `--fail-under`
when line coverage falls below the threshold:

```bash
cargo tarpaulin --workspace \
  --out Xml --out Lcov --output-dir coverage \
  --fail-under 70
```

The threshold is set by the `COVERAGE_THRESHOLD` env var in the workflow
(currently **70%**). Raise it as coverage improves — never lower it to make a
build pass.

## Codecov gates

Results are uploaded to Codecov, configured in [`codecov.yml`](../codecov.yml):

| Gate            | Target | Meaning                                        |
|-----------------|--------|------------------------------------------------|
| `project`       | 70%    | Overall coverage must stay at/above 70%        |
| `patch`         | 80%    | New/changed lines in a PR must be ≥80% covered  |

The `patch` gate is the important one for day-to-day work: it ensures new code
arrives with tests, even if overall coverage is still climbing.

Test files and fuzz targets are excluded from the coverage denominator (see the
`ignore` list in `codecov.yml`).

## Running locally

```bash
cargo install cargo-tarpaulin --locked
cargo tarpaulin --workspace --out Html --output-dir coverage
# open coverage/tarpaulin-report.html

# Reproduce the CI gate:
cargo tarpaulin --workspace --fail-under 70
```

## Raising coverage

If the gate fails, the report lists uncovered lines. Add tests targeting those
paths — prioritise state-mutating contract entry points and error branches,
which are also tracked by the [Security Audit Checklist](security-audit-checklist.md).
