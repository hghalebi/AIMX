#[cfg(test)]
use aimx::Tool;
use aimx::{
    AppleIntelligenceModels, Error, GenerationOptions, GenerationSchema, GenerationSchemaProperty,
    GenerationSchemaPropertyType, LanguageModelSessionBuilder, MaxTokens, Prompt,
    SystemInstructions, Temperature, ToolCallError, ToolDefinition, ToolOutput,
};
#[cfg(test)]
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentUseCase {
    ResearchBrief,
    SupportTriage,
    CodeReview,
    ReleaseNotes,
    MeetingActions,
    WeatherTool,
}

#[derive(Debug, Clone)]
struct AgentPlan {
    use_case: AgentUseCase,
    instructions: SystemInstructions,
    prompt: Prompt,
    options: GenerationOptions,
}

fn main() -> Result<(), Error> {
    for use_case in AgentUseCase::all() {
        let plan = agent_plan(*use_case)?;
        let schema_label = if output_schema(*use_case).is_some() {
            "structured"
        } else {
            "text"
        };
        let tool_label = if tools_for(*use_case).is_empty() {
            "no tools"
        } else {
            "tool augmented"
        };

        println!(
            "{}: {}, {}, prompt={} chars",
            plan.name(),
            schema_label,
            tool_label,
            plan.prompt.as_str().len()
        );
    }

    if std::env::var_os("AIMX_RUN_LIVE_AGENT_EXAMPLES").is_some() {
        futures_executor::block_on(run_live_research_brief())?;
    }

    Ok(())
}

async fn run_live_research_brief() -> Result<(), Error> {
    let plan = agent_plan(AgentUseCase::ResearchBrief)?;
    let session = builder_for(&plan).build()?;
    let response = session.respond_to(plan.prompt.as_str()).await?;
    println!("{response}");
    Ok(())
}

fn builder_for(plan: &AgentPlan) -> LanguageModelSessionBuilder {
    let builder = AppleIntelligenceModels::default()
        .session()
        .instructions(plan.instructions.as_str())
        .options(plan.options.clone());

    tools_for(plan.use_case)
        .into_iter()
        .fold(builder, LanguageModelSessionBuilder::tool)
}

fn agent_plan(use_case: AgentUseCase) -> Result<AgentPlan, Error> {
    Ok(AgentPlan {
        use_case,
        instructions: SystemInstructions::new(use_case.instructions())?,
        prompt: Prompt::new(use_case.prompt())?,
        options: GenerationOptions::new()
            .temperature(Temperature::new(use_case.temperature())?)
            .max_tokens(MaxTokens::new(use_case.max_tokens())?),
    })
}

fn output_schema(use_case: AgentUseCase) -> Option<GenerationSchema> {
    match use_case {
        AgentUseCase::SupportTriage => Some(
            GenerationSchema::new("SupportTriage")
                .description("Routing decision for an incoming customer support ticket")
                .property(GenerationSchemaProperty::new(
                    "priority",
                    GenerationSchemaPropertyType::String,
                ))
                .property(GenerationSchemaProperty::new(
                    "team",
                    GenerationSchemaPropertyType::String,
                ))
                .property(
                    GenerationSchemaProperty::new("summary", GenerationSchemaPropertyType::String)
                        .description("One-sentence customer-visible summary"),
                ),
        ),
        AgentUseCase::ReleaseNotes => Some(
            GenerationSchema::new("ReleaseNote")
                .description("Release note generated from engineering changes")
                .property(GenerationSchemaProperty::new(
                    "title",
                    GenerationSchemaPropertyType::String,
                ))
                .property(GenerationSchemaProperty::new(
                    "summary",
                    GenerationSchemaPropertyType::String,
                ))
                .property(
                    GenerationSchemaProperty::new("risk", GenerationSchemaPropertyType::String)
                        .optional(),
                ),
        ),
        AgentUseCase::MeetingActions => Some(
            GenerationSchema::new("MeetingActions")
                .description("Action items extracted from a meeting transcript")
                .property(GenerationSchemaProperty::new(
                    "owner",
                    GenerationSchemaPropertyType::String,
                ))
                .property(GenerationSchemaProperty::new(
                    "action",
                    GenerationSchemaPropertyType::String,
                ))
                .property(GenerationSchemaProperty::new(
                    "due_date",
                    GenerationSchemaPropertyType::String,
                )),
        ),
        AgentUseCase::ResearchBrief | AgentUseCase::CodeReview | AgentUseCase::WeatherTool => None,
    }
}

