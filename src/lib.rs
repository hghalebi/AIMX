//! AIMX: safe Rust bindings for Apple's [FoundationModels] on-device language
//! model framework, also known as Apple Intelligence.
//!
//! `aimx` is a small, safe Rust API over the system
//! `FoundationModels.framework`. The model runs locally on supported Apple
//! hardware, so prompts and responses do not require API keys, network requests,
//! or a hosted inference provider.
//!
//! # API overview
//!
//! The crate is organized around a few public concepts:
//!
//! - [`AppleIntelligenceModels`] starts session builders with [`AppleIntelligenceModels::session`].
//! - [`LanguageModelSession`] owns a stateful `LanguageModelSession` transcript.
//! - [`GenerationOptions`] configures per-request temperature and token limits.
//! - [`ResponseStream`] implements [`futures_core::Stream`] for incremental text.
//! - [`GenerationSchema`] and [`GenerationSchemaProperty`] mirror Apple's guided-generation
//!   schema vocabulary for structured JSON responses.
//! - [`ToolDefinition`] registers Rust callbacks the model can call during a
//!   response.
//! - [`Prompt`], [`SystemInstructions`], [`Temperature`], and [`MaxTokens`] make
//!   important FFI and generation boundaries explicit.
//!
//! Top-level helpers such as [`respond`] are available for one-off prompts, but
//! production code should usually build a [`LanguageModelSession`] so instructions, tools, and
//! defaults are visible in one place.
//!
//! # Platform requirements
//!
//! | Requirement | Value |
//! |---|---|
//! | macOS | 26 (Tahoe) or later |
//! | Hardware | Apple Silicon (M1 or later) |
//! | System setting | Apple Intelligence enabled |
//! | Build tool | Xcode with the macOS 26 SDK |
//!
//! The crate still compiles on unsupported hosts. When the Swift bridge cannot
//! be built, or when the current machine cannot run Apple Intelligence, public
//! model APIs return [`Error::Unavailable`] instead of panicking or failing to
//! link.
//!
//! # Quick start
//!
//! ```no_run
//! # async fn example() -> Result<(), aimx::Error> {
//! use aimx::{is_available, respond};
//!
//! if !is_available() {
//!     eprintln!("Apple Intelligence not available on this device");
//!     return Ok(());
//! }
//!
//! let answer = respond("What is the capital of France?").await?;
//! println!("{answer}");
//! # Ok(()) }
//! ```
//!
//! # Builder-style sessions
//!
//! ```no_run
//! # async fn example() -> Result<(), aimx::Error> {
//! use aimx::{MaxTokens, AppleIntelligenceModels, Temperature};
//!
//! let session = AppleIntelligenceModels::default()
//!     .session()
//!     .instructions("You are a concise Rust expert.")
//!     .temperature(Temperature::new(0.2)?)
//!     .max_tokens(MaxTokens::new(256)?)
//!     .build()?;
//! let r1 = session.respond_to("What is ownership?").await?;
//! let r2 = session.respond_to("Give me a one-line example.").await?;
//! println!("{r1}\n{r2}");
//! # Ok(()) }
//! ```
//!
//! # Generation options
//!
//! Use [`GenerationOptions::new`] with [`Temperature`] and [`MaxTokens`] to keep
//! generation defaults type-safe after input has crossed your application boundary.
//!
//! ```
//! use aimx::{GenerationOptions, MaxTokens, Temperature};
//!
//! let precise = GenerationOptions::new()
//!     .temperature(Temperature::new(0.2)?)
//!     .max_tokens(MaxTokens::new(256)?);
//! # Ok::<(), aimx::Error>(())
//! ```
//!
//! # Streaming
//!
//! ```no_run
//! # async fn example() -> Result<(), aimx::Error> {
//! use aimx::LanguageModelSession;
//!
//! let session = LanguageModelSession::new()?;
//! let stream = session.stream_response("Tell me a short story.")?;
//! # Ok(()) }
//! ```
//!
//! # Structured generation
//!
//! ```no_run
//! # async fn example() -> Result<(), aimx::Error> {
//! use serde::Deserialize;
//! use aimx::{LanguageModelSession, GenerationSchema, GenerationSchemaProperty, GenerationSchemaPropertyType};
//!
//! #[derive(Deserialize)]
//! struct CityInfo { name: String, population: f64, country: String }
//!
//! let session = LanguageModelSession::new()?;
//! let schema = GenerationSchema::new("CityInfo")
//!     .property(GenerationSchemaProperty::new("name", GenerationSchemaPropertyType::String))
//!     .property(GenerationSchemaProperty::new("population", GenerationSchemaPropertyType::Double))
//!     .property(GenerationSchemaProperty::new("country", GenerationSchemaPropertyType::String));
//!
//! let info: CityInfo = session.respond_generating("Describe Paris.", &schema).await?;
//! println!("{} has {} people", info.name, info.population);
//! # Ok(()) }
//! ```
//!
//! # Tool calling
//!
//! ```no_run
//! # async fn example() -> Result<(), aimx::Error> {
//! use aimx::{
//!     AppleIntelligenceModels, GenerationSchema, GenerationSchemaProperty, GenerationSchemaPropertyType, ToolDefinition, ToolOutput,
//! };
//!
//! let tool = ToolDefinition::builder(
//!     "get_weather",
//!     "Get current weather for a city",
//!     GenerationSchema::new("GetWeatherArgs")
//!         .property(GenerationSchemaProperty::new("city", GenerationSchemaPropertyType::String)
//!             .description("City name")),
//! )
//! .handler(|args| {
//!     let city = args["city"].as_str().unwrap_or("unknown");
//!     Ok(ToolOutput::from(format!("Weather in {city}: sunny, 72°F")))
//! });
//!
//! let session = AppleIntelligenceModels::default()
//!     .session()
//!     .instructions("You are a weather assistant.")
//!     .tool(tool)
//!     .build()?;
//! let response = session.respond_to("What's the weather in Tokyo?").await?;
//! println!("{response}");
//! # Ok(()) }
//! ```
//!
//! # Errors
//!
//! All fallible APIs use [`Error`]. The most common variants are:
//!
//! - [`Error::Unavailable`] when Apple Intelligence cannot run on this machine.
//! - [`Error::NullByte`] when prompt or instruction text cannot cross the C FFI
//!   boundary.
//! - [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when generation
//!   options are outside the bridge-supported range.
//! - [`Error::Generation`] for model or bridge failures during generation.
//! - [`Error::Json`] for schema serialization or structured-response decoding.
//!
//! # Panics
//!
//! Public APIs in this crate are designed not to panic for user-provided input.
//! Validation errors are reported through [`Error`]. Panics inside user-provided
//! tool handlers are caught and returned as [`ToolCallError`].
//!
//! # Safety
//!
//! This is a safe Rust wrapper. The private FFI layer owns all `unsafe` calls to
//! Swift-exported C functions, validates string inputs before crossing the
//! boundary, and stores opaque Swift handles behind an owned `SessionHandle`.
//! Callers do not need to uphold any unsafe preconditions.
//!
//! # Documentation style
//!
//! The public docs intentionally follow the shape recommended by the
//! [rustdoc book] and the [Rust API Guidelines]: crate-level overview, tested
//! examples where possible, intra-doc links, and explicit error/panic/safety
//! sections for fallible or boundary-sensitive APIs.
//!
//! [FoundationModels]: https://developer.apple.com/documentation/foundationmodels
//! [rustdoc book]: https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
//! [Rust API Guidelines]: https://rust-lang.github.io/api-guidelines/documentation.html

#![warn(
    missing_docs,
    rustdoc::bare_urls,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_codeblock_attributes
)]
#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unwrap_used
    )
)]

use std::convert::Infallible;
use std::ffi::{CStr, CString, NulError};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as StdContext, Poll};

use futures_channel::mpsc;
use futures_core::Stream;

#[cfg(aimx_bridge)]
use std::ffi::{c_char, c_void};

#[cfg(aimx_bridge)]
use std::ptr::null;

#[cfg(aimx_bridge)]
use std::ptr::NonNull;

#[cfg(aimx_bridge)]
use std::sync::Arc;

#[cfg(aimx_bridge)]
use futures_channel::oneshot;

// ─── FFI declarations ──────────────────────────────────────────────────────────

#[cfg(aimx_bridge)]
unsafe extern "C" {
    fn fm_availability_reason() -> i32;
    fn fm_session_create(instructions: *const c_char) -> *mut c_void;
    fn fm_session_create_with_tools(
        instructions: *const c_char,
        tools_json: *const c_char,
        tool_ctx: *mut c_void,
        tool_dispatch: extern "C" fn(
            *mut c_void,
            *const c_char,
            *const c_char,
            *mut c_void,
            extern "C" fn(*mut c_void, *const c_char, *const c_char),
        ),
    ) -> *mut c_void;
    fn fm_session_destroy(handle: *mut c_void);
    fn fm_session_respond(
        handle: *mut c_void,
        prompt: *const c_char,
        temperature: f64,
        max_tokens: i64,
        ctx: *mut c_void,
        callback: extern "C" fn(*mut c_void, *const c_char, *const c_char),
    );
    fn fm_session_respond_structured(
        handle: *mut c_void,
        prompt: *const c_char,
        schema_json: *const c_char,
        temperature: f64,
        max_tokens: i64,
        ctx: *mut c_void,
        callback: extern "C" fn(*mut c_void, *const c_char, *const c_char),
    );
    fn fm_session_stream(
        handle: *mut c_void,
        prompt: *const c_char,
        temperature: f64,
        max_tokens: i64,
        ctx: *mut c_void,
        on_token: extern "C" fn(*mut c_void, *const c_char),
        on_done: extern "C" fn(*mut c_void, *const c_char),
    );
}

// ─── Cross-target trait bounds ────────────────────────────────────────────────

/// `Send` on native targets and a no-op marker on WebAssembly.
#[cfg(not(target_family = "wasm"))]
pub trait WasmCompatSend: Send {}

/// No-op marker on WebAssembly where single-threaded runtimes do not require `Send`.
#[cfg(target_family = "wasm")]
pub trait WasmCompatSend {}

#[cfg(not(target_family = "wasm"))]
impl<T> WasmCompatSend for T where T: Send {}

#[cfg(target_family = "wasm")]
impl<T> WasmCompatSend for T {}

/// `Sync` on native targets and a no-op marker on WebAssembly.
#[cfg(not(target_family = "wasm"))]
pub trait WasmCompatSync: Sync {}

/// No-op marker on WebAssembly where single-threaded runtimes do not require `Sync`.
#[cfg(target_family = "wasm")]
pub trait WasmCompatSync {}

#[cfg(not(target_family = "wasm"))]
impl<T> WasmCompatSync for T where T: Sync {}

#[cfg(target_family = "wasm")]
impl<T> WasmCompatSync for T {}

