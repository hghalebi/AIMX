# AIMX Tutorial

This tutorial teaches AIMX the way a good Rust crate should be learned: one
small invariant at a time, with examples that make the failure modes visible.
AIMX is the `apple-intelligence-models` package, imported in Rust as `aimx`.

By the end, you will have code that can:

1. Check whether Apple Intelligence is available.
2. Send a one-shot prompt.
3. Build a reusable session with instructions and generation options.
4. Stream response chunks.
5. Request structured JSON output.
6. Register a Rust tool that the model can call.
7. Run a tested gallery of agent use cases.
8. Handle unavailable hardware, invalid input, and model errors without panics.

## The Big Idea

The simplest way to understand AIMX is:

> Build a safe Rust boundary around the local Apple Intelligence model.

That boundary has three jobs.

1. Check availability before the user waits on work that cannot run.
2. Convert raw text, numbers, schemas, and tool arguments into typed Rust values.
3. Return typed errors instead of panicking when the platform, input, or model
   response is not usable.

This is the same shape you see in the standard Rust documentation: start with a
minimal working example, then explain the type that makes the example safe, then
show how errors are reported.

## Naming Map

AIMX uses a few names because Rust package names, Rust crate imports, and Apple
framework names serve different purposes.

| Role | Name |
|---|---|
| Project brand | AIMX |
| Cargo package | `apple-intelligence-models` |
| Rust import | `aimx` |
| Preferred model handle | `AppleIntelligenceModels` |
| Apple framework | `FoundationModels.framework` |
| Internal bridge cfg | `aimx_bridge` |

Use `AppleIntelligenceModels` in new application code. Compatibility aliases
such as `SystemLanguageModel`, `FoundationModels`, and `Client` still compile,
but examples and docs use the AIMX-first naming.

## Learning Path

The tutorial is ordered so each step introduces one new idea:

| Step | New concept | Why it matters |
|---|---|---|
| 1 | Availability | Avoid work the local platform cannot complete. |
| 2 | One-shot response | Learn the smallest async call. |
| 3 | Session builder | Put instructions and defaults in one visible place. |
| 4 | Typed options | Keep raw primitives at input boundaries. |
| 5 | Streaming | Handle incremental model output. |
| 6 | Structured output | Ask for JSON that maps to Rust types. |
| 7 | Tools | Let the model call recoverable Rust functions. |
| 8 | Tested agent examples | Keep examples deterministic even without Apple hardware. |
| 9 | Error handling | Turn platform and input failures into user-facing actions. |
| 10 | Integration | Combine the pieces into a small application. |

## Prerequisites

For normal compilation and tests:

| Requirement | Value |
|---|---|
| Rust | 1.85 or newer |
| Cargo | Included with Rust |
| OS | Any host that can compile Rust |

For live model responses:

| Requirement | Value |
|---|---|
| macOS | 26 (Tahoe) or newer |
| Hardware | Apple Silicon M1 or newer |
| System setting | Apple Intelligence enabled |
| SDK | Xcode with the macOS 26 SDK |

AIMX is intentionally graceful on unsupported hosts. If the Swift bridge cannot
be compiled, or the current Mac cannot run Apple Intelligence, model calls
return `Err(Error::Unavailable(...))` instead of panicking or failing at link
time.

## Create A Tutorial Project

Create a new Rust binary project:

```sh
cargo new aimx-tutorial
cd aimx-tutorial
```

Add AIMX and a small async runtime to `Cargo.toml`:

```toml
[dependencies]
aimx = { package = "apple-intelligence-models", version = "0.2.1" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

AIMX itself is runtime-agnostic. Tokio is used here only to keep the tutorial
binary short and familiar.

## Step 1: Check Availability

Start with availability. This lets your application show a good fallback before
the user waits on a model call that cannot run on the current machine.

```rust
use aimx::{availability, AvailabilityError};

