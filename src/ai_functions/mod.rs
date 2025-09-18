use crate::Error;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

#[async_trait]
pub trait AIFunction: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> HashMap<String, AIFunctionParameter>;
    async fn execute(&self, args: HashMap<String, Value>) -> Result<Value, Error>;
}

#[derive(Clone, Debug)]
pub struct AIFunctionParameter {
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

impl AIFunctionParameter {
    pub fn new(param_type: &str, description: &str, required: bool) -> Self {
        Self {
            param_type: param_type.to_string(),
            description: description.to_string(),
            required,
        }
    }
}

pub mod get_weather;
pub mod website_to_md;

pub fn get_all_functions() -> Vec<Box<dyn AIFunction>> {
    vec![
        Box::new(get_weather::GetWeatherFunction),
        Box::new(website_to_md::WebsiteToMdFunction),
    ]
}