macro_rules! string_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates a new value.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrows the inner string.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes the wrapper and returns the inner string.
            pub fn into_string(self) -> String {
                self.0
            }

            /// Returns `true` when the wrapped string is empty.
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl PartialEq<$name> for &str {
            fn eq(&self, other: &$name) -> bool {
                *self == other.as_str()
            }
        }
    };
}

string_newtype!(
    /// Developer-provided system instructions for a session.
    InstructionsText
);
string_newtype!(
    /// UTF-8 prompt text sent to the model.
    PromptText
);
string_newtype!(
    /// Text returned by the model.
    ResponseText
);
/// MLX-style alias for text generated by the model.
pub type GeneratedText = ResponseText;

string_newtype!(
    /// Name of a structured-generation schema.
    GenerationSchemaName
);
/// Compatibility alias for the earlier structured-generation schema-name type.
pub type ResponseSchemaName = GenerationSchemaName;
/// Compatibility alias for the older structured-generation schema-name type.
pub type SchemaName = GenerationSchemaName;

string_newtype!(
    /// Name of a property in a structured-generation schema.
    GenerationSchemaPropertyName
);
/// Compatibility alias for the earlier structured-generation field-name type.
pub type ResponseFieldName = GenerationSchemaPropertyName;
/// Compatibility alias for the older structured-generation field-name type.
pub type SchemaPropertyName = GenerationSchemaPropertyName;

string_newtype!(
    /// Human-readable description attached to a schema or schema property.
    SchemaDescription
);
string_newtype!(
    /// Name the model uses to invoke a tool.
    ToolName
);
string_newtype!(
    /// Human-readable description of a tool.
    ToolDescription
);
string_newtype!(
    /// Successful output returned from a tool call.
    ToolOutput
);

/// Prompt text that has been checked for C-FFI compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    text: String,
    c_text: CString,
}

impl Prompt {
    /// Creates a prompt, rejecting text containing interior null bytes.
    ///
    /// Use this when you want to validate user-provided prompt text before
    /// constructing a [`LanguageModelSession`] or calling [`respond`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aimx::Prompt;
    ///
    /// let prompt = Prompt::new("Summarize this note")?;
    /// assert_eq!(prompt.as_str(), "Summarize this note");
    /// # Ok::<(), aimx::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] if `value` contains an interior null byte.
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let text = value.into();
        let c_text = CString::new(text.clone())?;

        Ok(Self { text, c_text })
    }

    /// Borrows the prompt as UTF-8 text.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[cfg(aimx_bridge)]
    fn as_ptr(&self) -> *const c_char {
        self.c_text.as_ptr()
    }
}

impl TryFrom<&str> for Prompt {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Prompt {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<PromptText> for Prompt {
    type Error = Error;

    fn try_from(value: PromptText) -> Result<Self, Self::Error> {
        Self::new(value.into_string())
    }
}

impl AsRef<str> for Prompt {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Compatibility alias for the older prompt boundary name.
pub type PromptInput = Prompt;

/// LanguageModelSession instructions that have been checked for C-FFI compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInstructions {
    text: String,
    c_text: CString,
}

impl SystemInstructions {
    /// Creates session instructions, rejecting text containing interior null bytes.
    ///
    /// SystemInstructions are developer-controlled system guidance that persists for
    /// the lifetime of a [`LanguageModelSession`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aimx::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new("Answer in one concise paragraph.")?;
    /// assert_eq!(instructions.as_str(), "Answer in one concise paragraph.");
    /// # Ok::<(), aimx::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] if `value` contains an interior null byte.
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let text = value.into();
        let c_text = CString::new(text.clone())?;

        Ok(Self { text, c_text })
    }

    /// Empty system instructions.
    ///
    /// This is equivalent to `SystemInstructions::new("")` without the fallible
    /// allocation path.
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            c_text: CString::default(),
        }
    }

    /// Borrows the instructions as UTF-8 text.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[cfg(aimx_bridge)]
    fn as_ptr(&self) -> *const c_char {
        self.c_text.as_ptr()
    }
}

impl Default for SystemInstructions {
    fn default() -> Self {
        Self::empty()
    }
}

impl TryFrom<&str> for SystemInstructions {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for SystemInstructions {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<InstructionsText> for SystemInstructions {
    type Error = Error;

    fn try_from(value: InstructionsText) -> Result<Self, Self::Error> {
        Self::new(value.into_string())
    }
}

/// Compatibility alias for the older system-instructions boundary name.
pub type Instructions = SystemInstructions;

/// Valid model temperature in the inclusive range `0.0..=2.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature(f64);

impl Temperature {
    /// Lowest supported temperature.
    pub const MIN: f64 = 0.0;
    /// Highest supported temperature.
    pub const MAX: f64 = 2.0;

    /// Creates a validated temperature.
    ///
    /// Apple Intelligence accepts temperatures in the inclusive range
    /// [`Temperature::MIN`] through [`Temperature::MAX`]. Lower values make
    /// output more deterministic; higher values make output more varied.
    ///
    /// # Examples
    ///
    /// ```
    /// use aimx::Temperature;
    ///
    /// let temperature = Temperature::new(0.2)?;
    /// assert_eq!(temperature.as_f64(), 0.2);
    /// # Ok::<(), aimx::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTemperature`] when `value` is outside
    /// `0.0..=2.0` or is `NaN`.
    pub fn new(value: f64) -> Result<Self, Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidTemperature(value))
        }
    }

    /// Returns the raw floating-point temperature.
    pub fn as_f64(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Temperature {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Maximum number of response tokens requested from the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaxTokens(usize);

impl MaxTokens {
    /// Highest token limit representable by the Swift bridge.
    pub const MAX: usize = i64::MAX as usize;

    /// Creates a token limit.
    ///
    /// The Rust API stores token counts as [`usize`], but the Swift bridge uses
    /// `i64`. This constructor rejects values that cannot cross that boundary
    /// without changing meaning.
    ///
    /// # Examples
    ///
    /// ```
    /// use aimx::MaxTokens;
    ///
    /// let max_tokens = MaxTokens::new(256)?;
    /// assert_eq!(max_tokens.get(), 256);
    /// # Ok::<(), aimx::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidMaxTokens`] when `value` is greater than
    /// [`MaxTokens::MAX`].
    pub fn new(value: usize) -> Result<Self, Error> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(Error::InvalidMaxTokens(value))
        }
    }

    /// Returns the raw token count.
    pub fn get(self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for MaxTokens {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Error returned by the model or bridge during generation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct GenerationError {
    message: String,
}

impl GenerationError {
    /// Creates a generation error with a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Borrows the error message.
    pub fn as_str(&self) -> &str {
        &self.message
    }
}

impl From<String> for GenerationError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for GenerationError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

/// Error returned by a Rust tool handler.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct ToolCallError {
    message: String,
}

impl ToolCallError {
    /// Creates a tool-call error with a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Borrows the error message.
    pub fn as_str(&self) -> &str {
        &self.message
    }
}

impl From<String> for ToolCallError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for ToolCallError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

/// Result type returned by tool handlers.
pub type ToolResult = Result<ToolOutput, ToolCallError>;

type ModelTextResult = Result<ResponseText, GenerationError>;
type StreamSender = mpsc::UnboundedSender<ModelTextResult>;
type StreamReceiver = mpsc::UnboundedReceiver<ModelTextResult>;
type ToolHandlerBox = Box<dyn ToolHandler>;

#[cfg(aimx_bridge)]
type ResponseSender = oneshot::Sender<ModelTextResult>;
#[cfg(aimx_bridge)]
type ResponseReceiver = oneshot::Receiver<ModelTextResult>;
#[cfg(aimx_bridge)]
type ToolResultCallback = extern "C" fn(*mut c_void, *const c_char, *const c_char);

// ─── Error ─────────────────────────────────────────────────────────────────────

/// Reasons why Apple Intelligence is not available on the current device.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AvailabilityError {
    /// The device does not have compatible hardware (requires Apple Silicon M1 or later).
    #[error("device is not eligible (requires Apple Silicon M1 or later)")]
    DeviceNotEligible,
    /// Apple Intelligence is supported but has not been enabled in System Settings.
    #[error("Apple Intelligence is not enabled in System Settings")]
    NotEnabled,
    /// The on-device model is still downloading or is otherwise not ready.
    #[error("the on-device model is not ready yet")]
    ModelNotReady,
    /// An unrecognized availability state was returned by the framework.
    #[error("unknown availability state")]
    Unknown,
}

/// Compatibility alias for the older availability error name.
pub type UnavailabilityReason = AvailabilityError;

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Apple Intelligence is not available on this device.
    #[error("Apple Intelligence unavailable: {0}")]
    Unavailable(#[source] AvailabilityError),

    /// The model produced an error during text generation.
    #[error("generation error: {0}")]
    Generation(#[from] GenerationError),

    /// An argument contained a null byte and could not be converted to a C string.
    #[error("argument contains a null byte: {0}")]
    NullByte(#[from] NulError),

    /// A `temperature` value outside the valid range [0.0, 2.0] was supplied.
    #[error("temperature {0} is out of range; expected 0.0 – 2.0")]
    InvalidTemperature(f64),

    /// A `max_tokens` value too large for the AIMX bridge was supplied.
    #[error("max_tokens {0} is out of range; expected no more than i64::MAX")]
    InvalidMaxTokens(usize),

    /// JSON serialisation or deserialisation failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A tool invoked by the model returned an error.
    #[error("tool '{name}' failed: {error}")]
    ToolError {
        /// Tool name.
        name: ToolName,
        /// Tool failure.
        #[source]
        error: ToolCallError,
    },
}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

// ─── GenerationOptions ─────────────────────────────────────────────────────────

/// Tuning parameters for a single generation request.
///
/// Values are optional; `None` uses the model's built-in default. Numeric
/// settings are stored as [`Temperature`] and [`MaxTokens`] so validated
/// generation semantics cannot be bypassed after construction.
#[derive(Debug, Default, Clone)]
pub struct GenerationOptions {
    temperature: Option<Temperature>,
    max_tokens: Option<MaxTokens>,
}

impl GenerationOptions {
    /// Creates options using model defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the generation temperature.
    ///
    /// Range: `0.0` (fully deterministic) to `2.0` (very creative).
    pub fn temperature(mut self, temperature: Temperature) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Alias for [`GenerationOptions::temperature`].
    pub fn with_temperature(mut self, temperature: Temperature) -> Self {
        self = self.temperature(temperature);
        self
    }

    /// Parses and sets a generation temperature from a raw boundary value.
    ///
    /// Prefer [`GenerationOptions::temperature`] when your code already has a
    /// [`Temperature`]. Use this at IO boundaries such as CLI, JSON, or UI input.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTemperature`] when `temperature` is outside
    /// Apple Intelligence's supported range.
    pub fn try_temperature(self, temperature: f64) -> Result<Self, Error> {
        Ok(self.temperature(Temperature::new(temperature)?))
    }

    /// Sets the maximum number of response tokens.
    ///
    /// The model's session has a combined context window of 4 096 tokens
    /// (instructions + all prompts + all responses). Leaving this unset lets the
    /// model decide.
    pub fn max_tokens(mut self, max_tokens: MaxTokens) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Alias for [`GenerationOptions::max_tokens`].
    pub fn with_max_tokens(mut self, max_tokens: MaxTokens) -> Self {
        self = self.max_tokens(max_tokens);
        self
    }

    /// Parses and sets a maximum token count from a raw boundary value.
    ///
    /// Prefer [`GenerationOptions::max_tokens`] when your code already has a
    /// [`MaxTokens`]. Use this at IO boundaries such as CLI, JSON, or UI input.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidMaxTokens`] when the value cannot be represented
    /// by the Swift bridge.
    pub fn try_max_tokens(self, max_tokens: usize) -> Result<Self, Error> {
        Ok(self.max_tokens(MaxTokens::new(max_tokens)?))
    }

    /// Returns the configured typed temperature, if any.
    pub fn temperature_value(&self) -> Option<Temperature> {
        self.temperature
    }

    /// Returns the configured typed maximum response token count, if any.
    pub fn max_tokens_value(&self) -> Option<MaxTokens> {
        self.max_tokens
    }

    /// Validates all configured option values.
    ///
    /// Values constructed through this type are already validated. This method
    /// is kept so generic setup code can verify options before storing them.
    ///
    /// # Examples
    ///
    /// ```
    /// use aimx::{GenerationOptions, MaxTokens, Temperature};
    ///
    /// let options = GenerationOptions::new()
    ///     .temperature(Temperature::new(0.4)?)
    ///     .max_tokens(MaxTokens::new(128)?);
    /// options.validate()?;
    /// # Ok::<(), aimx::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This method returns errors only if options were constructed through a
    /// future boundary path that can carry invalid data.
    pub fn validate(&self) -> Result<(), Error> {
        GenerationConfig::try_from(self).map(|_| ())
    }

    fn validated(&self) -> Result<GenerationConfig, Error> {
        GenerationConfig::try_from(self)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct GenerationConfig {
    temperature: Option<Temperature>,
    max_tokens: Option<MaxTokens>,
}

impl GenerationConfig {
    fn ffi_temperature(self) -> f64 {
        self.temperature.map(Temperature::as_f64).unwrap_or(-1.0)
    }

    fn ffi_max_tokens(self) -> i64 {
        self.max_tokens
            .map(|max_tokens| max_tokens.get() as i64)
            .unwrap_or(-1)
    }
}

impl TryFrom<&GenerationOptions> for GenerationConfig {
    type Error = Error;

    fn try_from(options: &GenerationOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            temperature: options.temperature,
            max_tokens: options.max_tokens,
        })
    }
}

// ─── GenerationSchema types for structured generation ────────────────────────────────────

/// The type of a single property in a [`GenerationSchema`].
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationSchemaPropertyType {
    /// UTF-8 text.
    String,
    /// Whole number (serialised as JSON integer).
    Integer,
    /// Floating-point number.
    Double,
    /// Boolean true/false.
    Bool,
}

/// Compatibility alias for the older structured-generation property type name.
pub type ResponseFieldType = GenerationSchemaPropertyType;
/// Compatibility alias for the oldest structured-generation property type name.
pub type SchemaPropertyType = GenerationSchemaPropertyType;

/// Whether a [`GenerationSchemaProperty`] must appear in structured output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GenerationSchemaPropertyRequirement {
    /// The model must include this property.
    #[default]
    Required,
    /// The model may omit this property.
    Optional,
}

impl GenerationSchemaPropertyRequirement {
    /// Returns `true` when the property may be omitted.
    pub fn is_optional(self) -> bool {
        matches!(self, Self::Optional)
    }

    /// Returns `true` when the property must be present.
    pub fn is_required(self) -> bool {
        matches!(self, Self::Required)
    }
}

/// A single property within a [`GenerationSchema`].
#[derive(Debug, Clone)]
pub struct GenerationSchemaProperty {
    /// Property name (matches the JSON key in the model output).
    pub name: GenerationSchemaPropertyName,
    /// Optional human-readable hint that guides the model.
    pub description: Option<SchemaDescription>,
    /// The expected type of this property.
    pub property_type: GenerationSchemaPropertyType,
    /// Whether the model must include or may omit this property.
    pub requirement: GenerationSchemaPropertyRequirement,
}

/// Compatibility alias for the older structured-generation property name.
pub type ResponseField = GenerationSchemaProperty;
/// Compatibility alias for the oldest structured-generation property name.
pub type SchemaProperty = GenerationSchemaProperty;

impl GenerationSchemaProperty {
    /// Creates a required property with the given name and type.
    pub fn new(
        name: impl Into<GenerationSchemaPropertyName>,
        property_type: GenerationSchemaPropertyType,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            property_type,
            requirement: GenerationSchemaPropertyRequirement::Required,
        }
    }

    /// Attaches a human-readable description that guides the model.
    pub fn description(mut self, description: impl Into<SchemaDescription>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Marks this property as optional (the model may omit it).
    pub fn optional(mut self) -> Self {
        self.requirement = GenerationSchemaPropertyRequirement::Optional;
        self
    }

    /// Marks this property as required.
    pub fn required(mut self) -> Self {
        self.requirement = GenerationSchemaPropertyRequirement::Required;
        self
    }
}

impl serde::Serialize for GenerationSchemaProperty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let field_count = if self.description.is_some() { 4 } else { 3 };
        let mut state = serializer.serialize_struct("GenerationSchemaProperty", field_count)?;
        state.serialize_field("name", &self.name)?;
        if let Some(description) = &self.description {
            state.serialize_field("description", description)?;
        }
        state.serialize_field("type", &self.property_type)?;
        state.serialize_field("optional", &self.requirement.is_optional())?;
        state.end()
    }
}