fn print_availability() -> bool {
    match availability() {
        Ok(()) => {
            println!("Apple Intelligence is ready.");
            true
        }
        Err(AvailabilityError::DeviceNotEligible) => {
            eprintln!("This device cannot run the local Apple Intelligence model.");
            false
        }
        Err(AvailabilityError::NotEnabled) => {
            eprintln!("Enable Apple Intelligence in System Settings.");
            false
        }
        Err(AvailabilityError::ModelNotReady) => {
            eprintln!("The on-device model is still downloading or preparing.");
            false
        }
        Err(AvailabilityError::Unknown) => {
            eprintln!("Apple Intelligence is unavailable for an unknown reason.");
            false
        }
    }
}
```

The same check is also available on the preferred model handle:

```rust
use aimx::AppleIntelligenceModels;

let model = AppleIntelligenceModels::new();
let ready = model.is_available();
```

## Step 2: Send A One-Shot Prompt

Use `respond` for the smallest possible request. It creates a fresh session,
sends one prompt, and returns the model text as a `String`.

```rust
use aimx::{is_available, respond, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    if !is_available() {
        eprintln!("Apple Intelligence is not available on this device.");
        return Ok(());
    }

    let answer = respond("Explain Rust ownership in one sentence.").await?;
    println!("{answer}");

    Ok(())
}
```

This is the right API for quick commands, checks, and scripts. If your program
needs instructions, tools, or repeated turns, build a session instead.

## Step 3: Build A Stateful Session

A `LanguageModelSession` keeps instructions and conversation context together.
Use the builder from `AppleIntelligenceModels::default().session()`.

```rust
use aimx::{AppleIntelligenceModels, Error, MaxTokens, Temperature};

async fn answer_rust_question() -> Result<(), Error> {
    let session = AppleIntelligenceModels::default()
        .session()
        .instructions("You are a concise Rust tutor. Prefer short examples.")
        .temperature(Temperature::new(0.2)?)
        .max_tokens(MaxTokens::new(256)?)
        .build()?;

    let first = session.respond_to("What problem does ownership solve?").await?;
    let second = session.respond_to("Show a minimal example.").await?;

    println!("{first}\n\n{second}");
    Ok(())
}
```

The builder also keeps Rig-style aliases for users coming from agent-builder
APIs:

```rust
use aimx::{AppleIntelligenceModels, Error, Temperature};

fn build_with_rig_style_names() -> Result<aimx::LanguageModelSession, Error> {
    AppleIntelligenceModels::default()
        .agent()
        .preamble("You are a careful code reviewer.")
        .temperature(Temperature::new(0.1)?)
        .build()
}
```

Prefer the Apple-style names in new docs and application code:

| Preferred | Compatibility alias |
|---|---|
| `session()` | `agent()` |
| `instructions(...)` | `preamble(...)` |
| `respond_to(...)` | `respond(...)` / `complete(...)` |
| `stream_response(...)` | `stream(...)` |
| `respond_generating(...)` | `respond_as(...)` |

## Step 4: Keep Raw Inputs At Boundaries

AIMX avoids meaning-bearing primitives in reusable domain state. Instead of
storing raw `f64` or `usize` values in your application model, parse them into
typed options as soon as they enter your code.

```rust
use aimx::{Error, GenerationOptions, MaxTokens, Temperature};

fn fixed_options() -> Result<GenerationOptions, Error> {
    Ok(GenerationOptions::new()
        .temperature(Temperature::new(0.2)?)
        .max_tokens(MaxTokens::new(512)?))
}
```

When values come from a CLI flag, UI form, or JSON payload, use the explicitly
fallible boundary methods:

```rust
use aimx::{Error, GenerationOptions};

fn options_from_user_input(temperature: f64, max_tokens: usize) -> Result<GenerationOptions, Error> {
    GenerationOptions::new()
        .try_temperature(temperature)?
        .try_max_tokens(max_tokens)
}
```

Invalid values become typed errors:

| Bad input | Error |
|---|---|
| `NaN` temperature | `Error::InvalidTemperature` |
| temperature outside the allowed range | `Error::InvalidTemperature` |
| too many tokens for the bridge | `Error::InvalidMaxTokens` |
| prompt or instructions containing `\0` | `Error::NullByte` |

## Step 5: Stream Response Text

Streaming returns a `ResponseStream`. Each item is a typed `ResponseText`, so
you can print it directly or call `as_str()` / `into_string()`.

```rust
use aimx::{Error, LanguageModelSession};
use futures_util::StreamExt as _;

