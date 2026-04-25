# Documentation Style

This repository uses a Rust-first teaching style: precise like official Rust
documentation, but paced like a short machine-learning lesson. The goal is to
help a reader build the right mental model before they copy an example.

This is a style guide, not an authorship claim.

## Principles

1. Start with the smallest correct example.
2. Explain the one idea the example teaches.
3. Name the invariant that makes the example safe.
4. Show the recoverable error path.
5. Link to the deeper reference only after the reader has a working model.

## Voice

Use direct, calm sentences.

Prefer:

```text
Check availability before sending a prompt. Unsupported hosts return
Error::Unavailable, so applications can show a fallback instead of panicking.
```

Avoid:

```text
This amazing crate seamlessly unlocks local AI with a powerful and flexible
developer experience.
```

## Structure

Public docs should follow this order when it fits the topic:

1. What the item is.
2. When to use it.
3. A minimal example.
4. `# Errors` for fallible APIs.
5. `# Panics` when a public API can panic.
6. `# Safety` when unsafe preconditions exist.

Repository markdown should follow this order:

1. Problem or learning objective.
2. Mental model.
3. Example.
4. Error handling.
5. Tests or verification.
6. Links to deeper references.

## AIMX Terms

Use these names in new docs:

| Meaning | Preferred term |
|---|---|
| Project | AIMX |
| Cargo package | `apple-intelligence-models` |
| Rust import | `aimx` |
| Model handle | `AppleIntelligenceModels` |
| Session | `LanguageModelSession` |
| Structured output | `GenerationSchema` |
| One-shot response | `respond` |
| Session response | `respond_to` |
| Streaming response | `stream_response` |

Compatibility aliases may be documented when explaining migration, but examples
should teach the preferred names first.

## Examples

Examples should compile when possible. Use `no_run` for snippets that need live
Apple Intelligence hardware.

Good examples:

- import only the items they use
- return `Result`
- use `?` for recoverable failures
- match on `Error::Unavailable` at application boundaries
- avoid `unwrap`, `expect`, and `panic!`
- keep raw primitives at input boundaries

## Error Sections

Every public function that returns `Result` should document what can fail. The
section should tell the reader what action to take, not only name the variant.

Example:

```text
# Errors

Returns Error::NullByte when the prompt contains an interior NUL byte. Reject
or escape the input before sending it across the C FFI boundary.
```

## Teaching Pattern

Use this pattern for tutorials:

1. "Here is the goal."
2. "Here is the smallest code."
3. "Here is why this type exists."
4. "Here is what can fail."
5. "Here is how to test it without live hardware."

This keeps the documentation useful for both Rust users who want an API
reference and learners who need the intuition behind the API.