/// Describes the JSON object shape that the model must produce for structured generation.
///
/// Build one using the builder methods, then pass it to [`LanguageModelSession::generate_object`].
///
/// ```
/// use aimx::{GenerationSchema, GenerationSchemaProperty, GenerationSchemaPropertyType};
///
/// let schema = GenerationSchema::new("Point")
///     .property(GenerationSchemaProperty::new("x", GenerationSchemaPropertyType::Double))
///     .property(GenerationSchemaProperty::new("y", GenerationSchemaPropertyType::Double));
/// ```
#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerationSchema {
    /// Internal type name used by the model's structured generation system.
    pub name: GenerationSchemaName,
    /// Optional description of what this type represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<SchemaDescription>,
    /// The properties the model must populate.
    pub properties: Vec<GenerationSchemaProperty>,
}

/// Compatibility alias for the older structured-generation schema name.
pub type ResponseSchema = GenerationSchema;
/// Compatibility alias for the oldest structured-generation schema name.
pub type Schema = GenerationSchema;

impl GenerationSchema {
    /// Creates a new empty schema with the given type name.
    pub fn new(name: impl Into<GenerationSchemaName>) -> Self {
        Self {
            name: name.into(),
            description: None,
            properties: Vec::new(),
        }
    }

    /// Attaches a description of this type.
    pub fn description(mut self, description: impl Into<SchemaDescription>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a property to this schema.
    pub fn property(mut self, property: GenerationSchemaProperty) -> Self {
        self.properties.push(property);
        self
    }
}

// ─── Tool calling ──────────────────────────────────────────────────────────────

/// A function that the model can invoke when responding to a prompt.
///
/// The handler receives the model's arguments as a [`serde_json::Value`] and must return
/// either a [`ToolOutput`] delivered back to the model or a [`ToolCallError`].
///
/// Build one with [`ToolDefinition::builder`], then attach it to a [`LanguageModelSessionBuilder`].
pub struct ToolDefinition {
    /// Name the model uses to reference this tool. Must be unique within a session.
    pub name: ToolName,
    /// Human-readable description shown to the model.
    pub description: ToolDescription,
    /// GenerationSchema describing the arguments the model must supply when calling this tool.
    pub parameters: GenerationSchema,
    handler: ToolHandlerBox,
}

impl ToolDefinition {
    /// Creates a complete tool definition from a typed handler.
    pub fn new(
        name: impl Into<ToolName>,
        description: impl Into<ToolDescription>,
        parameters: GenerationSchema,
        handler: impl Fn(serde_json::Value) -> ToolResult + WasmCompatSend + WasmCompatSync + 'static,
    ) -> Self {
        Self::builder(name, description, parameters).handler(handler)
    }

    /// Starts building a tool definition.
    pub fn builder(
        name: impl Into<ToolName>,
        description: impl Into<ToolDescription>,
        parameters: GenerationSchema,
    ) -> ToolDefinitionBuilder {
        ToolDefinitionBuilder {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }

    /// Alias for [`ToolDefinition::new`] that reads well at call sites.
    pub fn from_handler(
        name: impl Into<ToolName>,
        description: impl Into<ToolDescription>,
        parameters: GenerationSchema,
        handler: impl Fn(serde_json::Value) -> ToolResult + WasmCompatSend + WasmCompatSync + 'static,
    ) -> Self {
        Self::new(name, description, parameters, handler)
    }

    #[cfg(aimx_bridge)]
    fn bridge_description(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.as_str(),
            "description": self.description.as_str(),
            "properties": &self.parameters.properties,
        })
    }
}

/// Builder for [`ToolDefinition`].
#[derive(Debug, Clone)]
pub struct ToolDefinitionBuilder {
    name: ToolName,
    description: ToolDescription,
    parameters: GenerationSchema,
}

impl ToolDefinitionBuilder {
    /// Attaches the Rust handler and returns a complete tool definition.
    pub fn handler(
        self,
        handler: impl Fn(serde_json::Value) -> ToolResult + WasmCompatSend + WasmCompatSync + 'static,
    ) -> ToolDefinition {
        ToolDefinition {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
            handler: Box::new(handler),
        }
    }
}

/// Trait boundary implemented by callable Rust tools.
pub trait Tool: std::fmt::Debug + WasmCompatSend + WasmCompatSync {
    /// Returns the tool name visible to the model.
    fn name(&self) -> &ToolName;

    /// Returns the human-readable tool description visible to the model.
    fn description(&self) -> &ToolDescription;

    /// Returns the JSON argument schema visible to the model.
    fn parameters(&self) -> &GenerationSchema;

    /// Executes the tool with model-supplied arguments.
    ///
    /// # Errors
    ///
    /// Returns [`ToolCallError`] when the handler cannot satisfy the model's
    /// request. The error text is forwarded back through the bridge as the tool
    /// result error.
    fn call(&self, args: serde_json::Value) -> ToolResult;
}

impl Tool for ToolDefinition {
    fn name(&self) -> &ToolName {
        &self.name
    }