async fn stream_story() -> Result<(), Error> {
    let session = LanguageModelSession::with_instructions(
        "Write vivid but concise responses.",
    )?;

    let mut stream = session.stream_response("Write a three-sentence story.")?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        print!("{}", chunk.as_str());
    }

    println!();
    Ok(())
}
```

You can also use MLX-style names when you want model-inference terminology:

```rust
use aimx::{AppleIntelligenceModels, Error};

fn start_generation_stream() -> Result<aimx::ResponseStream, Error> {
    AppleIntelligenceModels::default()
        .session()
        .build()?
        .stream_generate("List three reasons Rust is useful for FFI.")
}
```

## Step 6: Generate Structured Output

Use a `GenerationSchema` when you want JSON that deserializes into your own Rust
type. Field names in the schema must match the fields in the Rust type.

```rust
use aimx::{
    Error, GenerationSchema, GenerationSchemaProperty, GenerationSchemaPropertyType,
    LanguageModelSession,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ReleaseNote {
    title: String,
    summary: String,
    risk: String,
}

fn release_note_schema() -> GenerationSchema {
    GenerationSchema::new("ReleaseNote")
        .description("A short engineering release note")
        .property(
            GenerationSchemaProperty::new("title", GenerationSchemaPropertyType::String)
                .description("A concise title"),
        )
        .property(
            GenerationSchemaProperty::new("summary", GenerationSchemaPropertyType::String)
                .description("What changed"),
        )
        .property(
            GenerationSchemaProperty::new("risk", GenerationSchemaPropertyType::String)
                .description("Main release risk or fallback"),
        )
}

async fn write_release_note() -> Result<ReleaseNote, Error> {
    let session = LanguageModelSession::new()?;
    session
        .respond_generating(
            "Summarize a refactor that renamed a crate to AIMX.",
            &release_note_schema(),
        )
        .await
}
```

Schema properties are typed. The public schema model uses
`GenerationSchemaPropertyRequirement` instead of a raw boolean for optionality.

```rust
use aimx::{
    GenerationSchemaProperty, GenerationSchemaPropertyRequirement, GenerationSchemaPropertyType,
};

let property = GenerationSchemaProperty::new("details", GenerationSchemaPropertyType::String)
    .required(GenerationSchemaPropertyRequirement::Optional);
```

## Step 7: Register A Rust Tool

Tools let the model ask your Rust code for data while forming a response. The
safe mental model is to treat a tool like a small API endpoint: it has a name, a
description, a schema for its arguments, and a handler that can fail normally.

The handler returns `ToolOutput` on success or `ToolCallError` on failure. That
keeps tool failures inside the model/tool protocol instead of panicking.

```rust
use aimx::{
    AppleIntelligenceModels, Error, GenerationSchema, GenerationSchemaProperty,
    GenerationSchemaPropertyType, ToolCallError, ToolDefinition, ToolOutput,
};
use serde_json::Value;

fn weather_tool() -> ToolDefinition {
    let parameters = GenerationSchema::new("WeatherArgs")
        .description("Arguments for a weather lookup")
        .property(
            GenerationSchemaProperty::new("city", GenerationSchemaPropertyType::String)
                .description("City name"),
        );

    ToolDefinition::builder(
        "get_weather",
        "Return current weather for a city",
        parameters,
    )
    .handler(|args: Value| {
        let city = args
            .get("city")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolCallError::new("missing string field: city"))?;

        Ok(ToolOutput::from(format!("{city}: 22 C, sunny")))
    })
}

