# Changelog

All notable changes to AIMX (`apple-intelligence-models`, imported as `aimx`) are documented here.

This project follows semantic versioning.

## 0.2.1 - 2026-04-25

### Added

- Original AIMX SVG logo and README placement.
- GitHub Actions CI for Rust library checks across Linux and macOS.
- GitHub Pages workflow that builds and deploys rustdoc from `main`.
- crates.io release workflow using trusted publishing from version tags.

### Changed

- Package metadata now includes repository assets so the README logo ships with
  future crate packages.
- Release documentation now points maintainers to the GitHub Actions release
  path instead of local publishing as the default.

## 0.2.0 - 2026-04-25

### Added

- Preferred AIMX-aligned `AppleIntelligenceModels`, `LanguageModelSession`, `Prompt`, `SystemInstructions`, `GenerationSchema`, and `AvailabilityError` public names.
- Apple-style `respond_to`, `respond_generating`, and `stream_response` methods, plus MLX-style `generate` and `stream_generate` aliases.
- Session builder APIs with Rig-compatible `agent()` and `preamble()` aliases.
- Typed newtype boundaries for prompts, instructions, response text, temperature, max tokens, schema names, tool names, tool output, generation errors, and tool-call errors.
- `LanguageModel` and `GenerateText` traits for provider-style generation boundaries, plus `CompletionModel` compatibility naming.
- Public integration tests and property tests for C-string and numeric FFI boundaries.
- Criterion benchmarks for deterministic Rust-layer performance boundaries.
- Crate-level rustdoc with platform requirements, examples, error handling, panic behavior, and safety notes.
- An extensive [TUTORIAL.md](TUTORIAL.md) covering availability checks, one-shot responses, sessions, streaming, structured output, tools, testing, and troubleshooting.
- Repository documentation style guide in [references/documentation-style.md](references/documentation-style.md).
- `examples/quickstart.rs` and `examples/agent_use_cases.rs`.

### Changed

- The crate identity is now AIMX: package `apple-intelligence-models`, Rust import `aimx`.
- README, tutorial, contributing guide, and crate-level rustdoc now use a more official Rust documentation structure with a clearer teaching sequence.
- `ResponseStream` now yields typed `ResponseText` chunks instead of raw `String` values.
- Older names such as `SystemLanguageModel`, `FoundationModels`, `Client`, `Session`, `ResponseSchema`, `Schema`, and `UnavailabilityReason` remain as compatibility aliases.
- Tool handlers now return `ToolResult`, which is `Result<ToolOutput, ToolCallError>`.
- `GenerationOptions` uses builder-style setters and validates through `Temperature` and `MaxTokens`.
- Swift session handles are owned by an internal `SessionHandle` newtype.
- In-flight responses and streams now retain a cloned session handle until Swift
  calls the completion callback, making caller cancellation safe for the FFI
  handle lifetime.
- Tool-handler panics are converted into `ToolCallError` instead of unwinding
  through the tool callback path.
- Minimum supported Rust version is now 1.85.

### Fixed

- `max_tokens` values larger than `i64::MAX` are rejected before crossing the Swift bridge.
- Null-byte validation now happens at typed prompt and instruction boundaries before availability checks.
- Prompt and instruction text wrappers no longer use panic-prone UTF-8 `expect` checks after construction.
- The build script now reports missing Cargo environment through a typed `thiserror` error instead of panicking.
- Cargo packaging now includes integration tests and examples.

## 0.1.0

- Initial public crate release.