    fn description(&self) -> &ToolDescription {
        &self.description
    }

    fn parameters(&self) -> &GenerationSchema {
        &self.parameters
    }

    fn call(&self, args: serde_json::Value) -> ToolResult {
        call_tool_handler(self.handler.as_ref(), args)
    }
}

trait ToolHandler: WasmCompatSend + WasmCompatSync {
    fn call(&self, args: serde_json::Value) -> ToolResult;
}

impl<F> ToolHandler for F
where
    F: Fn(serde_json::Value) -> ToolResult + WasmCompatSend + WasmCompatSync,
{
    fn call(&self, args: serde_json::Value) -> ToolResult {
        self(args)
    }
}

fn call_tool_handler(handler: &dyn ToolHandler, args: serde_json::Value) -> ToolResult {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler.call(args))) {
        Ok(result) => result,
        Err(payload) => Err(ToolCallError::new(format!(
            "tool handler panicked: {}",
            panic_payload_message(payload.as_ref())
        ))),
    }
}

fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_owned();
    }

    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "non-string panic payload".to_owned()
}

impl std::fmt::Debug for ToolDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolDefinition")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

/// Internal context that holds tool handlers. A raw pointer to this is passed to Swift
/// as `tool_ctx` and lives for the full `LanguageModelSession` lifetime via `Arc`.
#[cfg(aimx_bridge)]
struct ToolsContext {
    tools: Vec<(ToolName, ToolHandlerBox)>,
}

#[cfg(aimx_bridge)]
impl ToolsContext {
    fn from_definitions(tools: Vec<ToolDefinition>) -> Arc<Self> {
        Arc::new(Self {
            tools: tools
                .into_iter()
                .map(|tool| (tool.name, tool.handler))
                .collect(),
        })
    }

    fn call(&self, name: &str, args: serde_json::Value) -> ToolResult {
        let handler = self
            .tools
            .iter()
            .find_map(|(tool_name, handler)| (tool_name.as_str() == name).then_some(handler));

        match handler {
            Some(handler) => call_tool_handler(handler.as_ref(), args),
            None => Err(ToolCallError::new(format!("unknown tool: {name}"))),
        }
    }
}

// ─── Availability ──────────────────────────────────────────────────────────────

const FM_AVAILABLE: i32 = 0;
const FM_DEVICE_NOT_ELIGIBLE: i32 = 1;
const FM_NOT_ENABLED: i32 = 2;
const FM_MODEL_NOT_READY: i32 = 3;

/// Returns `true` if Apple Intelligence is available and ready on this device.
///
/// This is a cheap synchronous check. See [`availability`] for the specific reason
/// when this returns `false`.
pub fn is_available() -> bool {
    availability().is_ok()
}

/// Returns `Ok(())` if Apple Intelligence is ready.
///
/// # Errors
///
/// Returns [`AvailabilityError`] describing why the local model cannot be
/// used on the current machine.
pub fn availability() -> Result<(), AvailabilityError> {
    #[cfg(aimx_bridge)]
    {
        let code = unsafe { fm_availability_reason() };
        match code {
            FM_AVAILABLE => Ok(()),
            FM_DEVICE_NOT_ELIGIBLE => Err(AvailabilityError::DeviceNotEligible),
            FM_NOT_ENABLED => Err(AvailabilityError::NotEnabled),
            FM_MODEL_NOT_READY => Err(AvailabilityError::ModelNotReady),
            _ => Err(AvailabilityError::Unknown),
        }
    }
    #[cfg(not(aimx_bridge))]
    Err(AvailabilityError::DeviceNotEligible)
}

// ─── Apple/MLX-style model handle, builders, and trait boundary ──────────────

/// Handle for Apple's default on-device system language model.
#[derive(Debug, Default, Clone, Copy)]
pub struct AppleIntelligenceModels {
    _private: (),
}

impl AppleIntelligenceModels {
    /// Creates a handle to the default system language model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `Ok(())` when Apple Intelligence is ready for this model.
    ///
    /// This mirrors Swift's `SystemLanguageModel.default.availability` shape,
    /// but maps unavailable states into [`AvailabilityError`].
    ///
    /// # Errors
    ///
    /// Returns [`AvailabilityError`] describing why the local model cannot be
    /// used on the current machine.
    pub fn availability(&self) -> Result<(), AvailabilityError> {
        availability()
    }

    /// Returns `true` when the default system language model is available.
    pub fn is_available(&self) -> bool {
        self.availability().is_ok()
    }

    /// Starts building a stateful session.
    pub fn session(&self) -> LanguageModelSessionBuilder {
        LanguageModelSessionBuilder::new()
    }

    /// Alias for [`AppleIntelligenceModels::session`] for users coming from Rig's agent builders.
    pub fn agent(&self) -> LanguageModelSessionBuilder {
        self.session()
    }

    /// Sends a single prompt in a fresh session and returns plain response text.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] for prompt text that cannot cross the C FFI
    /// boundary, [`Error::Unavailable`] when Apple Intelligence is unavailable,
    /// or [`Error::Generation`] when the model or bridge fails.
    pub async fn respond<P>(&self, prompt: P) -> Result<String, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        Ok(self.generate_text(prompt).await?.into_string())
    }

    /// MLX-style alias for [`AppleIntelligenceModels::generate_text`].
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::generate_text`].
    pub async fn generate<P>(&self, prompt: P) -> Result<GeneratedText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.generate_text(prompt).await
    }

    /// MLX-style alias for [`AppleIntelligenceModels::generate_text_with_options`].
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::generate_text_with_options`].
    pub async fn generate_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<GeneratedText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.generate_text_with_options(prompt, options).await
    }

    /// Sends a single prompt in a fresh session and returns typed response text.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::respond`].
    pub async fn complete<P>(&self, prompt: P) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.generate_text(prompt).await
    }

    /// Generates typed response text in a fresh session.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::respond`].
    pub async fn generate_text<P>(&self, prompt: P) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let options = GenerationOptions::default();
        self.generate_text_with_options(prompt, &options).await
    }

    /// Generates typed response text in a fresh session with explicit options.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] for invalid prompt text,
    /// [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] for invalid
    /// options, [`Error::Unavailable`] when Apple Intelligence is unavailable,
    /// or [`Error::Generation`] when the model or bridge fails.
    pub async fn generate_text_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        LanguageModel::generate_text_with_options(self, prompt, options.clone()).await
    }

    /// Streams response text from a fresh session.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::generate_text`].
    pub fn stream_text<P>(&self, prompt: P) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let options = GenerationOptions::default();
        self.stream_text_with_options(prompt, &options)
    }

    /// Streams response text from a fresh session with explicit options.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::generate_text_with_options`].
    pub fn stream_text_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        LanguageModel::stream_text_with_options(self, prompt, options.clone())
    }

    /// MLX-style alias for [`AppleIntelligenceModels::stream_text`].
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::stream_text`].
    pub fn stream_generate<P>(&self, prompt: P) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_text(prompt)
    }

    /// MLX-style alias for [`AppleIntelligenceModels::stream_text_with_options`].
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`AppleIntelligenceModels::stream_text_with_options`].
    pub fn stream_generate_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_text_with_options(prompt, options)
    }
}

/// Compatibility alias for Apple's framework-facing system-language-model name.
pub type SystemLanguageModel = AppleIntelligenceModels;
/// Compatibility alias for the earlier provider handle name.
pub type FoundationModels = AppleIntelligenceModels;
/// Compatibility alias for the older provider handle name.
pub type Client = AppleIntelligenceModels;

impl LanguageModel for AppleIntelligenceModels {
    fn generate_text_with_options<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> impl Future<Output = Result<ResponseText, Error>> + '_
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into);
        let builder = self.session().options(options.clone());

        async move {
            let prompt = prompt?;
            let session = builder.build()?;
            session.generate_prompt_with_options(prompt, &options).await
        }
    }

    fn stream_text_with_options<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into)?;
        let session = self.session().options(options.clone()).build()?;
        session.stream_text_with_options(prompt, &options)
    }
}

/// Provider-agnostic language-model boundary used by sessions and provider handles.
pub trait LanguageModel {
    /// Generates response text for a prompt with explicit generation options.
    ///
    /// # Errors
    ///
    /// Implementations return [`Error`] when prompt conversion, option
    /// validation, session creation, or model generation fails.
    fn generate_text_with_options<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> impl Future<Output = Result<ResponseText, Error>> + '_
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>;

    /// Streams response text for a prompt with explicit generation options.
    ///
    /// # Errors
    ///
    /// Implementations return [`Error`] when prompt conversion, option
    /// validation, session creation, or stream startup fails.
    fn stream_text_with_options<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>;
}

/// Compatibility trait for older completion-oriented naming.
pub trait CompletionModel: LanguageModel {
    /// Generates response text for a prompt with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when prompt conversion, option validation, session
    /// creation, or model generation fails.
    fn completion<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> impl Future<Output = Result<ResponseText, Error>> + '_
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.generate_text_with_options(prompt, options)
    }

    /// Streams response text for a prompt with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when prompt conversion, option validation, session
    /// creation, or stream startup fails.
    fn stream_completion<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_text_with_options(prompt, options)
    }
}

impl<T> CompletionModel for T where T: LanguageModel {}

/// Convenience trait for sending prompts with default generation options.
pub trait GenerateText: LanguageModel {
    /// Sends a prompt with default generation options.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when prompt conversion or the underlying generation
    /// request fails.
    fn prompt<P>(&self, prompt: P) -> impl Future<Output = Result<ResponseText, Error>> + '_
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into);

        async move {
            let prompt = prompt?;
            self.generate_text_with_options(prompt, GenerationOptions::default())
                .await
        }
    }
}

impl<T> GenerateText for T where T: LanguageModel {}

/// Builder for [`LanguageModelSession`].
#[derive(Debug)]
pub struct LanguageModelSessionBuilder {
    instructions: InstructionsText,
    tools: Vec<ToolDefinition>,
    default_options: GenerationOptions,
}

impl LanguageModelSessionBuilder {
    /// Creates an empty session builder.
    pub fn new() -> Self {
        Self {
            instructions: InstructionsText::new(""),
            tools: Vec::new(),
            default_options: GenerationOptions::default(),
        }
    }

    /// Sets persistent system instructions for the session.
    pub fn instructions(mut self, instructions: impl Into<InstructionsText>) -> Self {
        self.instructions = instructions.into();
        self
    }

    /// Alias for [`LanguageModelSessionBuilder::instructions`] using Rig terminology.
    pub fn preamble(self, instructions: impl Into<InstructionsText>) -> Self {
        self.instructions(instructions)
    }

    /// Adds one callable tool.
    pub fn tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    /// Adds multiple callable tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = ToolDefinition>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Sets the typed default temperature used by [`LanguageModelSession::respond_to`] and [`LanguageModelSession::stream_response`].
    pub fn temperature(mut self, temperature: Temperature) -> Self {
        self.default_options = self.default_options.temperature(temperature);
        self
    }

    /// Alias for [`LanguageModelSessionBuilder::temperature`].
    pub fn with_temperature(mut self, temperature: Temperature) -> Self {
        self = self.temperature(temperature);
        self
    }

    /// Parses and sets the default temperature from a raw boundary value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTemperature`] when `temperature` is outside
    /// Apple Intelligence's supported range.
    pub fn try_temperature(mut self, temperature: f64) -> Result<Self, Error> {
        self.default_options = self.default_options.try_temperature(temperature)?;
        Ok(self)
    }

    /// Sets the default maximum response tokens.
    pub fn max_tokens(mut self, max_tokens: MaxTokens) -> Self {
        self.default_options = self.default_options.max_tokens(max_tokens);
        self
    }

    /// Alias for [`LanguageModelSessionBuilder::max_tokens`].
    pub fn with_max_tokens(mut self, max_tokens: MaxTokens) -> Self {
        self = self.max_tokens(max_tokens);
        self
    }

    /// Parses and sets the default maximum response token count from a raw boundary value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidMaxTokens`] when `max_tokens` cannot be represented
    /// by the Swift bridge.
    pub fn try_max_tokens(mut self, max_tokens: usize) -> Result<Self, Error> {
        self.default_options = self.default_options.try_max_tokens(max_tokens)?;
        Ok(self)
    }

    /// Replaces all default generation options.
    pub fn options(mut self, options: GenerationOptions) -> Self {
        self.default_options = options;
        self
    }

    /// Builds a stateful model session.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn example() -> Result<(), aimx::Error> {
    /// use aimx::{AppleIntelligenceModels, Temperature};
    ///
    /// let session = AppleIntelligenceModels::default()
    ///     .session()
    ///     .instructions("Answer in short paragraphs.")
    ///     .temperature(Temperature::new(0.2)?)
    ///     .build()?;
    /// # let _ = session;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] for invalid instructions,
    /// [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] for invalid
    /// defaults, [`Error::Json`] if tool metadata cannot be serialized, or
    /// [`Error::Unavailable`] when Apple Intelligence is not ready.
    pub fn build(self) -> Result<LanguageModelSession, Error> {
        let instructions = SystemInstructions::try_from(self.instructions)?;
        LanguageModelSession::create(instructions, self.tools, self.default_options)
    }
}