async fn answer_with_tool() -> Result<(), Error> {
    let session = AppleIntelligenceModels::default()
        .session()
        .instructions("Use tools when factual lookup is needed.")
        .tool(weather_tool())
        .build()?;

    let answer = session.respond_to("What is the weather in Tokyo?").await?;
    println!("{answer}");

    Ok(())
}
```

Tool handlers are user code. AIMX catches handler panics before they can cross
the tool boundary, but production tools should still return explicit
`ToolCallError` values for bad arguments, missing local data, or disabled
features.

## Step 8: Explore Tested Agent Use Cases

The repository includes a deterministic agent gallery in
[`examples/agent_use_cases.rs`](examples/agent_use_cases.rs). It demonstrates a
variety of LLM-agent patterns without requiring live Apple Intelligence for the
unit tests.

Run the example:

```sh
cargo run --example agent_use_cases
```

Run its unit tests:

```sh
cargo test --example agent_use_cases
```

The example covers:

| Use case | Pattern | Tested contract |
|---|---|---|
| `research_brief` | plain text synthesis | prompt, instructions, and options validate |
| `support_triage` | structured routing | schema serializes `priority`, `team`, and `summary` |
| `code_review` | review assistant | prompt keeps typed-error and callback-risk semantics |
| `release_notes` | structured writing | schema marks `risk` optional |
| `meeting_actions` | extraction | schema serializes `owner`, `action`, and `due_date` |
| `weather_tool` | tool-augmented agent | tool returns `Tokyo: 22 C, sunny` and typed errors for bad args |

The tests assert expected deterministic results at the Rust boundary. They do
not assert live model text, because model output depends on local system state,
model availability, and prompt sampling. To run the live research-brief path
manually on supported hardware:

```sh
AIMX_RUN_LIVE_AGENT_EXAMPLES=1 cargo run --example agent_use_cases
```

Use this pattern for your own agents: test prompt construction, schemas, tool
handlers, and error mapping locally, then keep live generation as an explicit
integration test.

## Step 9: Handle Errors Smoothly

Most application entry points can return `Result<(), aimx::Error>` and use `?`.
At UX boundaries, match on errors so the caller sees the next useful action.

```rust
use aimx::{AvailabilityError, Error};

fn explain_error(error: Error) -> String {
    match error {
        Error::Unavailable(AvailabilityError::DeviceNotEligible) => {
            "This device does not support the local Apple Intelligence model.".to_string()
        }
        Error::Unavailable(AvailabilityError::NotEnabled) => {
            "Apple Intelligence is disabled in System Settings.".to_string()
        }
        Error::Unavailable(AvailabilityError::ModelNotReady) => {
            "The local model is not ready yet. Try again after setup finishes.".to_string()
        }
        Error::Unavailable(AvailabilityError::Unknown) => {
            "Apple Intelligence is unavailable for an unknown reason.".to_string()
        }
        Error::NullByte(_) => {
            "The prompt contains a NUL byte and cannot cross the C FFI boundary.".to_string()
        }
        Error::InvalidTemperature(value) => {
            format!("Temperature {value} is outside the supported range.")
        }
        Error::InvalidMaxTokens(value) => {
            format!("max_tokens value {value} is too large for the bridge.")
        }
        Error::Json(error) => format!("Structured generation failed JSON handling: {error}"),
        Error::Generation(error) => format!("Model generation failed: {error}"),
    }
}
```

Avoid `panic!`, `unwrap`, and `expect` in application paths that process user
input, model output, or tool arguments. AIMX exposes typed constructors and
typed errors so those cases can stay recoverable.

## Step 10: Put It Together

Here is a compact application that checks availability, builds a session, and
uses typed generation options.

```rust
use aimx::{
    AppleIntelligenceModels, AvailabilityError, Error, GenerationOptions, MaxTokens,
    Temperature,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let model = AppleIntelligenceModels::new();

    if let Err(reason) = model.availability() {
        print_unavailable(reason);
        return Ok(());
    }

    let options = GenerationOptions::new()
        .temperature(Temperature::new(0.2)?)
        .max_tokens(MaxTokens::new(256)?);

    let session = model
        .session()
        .instructions("You are a concise assistant for Rust engineers.")
        .options(options)
        .build()?;

    let answer = session
        .respond_to("Explain why typed FFI boundaries matter.")
        .await?;

    println!("{answer}");
    Ok(())
}

