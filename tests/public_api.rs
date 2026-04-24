use aimx::{
    AppleIntelligenceModels, Error, GenerationOptions, GenerationSchema, GenerationSchemaProperty,
    GenerationSchemaPropertyType, MaxTokens, Prompt, ResponseField, ResponseFieldType,
    ResponseSchema, Temperature, Tool, ToolCallError, ToolDefinition, ToolOutput,
};
use futures_executor::block_on;

#[test]
fn apple_intelligence_builder_rejects_invalid_temperature_before_model_availability() {
    let result = AppleIntelligenceModels::default()
        .session()
        .instructions("Use concise answers.")
        .try_temperature(-0.01)
        .and_then(|builder| builder.build());

    assert!(matches!(result, Err(Error::InvalidTemperature(value)) if value == -0.01));
}

#[test]
fn apple_intelligence_builder_rejects_null_instructions_before_model_availability() {
    let result = AppleIntelligenceModels::default()
        .session()
        .instructions("bad\0instructions")
        .build();

    assert!(matches!(result, Err(Error::NullByte(_))));
}

#[test]
fn prompt_validation_is_public_and_independent_of_model_availability() -> Result<(), Error> {
    let prompt = Prompt::try_from("safe prompt")?;
    assert_eq!(prompt.as_str(), "safe prompt");

    assert!(matches!(
        Prompt::try_from("bad\0prompt"),
        Err(Error::NullByte(_))
    ));
    Ok(())
}

#[test]
fn apple_intelligence_models_reject_null_prompt_before_model_availability() {
    let result = block_on(AppleIntelligenceModels::default().respond("bad\0prompt"));

    assert!(matches!(result, Err(Error::NullByte(_))));
}

#[test]
fn apple_and_mlx_semantic_aliases_reject_null_prompt_before_availability() {
    let model = AppleIntelligenceModels::default();

    let generate_result = block_on(model.generate("bad\0prompt"));
    let stream_result = model.stream_generate("bad\0prompt");

    assert!(matches!(generate_result, Err(Error::NullByte(_))));
    assert!(matches!(stream_result, Err(Error::NullByte(_))));
}

#[test]
fn generation_options_validate_numeric_boundaries() -> Result<(), Error> {
    assert!(GenerationOptions::new()
        .temperature(Temperature::new(0.0)?)
        .max_tokens(MaxTokens::new(512)?)
        .validate()
        .is_ok());

    assert!(matches!(
        GenerationOptions::new().try_temperature(f64::NAN),
        Err(Error::InvalidTemperature(value)) if value.is_nan()
    ));

    if usize::BITS >= i64::BITS {
        let invalid = MaxTokens::MAX + 1;
        assert!(matches!(
            GenerationOptions::new().try_max_tokens(invalid),
            Err(Error::InvalidMaxTokens(value)) if value == invalid
        ));
    }

    Ok(())
}

#[test]
fn response_schema_aliases_still_compile() {
    let schema: ResponseSchema = GenerationSchema::new("AliasAnswer")
        .property(ResponseField::new("title", ResponseFieldType::String));

    assert_eq!(schema.name.as_str(), "AliasAnswer");
}

#[test]
fn schema_serializes_with_typed_names_and_descriptions() -> Result<(), Error> {
    let schema = GenerationSchema::new("Answer")
        .description("Structured answer")
        .property(
            GenerationSchemaProperty::new("title", GenerationSchemaPropertyType::String)
                .description("Short title"),
        )
        .property(
            GenerationSchemaProperty::new("score", GenerationSchemaPropertyType::Double).optional(),
        );

    let json = serde_json::to_value(&schema)?;

    assert_eq!(json["name"], "Answer");
    assert_eq!(json["description"], "Structured answer");
    assert_eq!(json["properties"][0]["name"], "title");
    assert_eq!(json["properties"][0]["type"], "string");
    assert_eq!(json["properties"][1]["optional"], true);
    Ok(())
}

#[test]
fn tool_definition_uses_typed_trait_boundary() -> Result<(), ToolCallError> {
    let tool = ToolDefinition::builder(
        "double",
        "Double a numeric value",
        GenerationSchema::new("DoubleArgs").property(GenerationSchemaProperty::new(
            "value",
            GenerationSchemaPropertyType::Double,
        )),
    )
    .handler(|args| {
        let value = args["value"]
            .as_f64()
            .ok_or_else(|| ToolCallError::new("missing numeric value"))?;
        Ok(ToolOutput::from(format!("{}", value * 2.0)))
    });

    assert_eq!(tool.name().as_str(), "double");
    assert_eq!(tool.description().as_str(), "Double a numeric value");
    assert_eq!(tool.call(serde_json::json!({ "value": 21.0 }))?, "42");
    assert!(tool.call(serde_json::json!({})).is_err());
    Ok(())
}