impl Default for LanguageModelSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compatibility alias for the older session-builder name.
pub type SessionBuilder = LanguageModelSessionBuilder;

// ─── Convenience top-level functions ──────────────────────────────────────────

/// Sends a single prompt to the model and returns the response text.
///
/// Each call creates a fresh session with no prior context. For multi-turn
/// conversations use [`LanguageModelSession`] directly.
///
/// # Errors
///
/// Returns [`Error::NullByte`] for prompt text that cannot cross the C FFI
/// boundary, [`Error::Unavailable`] when Apple Intelligence is unavailable, or
/// [`Error::Generation`] when the model or bridge fails.
pub async fn respond<P>(prompt: P) -> Result<String, Error>
where
    P: TryInto<Prompt>,
    P::Error: Into<Error>,
{
    AppleIntelligenceModels::default().respond(prompt).await
}

/// Like [`respond`] but allows tuning generation via [`GenerationOptions`].
///
/// # Errors
///
/// Returns every error documented by [`respond`]. It can also return
/// [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when `options`
/// contains out-of-range values.
pub async fn respond_with_options<P>(
    prompt: P,
    options: &GenerationOptions,
) -> Result<String, Error>
where
    P: TryInto<Prompt>,
    P::Error: Into<Error>,
{
    Ok(AppleIntelligenceModels::default()
        .generate_with_options(prompt, options)
        .await?
        .into_string())
}

/// MLX-style top-level text generation helper.
///
/// Each call creates a fresh session with no prior context. For multi-turn
/// conversations use [`LanguageModelSession`] directly.
///
/// # Errors
///
/// Returns every error documented by [`respond`].
pub async fn generate<P>(prompt: P) -> Result<String, Error>
where
    P: TryInto<Prompt>,
    P::Error: Into<Error>,
{
    respond(prompt).await
}

/// MLX-style top-level generation helper with explicit generation options.
///
/// # Errors
///
/// Returns every error documented by [`respond_with_options`].
pub async fn generate_with_options<P>(
    prompt: P,
    options: &GenerationOptions,
) -> Result<String, Error>
where
    P: TryInto<Prompt>,
    P::Error: Into<Error>,
{
    respond_with_options(prompt, options).await
}

/// MLX-style top-level streaming helper.
///
/// # Errors
///
/// Returns [`Error::NullByte`] for invalid prompt text,
/// [`Error::Unavailable`] when Apple Intelligence is unavailable, or
/// [`Error::Generation`] if stream startup fails.
pub fn stream_generate<P>(prompt: P) -> Result<ResponseStream, Error>
where
    P: TryInto<Prompt>,
    P::Error: Into<Error>,
{
    AppleIntelligenceModels::default().stream_generate(prompt)
}

/// MLX-style top-level streaming helper with explicit generation options.
///
/// # Errors
///
/// Returns every error documented by [`stream_generate`]. It can also return
/// [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when `options`
/// contains out-of-range values.
pub fn stream_generate_with_options<P>(
    prompt: P,
    options: &GenerationOptions,
) -> Result<ResponseStream, Error>
where
    P: TryInto<Prompt>,
    P::Error: Into<Error>,
{
    AppleIntelligenceModels::default().stream_generate_with_options(prompt, options)
}

// ─── LanguageModelSession ───────────────────────────────────────────────────────────────────

/// Owned opaque pointer to Swift's ARC-retained session holder.
#[cfg(aimx_bridge)]
#[derive(Debug)]
struct SessionHandle(NonNull<c_void>);

#[cfg(aimx_bridge)]
impl SessionHandle {
    fn from_raw(handle: *mut c_void) -> Result<Self, Error> {
        NonNull::new(handle)
            .map(Self)
            .ok_or(Error::Unavailable(AvailabilityError::Unknown))
    }

    fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

#[cfg(aimx_bridge)]
impl Drop for SessionHandle {
    fn drop(&mut self) {
        unsafe {
            fm_session_destroy(self.as_ptr());
        }
    }
}

#[cfg(aimx_bridge)]
unsafe impl Send for SessionHandle {}

#[cfg(aimx_bridge)]
unsafe impl Sync for SessionHandle {}

/// A stateful conversation session backed by a `LanguageModelSession`.
///
/// The session automatically maintains a conversation transcript, so each
/// successive call to [`respond_to`][LanguageModelSession::respond_to] has access to the full
/// prior context (subject to the 4 096-token context window limit).
///
/// # Thread safety
///
/// `LanguageModelSession` is `Send + Sync`. Concurrent calls are forwarded to the underlying
/// Swift session, which handles them via its internal async actor. Note however
/// that concurrent calls will interleave entries in the transcript in an
/// unspecified order; for predictable multi-turn behaviour call sequentially.
///
/// # Drop behaviour
///
/// Dropping a `LanguageModelSession` releases the caller's session reference.
/// In-flight generation futures and active [`ResponseStream`]s hold their own
/// cloned handle reference until Swift invokes the completion callback, so
/// cancellation cannot release the Swift session while the bridge still needs it.
pub struct LanguageModelSession {
    default_options: GenerationOptions,
    #[cfg(aimx_bridge)]
    handle: Arc<SessionHandle>,
    /// Keeps the tool handlers alive for the full session lifetime.
    /// A raw pointer to the Arc payload is passed to Swift as `tool_ctx`.
    #[cfg(aimx_bridge)]
    _tools: Option<Arc<ToolsContext>>,
}

impl std::fmt::Debug for LanguageModelSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageModelSession")
            .field("default_options", &self.default_options)
            .finish_non_exhaustive()
    }
}

impl LanguageModelSession {
    /// Starts building a session.
    pub fn builder() -> LanguageModelSessionBuilder {
        LanguageModelSessionBuilder::new()
    }

    /// Creates a new session with no system instructions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unavailable`] when Apple Intelligence is unavailable.
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    /// Creates a new session with the given system instructions.
    ///
    /// SystemInstructions act as a persistent system prompt that guides all subsequent
    /// responses in this session. They must come from developer code, never from
    /// user input, to prevent prompt-injection attacks.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] if the instructions cannot cross the C FFI
    /// boundary, or [`Error::Unavailable`] when Apple Intelligence is
    /// unavailable.
    pub fn with_instructions<I>(instructions: I) -> Result<Self, Error>
    where
        I: TryInto<SystemInstructions>,
        I::Error: Into<Error>,
    {
        let instructions = instructions.try_into().map_err(Into::into)?;
        Self::create(instructions, Vec::new(), GenerationOptions::default())
    }

    /// Creates a session pre-loaded with the given tools.
    ///
    /// The model will use these tools automatically when appropriate during `respond` calls.
    /// Tool names must be unique within the session.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] if the instructions or serialized tool
    /// metadata cannot cross the C FFI boundary, [`Error::Json`] if tool
    /// metadata serialization fails, or [`Error::Unavailable`] when Apple
    /// Intelligence is unavailable.
    pub fn with_tools<I>(instructions: I, tools: Vec<ToolDefinition>) -> Result<Self, Error>
    where
        I: TryInto<SystemInstructions>,
        I::Error: Into<Error>,
    {
        let instructions = instructions.try_into().map_err(Into::into)?;
        Self::create(instructions, tools, GenerationOptions::default())
    }

    fn create(
        instructions: SystemInstructions,
        tools: Vec<ToolDefinition>,
        default_options: GenerationOptions,
    ) -> Result<Self, Error> {
        default_options.validate()?;
        availability().map_err(Error::Unavailable)?;

        #[cfg(aimx_bridge)]
        {
            Self::create_bridge_session(instructions, tools, default_options)
        }
        #[cfg(not(aimx_bridge))]
        {
            let _ = (instructions, tools, default_options);
            Err(Error::Unavailable(AvailabilityError::DeviceNotEligible))
        }
    }

    #[cfg(aimx_bridge)]
    fn create_bridge_session(
        instructions: SystemInstructions,
        tools: Vec<ToolDefinition>,
        default_options: GenerationOptions,
    ) -> Result<Self, Error> {
        if tools.is_empty() {
            return Self::create_plain_bridge_session(instructions, default_options);
        }

        Self::create_tool_bridge_session(instructions, tools, default_options)
    }

    #[cfg(aimx_bridge)]
    fn create_plain_bridge_session(
        instructions: SystemInstructions,
        default_options: GenerationOptions,
    ) -> Result<Self, Error> {
        let handle = unsafe { fm_session_create(instructions.as_ptr()) };

        Ok(Self {
            default_options,
            handle: Arc::new(SessionHandle::from_raw(handle)?),
            _tools: None,
        })
    }

