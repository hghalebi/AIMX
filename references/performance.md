# Performance Benchmarks

This document records the local performance harness for AIMX
(`apple-intelligence-models`, imported as `aimx`).

## Scope

The benchmark suite measures deterministic CPU/allocation-bound work in the
safe Rust layer:

- prompt and instruction C-FFI boundary validation
- typed generation option construction and validation
- raw UI/CLI numeric boundary parsing
- `GenerationSchema` construction and JSON serialization
- Rust tool dispatch through the public `Tool` trait
- session-builder validation on hosts where Apple Intelligence is unavailable

It intentionally does not benchmark live Apple Intelligence model latency. Live
generation depends on hardware, OS model state, system load, and prompt content,
so those measurements should be captured as a separate end-to-end application
benchmark on eligible Apple Silicon hardware.

## Command

Run the local microbenchmarks with:

```sh
cargo bench --bench core -- --sample-size 10
```

Use the default Criterion settings for more stable release-quality numbers:

```sh
cargo bench --bench core
```

## Interpreting Results

Criterion reports each benchmark as a latency distribution. For this crate:

- `prompt/new/valid_ascii` and `instructions/new/valid_ascii` should stay in
  the low hundreds of nanoseconds for short ASCII input.
- `generation_options/*` should stay near single-digit nanoseconds because
  typed validation is pure arithmetic and enum construction.
- `generation_schema/serialize_3_fields_json` is expected to be microsecond
  scale because it allocates and serializes JSON.
- `tool/call_success_json_value` includes JSON value cloning and `String`
  allocation for `ToolOutput`.
- `session_builder/validate_then_unavailable` is a host-boundary smoke
  benchmark; on unsupported hosts it should remain cheap and return
  `Error::Unavailable` smoothly.

When optimizing, compare against a checked-in run summary or a saved Criterion
baseline instead of a single noisy local measurement.

## Latest Local Run

Environment:

| Item | Value |
|---|---|
| Timestamp | 2026-04-25 01:41:07 CEST |
| Host | macOS 26.4.1, arm64 |
| Rust | `rustc 1.94.0-nightly (f52090008 2025-12-10)` |
| Command | `cargo bench --bench core -- --sample-size 10` |

Results:

| Benchmark | Mean range |
|---|---:|
| `prompt/new/valid_ascii` | 61.822 ns - 62.118 ns |
| `instructions/new/valid_ascii` | 70.552 ns - 71.637 ns |
| `generation_options/typed_build_and_validate` | 3.0869 ns - 3.0980 ns |
| `generation_options/raw_boundary_build` | 7.2474 ns - 7.2820 ns |
| `generation_schema/build_3_fields` | 194.76 ns - 195.48 ns |
| `generation_schema/serialize_3_fields_json` | 321.77 ns - 324.16 ns |
| `tool/call_success_json_value` | 149.98 ns - 152.30 ns |
| `session_builder/validate_then_unavailable` | 1.1364 ms - 1.1457 ms |

The only millisecond-scale benchmark is unavailable-session construction. On
this host it includes the macOS availability boundary used before returning
`Error::Unavailable`; live model generation latency is outside this local
Rust-layer microbenchmark scope.
