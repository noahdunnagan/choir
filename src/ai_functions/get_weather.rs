use super::{AIFunction, AIFunctionParameter};
use crate::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Deserialize)]
struct GetWeatherArgs {
    location: String,
    units: Option<String>,
}

#[derive(Serialize)]
struct GetWeatherResponse {
    location: String,
    units: String,
    weather_data: String,
    timestamp: String,
}

pub struct GetWeatherFunction;

#[async_trait]
impl AIFunction for GetWeatherFunction {
    fn name(&self) -> &'static str {
        "get_weather"
    }

    fn description(&self) -> &'static str {
        "Get current weather information for a specified location"
    }

    fn parameters(&self) -> HashMap<String, AIFunctionParameter> {
        let mut params = HashMap::new();
        params.insert(
            "location".to_string(),
            AIFunctionParameter::new(
                "string",
                "The location to get weather for (city, state/country)",
                true,
            ),
        );
        params.insert(
            "units".to_string(),
            AIFunctionParameter::new(
                "string",
                "Temperature units: celsius, fahrenheit, kelvin",
                false,
            ),
        );
        params
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<Value, Error> {
        let args: GetWeatherArgs =
            serde_json::from_value(serde_json::Value::Object(args.into_iter().collect()))?;

        let units = args.units.unwrap_or_else(|| "celsius".to_string());
        let response_data = format!("Its currently 69dg {}. in {}", units, args.location); // Placeholder

        let response = GetWeatherResponse {
            location: args.location,
            units,
            weather_data: response_data,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Ok(json!(response))
    }
}