    #[cfg(aimx_bridge)]
    fn create_tool_bridge_session(
        instructions: SystemInstructions,
        tools: Vec<ToolDefinition>,
        default_options: GenerationOptions,
    ) -> Result<Self, Error> {
        let tool_descriptions = tools
            .iter()
            .map(ToolDefinition::bridge_description)
            .collect::<Vec<_>>();
        let c_tools_json = CString::new(serde_json::to_vec(&tool_descriptions)?)?;
        let tools_ctx = ToolsContext::from_definitions(tools);
        let tool_ctx_ptr = Arc::as_ptr(&tools_ctx) as *mut c_void;

        let handle = unsafe {
            fm_session_create_with_tools(
                instructions.as_ptr(),
                c_tools_json.as_ptr(),
                tool_ctx_ptr,
                tool_dispatch,
            )
        };

        Ok(Self {
            default_options,
            handle: Arc::new(SessionHandle::from_raw(handle)?),
            _tools: Some(tools_ctx),
        })
    }

    /// Sends a prompt and returns the full response text.
    ///
    /// The response is appended to this session's transcript, so subsequent
    /// calls have access to prior context.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] for invalid prompt text, or
    /// [`Error::Generation`] when the model or bridge fails.
    pub async fn respond<P>(&self, prompt: P) -> Result<String, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        Ok(self.respond_to(prompt).await?.into_string())
    }

    /// Apple-style typed response method matching Swift's `respond(to:)`.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond`].
    pub async fn respond_to<P>(&self, prompt: P) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to_with_options(prompt, &self.default_options)
            .await
    }

    /// Sends a prompt and returns typed response text.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_to`].
    pub async fn complete<P>(&self, prompt: P) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to(prompt).await
    }

    /// MLX-style typed response alias.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_to`].
    pub async fn generate<P>(&self, prompt: P) -> Result<GeneratedText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to(prompt).await
    }

    /// Generates typed response text and appends the exchange to this session's transcript.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_to`].
    pub async fn generate_text<P>(&self, prompt: P) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to(prompt).await
    }

    /// Like [`respond`][LanguageModelSession::respond] but allows tuning generation.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::respond`]. It can also
    /// return [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when
    /// `options` contains out-of-range values.
    pub async fn respond_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<String, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        Ok(self
            .respond_to_with_options(prompt, options)
            .await?
            .into_string())
    }

    /// Apple-style typed response method with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::respond`]. It can also
    /// return [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when
    /// `options` contains out-of-range values.
    pub async fn respond_to_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into)?;
        self.generate_prompt_with_options(prompt, options).await
    }

    /// Like [`complete`][LanguageModelSession::complete] but allows tuning generation.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_with_options`].
    pub async fn complete_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to_with_options(prompt, options).await
    }

    /// MLX-style typed response alias with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_to_with_options`].
    pub async fn generate_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<GeneratedText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to_with_options(prompt, options).await
    }

    /// Generates typed response text with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_with_options`].
    pub async fn generate_text_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseText, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_to_with_options(prompt, options).await
    }

    async fn generate_prompt_with_options(
        &self,
        prompt: Prompt,
        options: &GenerationOptions,
    ) -> Result<ResponseText, Error> {
        let config = options.validated()?;
        #[cfg(aimx_bridge)]
        {
            let handle = Arc::clone(&self.handle);
            let (tx, rx) = oneshot::channel::<ModelTextResult>();
            let ctx = Box::into_raw(Box::new(ResponseContext {
                tx,
                _handle: handle,
            })) as *mut c_void;

            unsafe {
                fm_session_respond(
                    self.handle.as_ptr(),
                    prompt.as_ptr(),
                    config.ffi_temperature(),
                    config.ffi_max_tokens(),
                    ctx,
                    respond_callback,
                );
            }

            receive_response(rx).await
        }
        #[cfg(not(aimx_bridge))]
        {
            let _ = (prompt, config);
            Err(Error::Unavailable(AvailabilityError::DeviceNotEligible))
        }
    }

    /// Sends a prompt and deserialises the response into `T` using the provided schema.
    ///
    /// The model generates output conforming to `schema` and this method deserialises it.
    /// Derive [`serde::Deserialize`] on `T` and ensure the field names match the schema
    /// property names exactly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] for invalid prompt text, [`Error::Json`] if
    /// the schema or model response cannot be serialized or deserialized, or
    /// [`Error::Generation`] when the model or bridge fails.
    pub async fn respond_as<T, P>(&self, prompt: P, schema: &GenerationSchema) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_generating(prompt, schema).await
    }

    /// Apple-style guided-generation method for a dynamic [`GenerationSchema`].
    ///
    /// This mirrors Swift's `respond(to:generating:)` terminology while
    /// deserializing the generated JSON into `T`.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::respond_as`].
    pub async fn respond_generating<T, P>(
        &self,
        prompt: P,
        schema: &GenerationSchema,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_generating_with_options(prompt, schema, &self.default_options)
            .await
    }

    /// Generates structured output and deserialises it into `T`.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_as`].
    pub async fn generate_object<T, P>(
        &self,
        prompt: P,
        schema: &GenerationSchema,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_generating(prompt, schema).await
    }

    /// Like [`respond_as`][LanguageModelSession::respond_as] but allows tuning generation.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::respond_as`]. It can also
    /// return [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when
    /// `options` contains out-of-range values.
    pub async fn respond_as_with_options<T, P>(
        &self,
        prompt: P,
        schema: &GenerationSchema,
        options: &GenerationOptions,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_generating_with_options(prompt, schema, options)
            .await
    }

    /// Apple-style guided-generation method with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::respond_as_with_options`].
    pub async fn respond_generating_with_options<T, P>(
        &self,
        prompt: P,
        schema: &GenerationSchema,
        options: &GenerationOptions,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into)?;
        let config = options.validated()?;

        self.respond_generating_prompt_with_config(prompt, schema, config)
            .await
    }

    /// Generates structured output with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::respond_as_with_options`].
    pub async fn generate_object_with_options<T, P>(
        &self,
        prompt: P,
        schema: &GenerationSchema,
        options: &GenerationOptions,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.respond_generating_with_options(prompt, schema, options)
            .await
    }

    async fn respond_generating_prompt_with_config<T>(
        &self,
        prompt: Prompt,
        schema: &GenerationSchema,
        config: GenerationConfig,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        #[cfg(aimx_bridge)]
        {
            let handle = Arc::clone(&self.handle);
            let (tx, rx) = oneshot::channel::<ModelTextResult>();
            let ctx = Box::into_raw(Box::new(ResponseContext {
                tx,
                _handle: handle,
            })) as *mut c_void;
            let c_schema_json = CString::new(serde_json::to_vec(schema)?)?;

            unsafe {
                fm_session_respond_structured(
                    self.handle.as_ptr(),
                    prompt.as_ptr(),
                    c_schema_json.as_ptr(),
                    config.ffi_temperature(),
                    config.ffi_max_tokens(),
                    ctx,
                    respond_callback,
                );
            }

            let json = receive_response(rx).await?.into_string();
            serde_json::from_str(&json).map_err(Error::from)
        }
        #[cfg(not(aimx_bridge))]
        {
            let _ = (prompt, schema, config);
            Err(Error::Unavailable(AvailabilityError::DeviceNotEligible))
        }
    }

    /// Returns a [`ResponseStream`] that yields text chunks as the model generates them.
    ///
    /// Each yielded chunk is an incremental snapshot of the response text. Drive the
    /// stream with your preferred async executor.
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), aimx::Error> {
    /// use aimx::LanguageModelSession;
    ///
    /// let session = LanguageModelSession::new()?;
    /// let stream = session.stream_response("Count to ten.")?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::NullByte`] for invalid prompt text, or
    /// [`Error::Generation`] if stream startup fails. Individual stream items
    /// can also yield [`Error::Generation`] after the stream has been created.
    pub fn stream<P>(&self, prompt: P) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response(prompt)
    }

    /// Apple-style streaming method matching Swift's `streamResponse(to:)`.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::stream`].
    pub fn stream_response<P>(&self, prompt: P) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response_with_options(prompt, &self.default_options)
    }

    /// MLX-style streaming alias matching `stream_generate`.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::stream_response`].
    pub fn stream_generate<P>(&self, prompt: P) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response(prompt)
    }

    /// Streams response text with this session's default generation options.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::stream_response`].
    pub fn stream_text<P>(&self, prompt: P) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response(prompt)
    }

    /// Like [`stream`][LanguageModelSession::stream] but allows tuning generation.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::stream`]. It can also return
    /// [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when
    /// `options` contains out-of-range values.
    pub fn stream_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response_with_options(prompt, options)
    }

    /// Apple-style streaming method with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::stream`]. It can also return
    /// [`Error::InvalidTemperature`] or [`Error::InvalidMaxTokens`] when
    /// `options` contains out-of-range values.
    pub fn stream_response_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into)?;
        let config = options.validated()?;

        self.stream_prompt_with_config(prompt, config)
    }

    /// MLX-style streaming alias with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns every error documented by [`LanguageModelSession::stream_response_with_options`].
    pub fn stream_generate_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response_with_options(prompt, options)
    }

    /// Streams response text with explicit generation options.
    ///
    /// # Errors
    ///
    /// Returns the same error variants as [`LanguageModelSession::stream_with_options`].
    pub fn stream_text_with_options<P>(
        &self,
        prompt: P,
        options: &GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        self.stream_response_with_options(prompt, options)
    }

    fn stream_prompt_with_config(
        &self,
        prompt: Prompt,
        config: GenerationConfig,
    ) -> Result<ResponseStream, Error> {
        #[cfg(aimx_bridge)]
        {
            let handle = Arc::clone(&self.handle);
            let (tx, rx) = mpsc::unbounded::<ModelTextResult>();
            let ctx = Box::into_raw(Box::new(StreamContext {
                tx,
                _handle: handle,
            })) as *mut c_void;

            unsafe {
                fm_session_stream(
                    self.handle.as_ptr(),
                    prompt.as_ptr(),
                    config.ffi_temperature(),
                    config.ffi_max_tokens(),
                    ctx,
                    stream_token_callback,
                    stream_done_callback,
                );
            }

            Ok(ResponseStream { rx })
        }
        #[cfg(not(aimx_bridge))]
        {
            let _ = (prompt, config);
            Err(Error::Unavailable(AvailabilityError::DeviceNotEligible))
        }
    }
}

/// Compatibility alias for the older session type name.
pub type Session = LanguageModelSession;

impl LanguageModel for LanguageModelSession {
    fn generate_text_with_options<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> impl Future<Output = Result<ResponseText, Error>> + '_
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        let prompt = prompt.try_into().map_err(Into::into);

