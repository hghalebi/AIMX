# Primitive Boundary Policy

AIMX (`apple-intelligence-models`, imported as `aimx`) keeps meaning-bearing
values out of domain and public state by using explicit value types.

## Domain Types

- Prompt and instruction text cross the C FFI boundary through `Prompt` and
  `SystemInstructions`, which reject interior null bytes at construction.
- Generation controls use `Temperature` and `MaxTokens`. `GenerationOptions`
  stores those typed values, not raw numbers.
- Structured-generation optionality uses
  `GenerationSchemaPropertyRequirement`, not a boolean flag.
- Tool output and tool-call failures use `ToolOutput` and `ToolCallError`
  instead of `String` results.

## Boundary Exceptions

Raw primitives are allowed only at edges where they are immediately parsed,
validated, or exposed:

- `TryFrom<&str>` and `FromStr` implementations for prompt/instruction values.
- `Temperature::new(f64)` and `MaxTokens::new(usize)` constructors.
- `GenerationOptions::try_temperature(f64)` and
  `GenerationOptions::try_max_tokens(usize)` for UI, CLI, env, or JSON input.
- `LanguageModelSessionBuilder::try_temperature(f64)` and
  `LanguageModelSessionBuilder::try_max_tokens(usize)` for builder input at
  application boundaries.
- `as_str`, `as_f64`, `get`, and `into_string` accessors when sending values to
  the Swift bridge, rendering, or serializing output.
- `serde_json::Value` at the tool-call boundary because arguments arrive as
  model-provided JSON and are owned by tool handlers.

## Regression Check

Run:

```sh
./scripts/find_raw_primitives.sh
```

Every hit must be either a boundary constructor/accessor or a candidate for a
new value type or enum.
