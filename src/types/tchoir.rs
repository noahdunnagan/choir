use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChoirRequest {
    pub query: String,
    pub json_schema: Option<Value>,
}

// Agent response.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ChoirAgentResponse {
    /// The agent's response in detail. Extremely detailed.
    pub detailed_response: String,
    /// A short overview of the response. Just a few sentences.
    pub short_overview: String,
    /// Thoughts on the response. Just a few sentences.
    pub thoughts: String,
}

// Kinda like doing this in types instead of route specific logic in case of future changes.
pub fn get_choir_agent_response_schema() -> Value {
    serde_json::to_value(schema_for!(ChoirAgentResponse)).unwrap()
}

impl ChoirAgentResponse {
    pub fn empty() -> Self {
        Self {
            detailed_response: "Agent failed while generating. Discount them from processing."
                .to_string(),
            short_overview: "Agent failed while generating. Discount them from processing."
                .to_string(),
            thoughts: "Agent failed while generating. Discount them from processing.".to_string(),
        }
    }
}