        async move {
            let prompt = prompt?;
            self.generate_prompt_with_options(prompt, &options).await
        }
    }

    fn stream_text_with_options<P>(
        &self,
        prompt: P,
        options: GenerationOptions,
    ) -> Result<ResponseStream, Error>
    where
        P: TryInto<Prompt>,
        P::Error: Into<Error>,
    {
        LanguageModelSession::stream_text_with_options(self, prompt, &options)
    }
}

// ─── ResponseStream ────────────────────────────────────────────────────────────

/// An async stream of text chunks produced by [`LanguageModelSession::stream`].
///
/// Each item is `Ok(ResponseText)` for a new chunk, or `Err(Error)` if generation failed.
/// The stream ends when the model finishes generating.
///
/// Implements [`futures_core::Stream`]; use with `.next()` from `StreamExt` or
/// any executor that can drive `Stream`.
pub struct ResponseStream {
    rx: StreamReceiver,
}

impl Stream for ResponseStream {
    type Item = Result<ResponseText, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx)
            .poll_next(cx)
            .map(|opt| opt.map(|r| r.map_err(Error::from)))
    }
}

#[cfg(aimx_bridge)]
async fn receive_response(receiver: ResponseReceiver) -> Result<ResponseText, Error> {
    receiver
        .await
        .map_err(|_| GenerationError::new("session was dropped before responding"))?
        .map_err(Error::from)
}

// ─── FFI callbacks ─────────────────────────────────────────────────────────────

/// Single-shot response context owned by Swift until `respond_callback`.
#[cfg(aimx_bridge)]
struct ResponseContext {
    tx: ResponseSender,
    _handle: Arc<SessionHandle>,
}

/// Callback for text and structured generation. Called exactly once by Swift.
#[cfg(aimx_bridge)]
extern "C" fn respond_callback(ctx: *mut c_void, result: *const c_char, error: *const c_char) {
    // Safety: ctx is always a Box<ResponseContext> allocated by a single-shot
    // generation call and consumed by this exactly-once callback.
    let context = unsafe { Box::from_raw(ctx as *mut ResponseContext) };

    if let Some(msg) = callback_owned_text(error) {
        context.tx.send(Err(GenerationError::from(msg))).ok();
    } else if let Some(text) = callback_owned_text(result) {
        context.tx.send(Ok(ResponseText::from(text))).ok();
    }
}

#[cfg(aimx_bridge)]
fn callback_owned_text(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
}

/// Internal state for a streaming request; owned by Swift via raw pointer until stream_done_callback.
#[cfg(aimx_bridge)]
struct StreamContext {
    tx: StreamSender,
    _handle: Arc<SessionHandle>,
}

/// Token callback for streaming. May be called many times before stream_done_callback.
#[cfg(aimx_bridge)]
extern "C" fn stream_token_callback(ctx: *mut c_void, token: *const c_char) {
    // Safety: ctx is a Box<StreamContext> allocated in stream_with_options; it remains
    // valid until stream_done_callback drops it.
    let stream_ctx = unsafe { &*(ctx as *const StreamContext) };
    let Some(text) = callback_owned_text(token) else {
        return;
    };
    // Failure here means the Rust ResponseStream was dropped; ignore silently.
    stream_ctx
        .tx
        .unbounded_send(Ok(ResponseText::from(text)))
        .ok();
}

/// Done callback for streaming. Called exactly once; takes ownership of StreamContext.
#[cfg(aimx_bridge)]
extern "C" fn stream_done_callback(ctx: *mut c_void, error: *const c_char) {
    // Safety: takes ownership of the Box<StreamContext> that was created in stream_with_options.
    let stream_ctx = unsafe { Box::from_raw(ctx as *mut StreamContext) };
    if let Some(msg) = callback_owned_text(error) {
        stream_ctx
            .tx
            .unbounded_send(Err(GenerationError::from(msg)))
            .ok();
    }
    // stream_ctx drops here, closing the channel and ending the ResponseStream.
}

/// Dispatches a tool call from Swift to the appropriate Rust handler in `ToolsContext`.
/// Calls `result_cb(result_ctx, result, null)` or `result_cb(result_ctx, null, error)`.
#[cfg(aimx_bridge)]
extern "C" fn tool_dispatch(
    ctx: *mut c_void,
    name_ptr: *const c_char,
    args_ptr: *const c_char,
    result_ctx: *mut c_void,
    result_cb: ToolResultCallback,
) {
    let result = dispatch_tool_call(ctx, name_ptr, args_ptr);
    send_tool_result(result_ctx, result_cb, result);
}

#[cfg(aimx_bridge)]
fn dispatch_tool_call(
    ctx: *mut c_void,
    name_ptr: *const c_char,
    args_ptr: *const c_char,
) -> ToolResult {
    if ctx.is_null() {
        return Err(ToolCallError::new("missing tool context"));
    }

    // Safety: ctx is Arc::as_ptr(&tools_ctx) cast to *mut c_void; the Arc outlives this call.
    let tools = unsafe { &*(ctx as *const ToolsContext) };
    with_callback_text(name_ptr, "tool name", |name| {
        let args = parse_tool_args(args_ptr)?;
        tools.call(name, args)
    })?
}

#[cfg(aimx_bridge)]
fn parse_tool_args(args_ptr: *const c_char) -> Result<serde_json::Value, ToolCallError> {
    if args_ptr.is_null() {
        return Err(ToolCallError::new("missing tool arguments"));
    }

    let args = unsafe { CStr::from_ptr(args_ptr) };
    serde_json::from_slice(args.to_bytes())
        .map_err(|error| ToolCallError::new(format!("invalid tool args JSON: {error}")))
}

#[cfg(aimx_bridge)]
fn with_callback_text<R>(
    ptr: *const c_char,
    label: &str,
    f: impl FnOnce(&str) -> R,
) -> Result<R, ToolCallError> {
    if ptr.is_null() {
        return Err(ToolCallError::new(format!("missing {label}")));
    }

    let text = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
    Ok(f(text.as_ref()))
}

#[cfg(aimx_bridge)]
fn send_tool_result(result_ctx: *mut c_void, result_cb: ToolResultCallback, result: ToolResult) {
    match result {
        Ok(output) => send_tool_output(result_ctx, result_cb, output),
        Err(error) => send_tool_error(result_ctx, result_cb, error.as_str()),
    }
}

#[cfg(aimx_bridge)]
fn send_tool_output(result_ctx: *mut c_void, result_cb: ToolResultCallback, output: ToolOutput) {
    match CString::new(output.into_string()) {
        Ok(c_output) => result_cb(result_ctx, c_output.as_ptr(), null()),
        Err(error) => send_tool_error(
            result_ctx,
            result_cb,
            &format!("tool result contains a null byte: {error}"),
        ),
    }
}

#[cfg(aimx_bridge)]
fn send_tool_error(result_ctx: *mut c_void, result_cb: ToolResultCallback, message: &str) {
    match CString::new(message) {
        Ok(c_error) => result_cb(result_ctx, null(), c_error.as_ptr()),
        Err(_) => result_cb(
            result_ctx,
            null(),
            TOOL_ERROR_ENCODING_FAILURE.as_ptr().cast::<c_char>(),
        ),
    }
}