fn tools_for(use_case: AgentUseCase) -> Vec<ToolDefinition> {
    match use_case {
        AgentUseCase::WeatherTool => vec![weather_tool()],
        AgentUseCase::ResearchBrief
        | AgentUseCase::SupportTriage
        | AgentUseCase::CodeReview
        | AgentUseCase::ReleaseNotes
        | AgentUseCase::MeetingActions => Vec::new(),
    }
}

fn weather_tool() -> ToolDefinition {
    let parameters = GenerationSchema::new("WeatherArgs")
        .description("City lookup arguments")
        .property(
            GenerationSchemaProperty::new("city", GenerationSchemaPropertyType::String)
                .description("City name"),
        );

    ToolDefinition::builder(
        "get_weather",
        "Return deterministic demo weather for a city",
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

impl AgentPlan {
    fn name(&self) -> &'static str {
        self.use_case.name()
    }
}

impl AgentUseCase {
    fn all() -> &'static [AgentUseCase] {
        &[
            AgentUseCase::ResearchBrief,
            AgentUseCase::SupportTriage,
            AgentUseCase::CodeReview,
            AgentUseCase::ReleaseNotes,
            AgentUseCase::MeetingActions,
            AgentUseCase::WeatherTool,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            AgentUseCase::ResearchBrief => "research_brief",
            AgentUseCase::SupportTriage => "support_triage",
            AgentUseCase::CodeReview => "code_review",
            AgentUseCase::ReleaseNotes => "release_notes",
            AgentUseCase::MeetingActions => "meeting_actions",
            AgentUseCase::WeatherTool => "weather_tool",
        }
    }

    fn instructions(self) -> &'static str {
        match self {
            AgentUseCase::ResearchBrief => {
                "You synthesize source notes into a concise research brief with claims, evidence, and uncertainty."
            }
            AgentUseCase::SupportTriage => {
                "You classify support tickets by priority, owning team, and customer-visible summary."
            }
            AgentUseCase::CodeReview => {
                "You review Rust changes for correctness, error handling, public API drift, and missing tests."
            }
            AgentUseCase::ReleaseNotes => {
                "You convert engineering changes into clear release notes for developers."
            }
            AgentUseCase::MeetingActions => {
                "You extract action items from meeting transcripts with owner, task, and due date."
            }
            AgentUseCase::WeatherTool => {
                "You answer weather questions by calling the provided weather tool when a city is present."
            }
        }
    }

    fn prompt(self) -> &'static str {
        match self {
            AgentUseCase::ResearchBrief => {
                "Summarize these notes: local models improve privacy; typed FFI boundaries reduce crash risk; benchmarks should separate model latency from wrapper overhead."
            }
            AgentUseCase::SupportTriage => {
                "Ticket: The export button fails for all admins after the latest release. Customer impact is high."
            }
            AgentUseCase::CodeReview => {
                "Review this change: a Rust FFI wrapper replaced typed errors with String and added unwrap in callback handling."
            }
            AgentUseCase::ReleaseNotes => {
                "Changes: renamed crate to AIMX, added typed prompt boundaries, added Criterion benchmarks."
            }
            AgentUseCase::MeetingActions => {
                "Transcript: Maya will update docs by Friday. Sam owns benchmark follow-up next week."
            }
            AgentUseCase::WeatherTool => "What is the weather in Tokyo?",
        }
    }

    fn temperature(self) -> f64 {
        match self {
            AgentUseCase::CodeReview | AgentUseCase::SupportTriage => 0.1,
            AgentUseCase::ResearchBrief | AgentUseCase::ReleaseNotes => 0.2,
            AgentUseCase::MeetingActions | AgentUseCase::WeatherTool => 0.0,
        }
    }

    fn max_tokens(self) -> usize {
        match self {
            AgentUseCase::ResearchBrief => 512,
            AgentUseCase::SupportTriage => 160,
            AgentUseCase::CodeReview => 384,
            AgentUseCase::ReleaseNotes => 220,
            AgentUseCase::MeetingActions => 180,
            AgentUseCase::WeatherTool => 96,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_agent_plans_have_valid_boundaries_and_options() -> Result<(), Error> {
        for use_case in AgentUseCase::all() {
            let plan = agent_plan(*use_case)?;

            assert_eq!(plan.name(), use_case.name());
            assert!(!plan.instructions.as_str().is_empty());
            assert!(!plan.prompt.as_str().is_empty());
            assert!(plan.options.validate().is_ok());
        }

        Ok(())
    }

    #[test]
    fn code_review_agent_prompt_keeps_review_semantics() -> Result<(), Error> {
        let plan = agent_plan(AgentUseCase::CodeReview)?;

        assert!(plan.instructions.as_str().contains("correctness"));
        assert!(plan.instructions.as_str().contains("missing tests"));
        assert!(plan.prompt.as_str().contains("typed errors"));
        assert!(plan.prompt.as_str().contains("unwrap"));
        Ok(())
    }

    #[test]
    fn support_triage_schema_matches_expected_contract() -> Result<(), String> {
        let schema = match output_schema(AgentUseCase::SupportTriage) {
            Some(schema) => schema,
            None => {
                return Err("support triage should have a schema".to_string());
            }
        };
        let json = serde_json::to_value(schema).map_err(|error| error.to_string())?;

        assert_eq!(json["name"], "SupportTriage");
        assert_eq!(json["properties"][0]["name"], "priority");
        assert_eq!(json["properties"][1]["name"], "team");
        assert_eq!(json["properties"][2]["name"], "summary");
        assert_eq!(json["properties"][2]["optional"], false);
        Ok(())
    }

    #[test]
    fn release_notes_schema_marks_risk_optional() -> Result<(), String> {
        let schema = match output_schema(AgentUseCase::ReleaseNotes) {
            Some(schema) => schema,
            None => {
                return Err("release notes should have a schema".to_string());
            }
        };
        let json = serde_json::to_value(schema).map_err(|error| error.to_string())?;

        assert_eq!(json["name"], "ReleaseNote");
        assert_eq!(json["properties"][0]["name"], "title");
        assert_eq!(json["properties"][1]["name"], "summary");
        assert_eq!(json["properties"][2]["name"], "risk");
        assert_eq!(json["properties"][2]["optional"], true);
        Ok(())
    }

    #[test]
    fn meeting_actions_schema_tracks_owner_action_and_due_date() -> Result<(), String> {
        let schema = match output_schema(AgentUseCase::MeetingActions) {
            Some(schema) => schema,
            None => {
                return Err("meeting actions should have a schema".to_string());
            }
        };
        let json = serde_json::to_value(schema).map_err(|error| error.to_string())?;

        assert_eq!(json["name"], "MeetingActions");
        assert_eq!(json["properties"][0]["name"], "owner");
        assert_eq!(json["properties"][1]["name"], "action");
        assert_eq!(json["properties"][2]["name"], "due_date");
        Ok(())
    }

    #[test]
    fn weather_tool_returns_expected_output_for_city() -> Result<(), ToolCallError> {
        let tool = weather_tool();
        let result = Tool::call(&tool, json!({ "city": "Tokyo" }))?;

        assert_eq!(result.as_str(), "Tokyo: 22 C, sunny");
        Ok(())
    }

    #[test]
    fn weather_tool_returns_expected_error_for_missing_city() {
        let tool = weather_tool();
        let result = Tool::call(&tool, json!({ "country": "Japan" }));

        assert!(matches!(
            result,
            Err(error) if error.as_str() == "missing string field: city"
        ));
    }

    #[test]
    fn builder_rejects_null_instructions_before_model_availability() {
        let result = AppleIntelligenceModels::default()
            .session()
            .instructions("bad\0instructions")
            .build();

        assert!(matches!(result, Err(Error::NullByte(_))));
    }

    #[test]
    fn live_builder_can_be_created_without_running_model() -> Result<(), Error> {
        let plan = agent_plan(AgentUseCase::WeatherTool)?;
        let _builder = builder_for(&plan);

        Ok(())
    }
}