fn print_unavailable(reason: AvailabilityError) {
    match reason {
        AvailabilityError::DeviceNotEligible => {
            eprintln!("AIMX requires Apple Silicon for live local generation.")
        }
        AvailabilityError::NotEnabled => {
            eprintln!("Enable Apple Intelligence before running live generation.")
        }
        AvailabilityError::ModelNotReady => {
            eprintln!("Apple Intelligence is present but the model is not ready.")
        }
        AvailabilityError::Unknown => {
            eprintln!("Apple Intelligence is unavailable for an unknown reason.")
        }
    }
}
```

## Step 11: Test Code That Does Not Need Hardware

You can test validation and public API behavior on any host because AIMX returns
typed errors before it reaches the live model.

```rust
use aimx::{AppleIntelligenceModels, Error, MaxTokens, Prompt, Temperature};
use futures_executor::block_on;

#[test]
fn prompt_rejects_nul_byte() {
    assert!(matches!(
        Prompt::try_from("bad\0prompt"),
        Err(Error::NullByte(_))
    ));
}

#[test]
fn options_validate_ranges() -> Result<(), Error> {
    Temperature::new(0.3)?;
    MaxTokens::new(256)?;
    assert!(matches!(
        Temperature::new(f64::NAN),
        Err(Error::InvalidTemperature(_))
    ));
    Ok(())
}

#[test]
fn model_rejects_bad_prompt_before_availability() {
    let result = block_on(AppleIntelligenceModels::default().respond("bad\0prompt"));
    assert!(matches!(result, Err(Error::NullByte(_))));
}
```

Live model tests should be ignored by default and enabled explicitly on a
supported Mac:

```rust
#[test]
#[ignore = "requires Apple Intelligence"]
fn live_model_smoke_test() {
    let answer = futures_executor::block_on(aimx::respond("Say hello."));
    assert!(answer.is_ok());
}
```

Run normal tests everywhere:

```sh
cargo test
```

Run live tests only on supported hardware:

```sh
cargo test -- --include-ignored
```

## Step 12: Troubleshooting

### `Error::Unavailable(DeviceNotEligible)`

The crate compiled, but the current host is not eligible for live local
generation. This is expected on non-macOS machines, older macOS versions, Intel
Macs, or CI runners without Apple Intelligence.

### `Error::Unavailable(NotEnabled)`

Apple Intelligence is available but disabled. Enable it in System Settings.

### `Error::Unavailable(ModelNotReady)`

The system reports that the model is not ready. Wait for the system model setup
or download to finish, then retry.

### Swift Bridge Did Not Compile

`build.rs` attempts to compile `bridge.swift` with `xcrun swiftc`. If Xcode or
the macOS 26 SDK is missing, the build continues without `aimx_bridge`.
Application code still compiles, and model APIs return `Error::Unavailable`.

### Structured Output Fails To Deserialize

Check that every field in your Rust type has the same name and compatible type
as the property in your `GenerationSchema`. Start with a small schema and add
fields one at a time.

### Tool Handler Returns An Error

Treat tool errors as normal model workflow. Validate model-supplied JSON with
`Value::get`, `Value::as_*`, and typed error messages. Return
`ToolCallError::new(...)` instead of panicking.

## Step 13: Production Checklist

Before publishing or shipping an AIMX integration:

1. Keep user input conversion at typed boundaries (`Prompt`, `SystemInstructions`,
   `Temperature`, `MaxTokens`, `GenerationSchema`).
2. Match on `Error::Unavailable` and provide a local fallback path.
3. Keep live Apple Intelligence tests ignored by default.
4. Run `cargo fmt --check`.
5. Run `cargo test`.
6. Run `cargo test --examples`.
7. Run `cargo clippy --all-targets --all-features -- -D warnings`.
8. Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
9. Run `cargo publish --dry-run` before publishing a crate release.

## Next Steps

After this tutorial, read:

- [README.md](README.md) for the concise API overview.
- [references/documentation-style.md](references/documentation-style.md) for the
  style expected in repository docs and public rustdoc.
- [references/async-architecture.md](references/async-architecture.md) for the
  callback, cancellation, stream, and tool-handler boundaries.
- [references/policy.md](references/policy.md) for the primitive-boundary policy.
- [CONTRIBUTING.md](CONTRIBUTING.md) before opening changes against AIMX.