#[cfg(aimx_bridge)]
const TOOL_ERROR_ENCODING_FAILURE: &[u8] = b"tool error contains a null byte\0";

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── Always-runnable unit tests ────────────────────────────────────────────

    #[test]
    fn test_is_available_returns_without_panic() {
        let _ = is_available();
    }

    #[test]
    fn test_availability_result_is_consistent() {
        let avail = availability();
        assert_eq!(is_available(), avail.is_ok());
    }

    #[test]
    fn test_options_default_is_valid() -> Result<(), Error> {
        let opts = GenerationOptions::default();
        assert!(opts.validate().is_ok());
        let config = opts.validated()?;
        assert_eq!(config.ffi_temperature(), -1.0);
        assert_eq!(config.ffi_max_tokens(), -1);
        Ok(())
    }

    #[test]
    fn test_options_valid_temperature() -> Result<(), Error> {
        for (temp, expected_ffi) in [(0.0_f64, 0.0), (1.0, 1.0), (2.0, 2.0)] {
            let opts = GenerationOptions::new().try_temperature(temp)?;
            assert!(
                opts.validate().is_ok(),
                "temperature {temp} should be valid"
            );
            let config = opts.validated()?;
            assert_eq!(config.ffi_temperature(), expected_ffi);
        }
        Ok(())
    }

    #[test]
    fn test_options_invalid_temperature() {
        for temp in [-f64::INFINITY, -0.1_f64, 2.001, f64::INFINITY, f64::NAN] {
            assert!(
                GenerationOptions::new().try_temperature(temp).is_err(),
                "temperature {temp} should be invalid"
            );
        }
    }

    #[test]
    fn test_options_invalid_max_tokens() {
        if usize::BITS < i64::BITS {
            return;
        }

        let invalid = MaxTokens::MAX + 1;

        assert!(matches!(
            GenerationOptions::new().try_max_tokens(invalid),
            Err(Error::InvalidMaxTokens(value)) if value == invalid
        ));
        assert!(matches!(
            MaxTokens::new(invalid),
            Err(Error::InvalidMaxTokens(value)) if value == invalid
        ));
    }

    #[test]
    fn test_session_creation_fails_gracefully_when_unavailable() {
        if is_available() {
            return; // skip — integration tests cover the available path
        }
        assert!(matches!(
            LanguageModelSession::new(),
            Err(Error::Unavailable(_))
        ));
    }

    #[test]
    fn test_null_byte_in_prompt_returns_error() {
        let result = futures_executor::block_on(respond("hello\0world"));
        assert!(matches!(result, Err(Error::NullByte(_))));
    }

    #[test]
    fn test_prompt_and_instruction_inputs_reject_null_bytes_before_availability() {
        let prompt = Prompt::try_from("hello\0world");
        let instructions = SystemInstructions::try_from("system\0prompt");

        assert!(matches!(prompt, Err(Error::NullByte(_))));
        assert!(matches!(instructions, Err(Error::NullByte(_))));
    }

    #[test]
    fn test_session_builder_validates_options_before_availability() {
        let result = AppleIntelligenceModels::default()
            .session()
            .instructions("Valid system prompt")
            .try_temperature(2.5)
            .and_then(LanguageModelSessionBuilder::build);

        assert!(matches!(result, Err(Error::InvalidTemperature(value)) if value == 2.5));
    }

    #[test]
    fn test_options_expose_typed_values() -> Result<(), Error> {
        let temperature = Temperature::new(0.4)?;
        let max_tokens = MaxTokens::new(128)?;
        let opts = GenerationOptions::new()
            .temperature(temperature)
            .max_tokens(max_tokens);

        assert_eq!(opts.temperature_value(), Some(temperature));
        assert_eq!(opts.max_tokens_value(), Some(max_tokens));
        Ok(())
    }

    #[test]
    fn test_schema_property_requirement_serializes_as_optional_flag() -> Result<(), Error> {
        let schema = GenerationSchema::new("Answer")
            .property(GenerationSchemaProperty::new(
                "required",
                GenerationSchemaPropertyType::String,
            ))
            .property(
                GenerationSchemaProperty::new("maybe", GenerationSchemaPropertyType::String)
                    .optional(),
            );

        let json = serde_json::to_value(schema)?;

        assert_eq!(json["properties"][0]["optional"], false);
        assert_eq!(json["properties"][1]["optional"], true);
        assert!(GenerationSchemaPropertyRequirement::Required.is_required());
        assert!(GenerationSchemaPropertyRequirement::Optional.is_optional());
        Ok(())
    }

    #[test]
    fn test_session_builder_validates_instructions_before_availability() {
        let result = AppleIntelligenceModels::default()
            .session()
            .instructions("bad\0instructions")
            .build();

        assert!(matches!(result, Err(Error::NullByte(_))));
    }

    #[test]
    fn test_string_newtypes_round_trip_through_display_and_inner_value() {
        let cases = [
            PromptText::new("prompt").into_string(),
            ResponseText::new("response").to_string(),
            GenerationSchemaName::new("GenerationSchema").to_string(),
            GenerationSchemaPropertyName::new("field").to_string(),
            ToolName::new("tool").to_string(),
            ToolOutput::new("output").to_string(),
        ];

        assert_eq!(
            cases,
            [
                "prompt",
                "response",
                "GenerationSchema",
                "field",
                "tool",
                "output"
            ]
        );
    }

    #[test]
    fn test_schema_builder() -> Result<(), Error> {
        let schema = GenerationSchema::new("Point")
            .description("A 2D point")
            .property(
                GenerationSchemaProperty::new("x", GenerationSchemaPropertyType::Double)
                    .description("X axis"),
            )
            .property(GenerationSchemaProperty::new(
                "y",
                GenerationSchemaPropertyType::Double,
            ));
        assert_eq!(schema.name, "Point");
        assert_eq!(schema.properties.len(), 2);
        let json = serde_json::to_string(&schema)?;
        assert!(json.contains("\"x\""));
        assert!(json.contains("\"double\""));
        Ok(())
    }

    #[test]
    fn test_tool_definition_builder() -> Result<(), ToolCallError> {
        let tool = ToolDefinition::builder(
            "add",
            "Add two numbers",
            GenerationSchema::new("AddArgs")
                .property(GenerationSchemaProperty::new(
                    "a",
                    GenerationSchemaPropertyType::Double,
                ))
                .property(GenerationSchemaProperty::new(
                    "b",
                    GenerationSchemaPropertyType::Double,
                )),
        )
        .handler(|args| {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Ok(ToolOutput::from(format!("{}", a + b)))
        });
        assert_eq!(tool.name, "add");
        let result = tool.call(serde_json::json!({"a": 3.0, "b": 4.0}));
        assert_eq!(result?, "7");
        Ok(())
    }

    #[test]
    fn test_tool_definition_new_and_trait_boundary() -> Result<(), ToolCallError> {
        let tool = ToolDefinition::new(
            "echo",
            "Echo an input string",
            GenerationSchema::new("EchoArgs").property(GenerationSchemaProperty::new(
                "value",
                GenerationSchemaPropertyType::String,
            )),
            |args| {
                args["value"]
                    .as_str()
                    .map(ToolOutput::from)
                    .ok_or_else(|| ToolCallError::new("missing value"))
            },
        );

        assert_eq!(tool.name().as_str(), "echo");
        assert_eq!(tool.description().as_str(), "Echo an input string");
        assert_eq!(tool.parameters().name, "EchoArgs");
        assert_eq!(tool.call(serde_json::json!({"value": "hello"}))?, "hello");
        assert!(tool.call(serde_json::json!({})).is_err());
        Ok(())
    }

    #[test]
    fn test_tool_handler_panic_returns_tool_error() {
        let tool = ToolDefinition::new(
            "panic_tool",
            "Tool that fails inside user code",
            GenerationSchema::new("PanicArgs"),
            |_| -> ToolResult {
                std::panic::resume_unwind(Box::new("boom"));
            },
        );

        let error = tool.call(serde_json::json!({})).err();

        assert!(
            error
                .as_ref()
                .is_some_and(|error| error.as_str().contains("tool handler panicked: boom")),
            "expected panic to be converted into ToolCallError"
        );
    }

    proptest! {
        #[test]
        fn proptest_prompt_input_matches_c_string_null_boundary(input in ".*") {
            let result = Prompt::try_from(input.as_str());

            if input.contains('\0') {
                prop_assert!(matches!(result, Err(Error::NullByte(_))));
            } else {
                match result {
                    Ok(prompt) => prop_assert_eq!(prompt.as_str(), input.as_str()),
                    Err(error) => prop_assert!(false, "unexpected prompt error: {error}"),
                }
            }
        }

        #[test]
        fn proptest_instructions_match_c_string_null_boundary(input in ".*") {
            let result = SystemInstructions::try_from(input.as_str());

            if input.contains('\0') {
                prop_assert!(matches!(result, Err(Error::NullByte(_))));
            } else {
                match result {
                    Ok(instructions) => prop_assert_eq!(instructions.as_str(), input.as_str()),
                    Err(error) => prop_assert!(false, "unexpected instructions error: {error}"),
                }
            }
        }

        #[test]
        fn proptest_temperature_validation_matches_closed_interval(temp in any::<f64>()) {
            let result = Temperature::new(temp);

            if (Temperature::MIN..=Temperature::MAX).contains(&temp) {
                match result {
                    Ok(temperature) => prop_assert_eq!(temperature.as_f64(), temp),
                    Err(error) => prop_assert!(false, "unexpected temperature error: {error}"),
                }
            } else {
                prop_assert!(matches!(result, Err(Error::InvalidTemperature(value)) if value.to_bits() == temp.to_bits()));
            }
        }

        #[test]
        fn proptest_generation_options_preserve_max_tokens(max_tokens in any::<usize>()) {
            if max_tokens <= MaxTokens::MAX {
                match GenerationOptions::new().try_max_tokens(max_tokens) {
                    Ok(opts) => match opts.validated() {
                        Ok(config) => prop_assert_eq!(config.ffi_max_tokens(), max_tokens as i64),
                        Err(error) => prop_assert!(false, "unexpected options error: {error}"),
                    },
                    Err(error) => prop_assert!(false, "unexpected max token error: {error}"),
                }
            } else {
                prop_assert!(matches!(
                    GenerationOptions::new().try_max_tokens(max_tokens),
                    Err(Error::InvalidMaxTokens(value)) if value == max_tokens
                ));
            }
        }
    }

    // ── Integration tests (require Apple Intelligence) ─────────────────────────
    //
    // Run with:  cargo test -- --include-ignored

    #[test]
    #[ignore = "requires Apple Intelligence (macOS 26+, Apple Silicon, AI enabled)"]
    fn test_simple_respond() -> Result<(), Error> {
        let response =
            futures_executor::block_on(respond("Reply with only the number: what is 2 + 2?"))?;
        assert!(
            response.as_str().contains('4'),
            "expected '4' in: {response:?}"
        );
        Ok(())
    }

    #[test]
    #[ignore = "requires Apple Intelligence"]
    fn test_respond_with_low_temperature() -> Result<(), Error> {
        let opts = GenerationOptions::new().temperature(Temperature::new(0.0)?);
        let r = futures_executor::block_on(respond_with_options(
            "Reply with only the word: capital of France?",
            &opts,
        ))?;
        assert!(
            r.as_str().to_lowercase().contains("paris"),
            "expected Paris in: {r:?}"
        );
        Ok(())
    }

    #[test]
    #[ignore = "requires Apple Intelligence"]
    fn test_multi_turn_session() -> Result<(), Error> {
        let session = LanguageModelSession::with_instructions(
            "Reply to every message with exactly one word.",
        )?;
        let r1 = futures_executor::block_on(session.respond_to("Say hello."))?;
        let r2 = futures_executor::block_on(session.respond_to("Say goodbye."))?;
        assert!(!r1.is_empty(), "first response was empty");
        assert!(!r2.is_empty(), "second response was empty");
        Ok(())
    }

    #[test]
    #[ignore = "requires Apple Intelligence"]
    fn test_streaming_yields_chunks() -> Result<(), Error> {
        let session = LanguageModelSession::new()?;
        let stream = session.stream_response("Count: one two three")?;

        let chunks: Vec<ResponseText> =
            futures_executor::block_on_stream(stream).collect::<Result<_, _>>()?;

        assert!(!chunks.is_empty(), "stream produced no chunks");
        let full = chunks
            .into_iter()
            .map(ResponseText::into_string)
            .collect::<Vec<_>>()
            .join("");
        assert!(!full.is_empty(), "concatenated response was empty");
        Ok(())
    }

    #[test]
    #[ignore = "requires Apple Intelligence"]
    fn test_structured_generation() -> Result<(), Error> {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct MathAnswer {
            value: f64,
            explanation: String,
        }

        let session = LanguageModelSession::new()?;
        let schema = GenerationSchema::new("MathAnswer")
            .description("A numeric answer with a brief explanation")
            .property(
                GenerationSchemaProperty::new("value", GenerationSchemaPropertyType::Double)
                    .description("The numeric result"),
            )
            .property(
                GenerationSchemaProperty::new("explanation", GenerationSchemaPropertyType::String)
                    .description("One-sentence explanation"),
            );

        let answer: MathAnswer =
            futures_executor::block_on(session.respond_generating("What is 6 × 7?", &schema))?;

        assert!(
            (answer.value - 42.0).abs() < 0.5,
            "expected 42, got {}",
            answer.value
        );
        assert!(!answer.explanation.is_empty(), "explanation was empty");
        Ok(())
    }

    #[test]
    #[ignore = "requires Apple Intelligence"]
    fn test_tool_calling() -> Result<(), Error> {
        let tool = ToolDefinition::builder(
            "add_numbers",
            "Add two numbers together and return the sum",
            GenerationSchema::new("AddArgs")
                .property(
                    GenerationSchemaProperty::new("a", GenerationSchemaPropertyType::Double)
                        .description("First number"),
                )
                .property(
                    GenerationSchemaProperty::new("b", GenerationSchemaPropertyType::Double)
                        .description("Second number"),
                ),
        )
        .handler(|args| {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Ok(ToolOutput::from(format!("{}", a + b)))
        });

        let session = LanguageModelSession::with_tools(
            "You are a calculator. Use the add_numbers tool when asked to add.",
            vec![tool],
        )?;

        let response = futures_executor::block_on(session.respond_to("What is 15 + 27?"))?;

        assert!(
            response.as_str().contains("42"),
            "expected 42 in response: {response:?}"
        );
        Ok(())
    }
}
