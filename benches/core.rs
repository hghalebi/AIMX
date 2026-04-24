use aimx::{
    AppleIntelligenceModels, GenerationOptions, GenerationSchema, GenerationSchemaProperty,
    GenerationSchemaPropertyType, MaxTokens, Prompt, SystemInstructions, Temperature, Tool,
    ToolCallError, ToolDefinition, ToolOutput,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn prompt_and_instruction_boundaries(c: &mut Criterion) {
    let prompt = "Explain typed FFI boundaries in one short paragraph.";
    let instructions = "You are a concise assistant for Rust engineers.";

    c.bench_function("prompt/new/valid_ascii", |b| {
        b.iter(|| black_box(Prompt::try_from(black_box(prompt))))
    });

    c.bench_function("instructions/new/valid_ascii", |b| {
        b.iter(|| black_box(SystemInstructions::new(black_box(instructions))))
    });
}

fn generation_options(c: &mut Criterion) {
    let temperature = valid_temperature(0.2);
    let max_tokens = valid_max_tokens(256);

    c.bench_function("generation_options/typed_build_and_validate", |b| {
        b.iter(|| {
            let options = GenerationOptions::new()
                .temperature(black_box(temperature))
                .max_tokens(black_box(max_tokens));

            black_box(options.validate())
        })
    });

    c.bench_function("generation_options/raw_boundary_build", |b| {
        b.iter(|| {
            let options = GenerationOptions::new()
                .try_temperature(black_box(0.2))
                .and_then(|options| options.try_max_tokens(black_box(256)));

            black_box(options)
        })
    });
}

fn schema_fixture() -> GenerationSchema {
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
                .description("Main release risk or fallback")
                .optional(),
        )
}

fn schema_serialization(c: &mut Criterion) {
    c.bench_function("generation_schema/build_3_fields", |b| {
        b.iter(|| black_box(schema_fixture()))
    });

    let schema = schema_fixture();
    c.bench_function("generation_schema/serialize_3_fields_json", |b| {
        b.iter(|| black_box(serde_json::to_vec(black_box(&schema))))
    });
}

fn tool_dispatch(c: &mut Criterion) {
    let tool = ToolDefinition::builder(
        "get_weather",
        "Return current weather for a city",
        GenerationSchema::new("WeatherArgs").property(GenerationSchemaProperty::new(
            "city",
            GenerationSchemaPropertyType::String,
        )),
    )
    .handler(|args| {
        let city = args
            .get("city")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ToolCallError::new("missing string field: city"))?;

        Ok(ToolOutput::from(format!("{city}: 22 C, sunny")))
    });
    let args = json!({ "city": "Tokyo" });

    c.bench_function("tool/call_success_json_value", |b| {
        b.iter(|| black_box(Tool::call(black_box(&tool), black_box(args.clone()))))
    });
}

fn session_builder_boundary(c: &mut Criterion) {
    let temperature = valid_temperature(0.2);
    let max_tokens = valid_max_tokens(128);

    c.bench_function("session_builder/validate_then_unavailable", |b| {
        b.iter(|| {
            let session = AppleIntelligenceModels::default()
                .session()
                .instructions("You are concise.")
                .temperature(black_box(temperature))
                .max_tokens(black_box(max_tokens))
                .build();

            black_box(session)
        })
    });
}

fn valid_temperature(value: f64) -> Temperature {
    match Temperature::new(value) {
        Ok(temperature) => temperature,
        Err(error) => {
            eprintln!("invalid benchmark temperature fixture: {error}");
            std::process::exit(2);
        }
    }
}

fn valid_max_tokens(value: usize) -> MaxTokens {
    match MaxTokens::new(value) {
        Ok(max_tokens) => max_tokens,
        Err(error) => {
            eprintln!("invalid benchmark max_tokens fixture: {error}");
            std::process::exit(2);
        }
    }
}

criterion_group!(
    benches,
    prompt_and_instruction_boundaries,
    generation_options,
    schema_serialization,
    tool_dispatch,
    session_builder_boundary
);
criterion_main!(benches);
