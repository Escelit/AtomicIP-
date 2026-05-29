# Load Testing Framework (#552)

## Overview

The load testing framework measures API throughput, latency percentiles, and
error rates under concurrent load. It lives in `api-server/src/load_testing.rs`
and runs as part of the standard test suite.

```bash
# Run against the mock in-process handler (no live server needed)
cargo test load_ -p api-server -- --nocapture

# Run against a live server
LOAD_TEST_BASE_URL=http://localhost:3000 cargo test load_ -p api-server -- --nocapture
```

## Test Scenarios

| Test | Requests | Concurrency | Threshold |
|------|----------|-------------|-----------|
| `load_commit_ip_throughput` | 200 | 20 | p99 < 500 ms, error rate < 1% |
| `load_get_ip_throughput` | 200 | 20 | p99 < 200 ms |
| `load_initiate_swap_throughput` | 200 | 20 | p99 < 500 ms |
| `load_mixed_workload` | 300 | 30 | p95 < 300 ms (70% reads / 30% writes) |
| `load_throughput_baseline` | 500 | 50 | > 100 req/s |

## Report Fields

Each test prints a `LoadTestReport`:

```
=== Load Test Report ===
  Total requests : 200
  Successful     : 200
  Failed         : 0
  Duration       : 312 ms
  Throughput     : 641.0 req/s
  Latency p50    : 1 ms
  Latency p95    : 2 ms
  Latency p99    : 3 ms
  Min / Max      : 0 / 5 ms
```

## Architecture

- `run_load(total, concurrency, task)` — spawns `total` async tasks bounded by a
  `Semaphore` of size `concurrency`, collects `RequestResult` values, and returns
  a `LoadTestReport`.
- `mock_request(endpoint)` — simulates an in-process handler call (no network).
  Replace with a `reqwest::Client` call to test a live server.
- `LoadTestReport::from_results` — computes percentiles and throughput from raw results.

## Adding New Load Tests

1. Add a `#[tokio::test]` function prefixed with `load_` in `api-server/src/load_testing.rs`.
2. Call `run_load(total, concurrency, || mock_request("/your/endpoint"))`.
3. Assert latency and error-rate thresholds.
4. Add the scenario to the table above.
