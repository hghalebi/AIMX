# Contributing

Thanks for helping improve AIMX (`apple-intelligence-models`, imported as `aimx`).

This crate is a safe Rust wrapper around Apple's FoundationModels framework. The
project accepts changes that keep the Rust API explicit, typed, documented, and
safe when the Swift bridge is unavailable.

## Development Setup

Required for normal library development:

- Rust 1.85 or newer
- `cargo`

Required for live FoundationModels integration tests:

- macOS 26 or newer
- Apple Silicon
- Apple Intelligence enabled
- Xcode with the macOS 26 SDK

The crate must still compile on hosts that cannot build the Swift bridge. In
that case, public model APIs should return `Error::Unavailable`.

## Quality Gates

Run these before opening a pull request:

```sh
cargo fmt
cargo test
cargo test --examples
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo package
```

Live model tests are ignored by default. Run them on compatible Apple hardware:

```sh
cargo test -- --include-ignored
```

## API Guidelines

- Use typed boundaries instead of raw primitives when values cross FFI,
  generation, or tool-call boundaries.
- Keep public fallible APIs documented with `# Errors`.
- Keep unsafe code private to the FFI layer.
- Preserve the unsupported-host behavior: compile successfully and return
  `Error::Unavailable` at runtime.
- Do not add TODOs, stubbed implementations, or stringly typed error surfaces.
- Do not use `panic!`, `unwrap`, or `expect` in production code. Return typed
  errors through `thiserror` instead.

## Documentation Guidelines

Follow [references/documentation-style.md](references/documentation-style.md)
for README, tutorial, reference, and rustdoc changes.

In short:

- Start with the smallest correct example.
- Explain the single idea the example teaches.
- Name the type or invariant that makes the example safe.
- Document recoverable errors instead of implying success is guaranteed.
- Prefer AIMX-first names such as `AppleIntelligenceModels`,
  `LanguageModelSession`, `GenerationSchema`, `respond_to`, and
  `stream_response`.

## Release Checklist

1. Update `version` in `Cargo.toml`.
2. Update `CHANGELOG.md`.
3. Run all quality gates.
4. Run `cargo package --list` and confirm only intended files are included.
5. Commit the release change.
6. Tag the commit with `vX.Y.Z`, matching the Cargo package version exactly.
7. Push `main` and the tag.
8. Let `.github/workflows/release.yml` publish to crates.io.

The crates.io release workflow uses trusted publishing. In crates.io, configure
`apple-intelligence-models` to trust:

- repository: `hghalebi/AIMX`
- workflow: `release.yml`
- environment: `crates-io`

Do not store a long-lived crates.io API token in GitHub secrets unless trusted
publishing is unavailable.
