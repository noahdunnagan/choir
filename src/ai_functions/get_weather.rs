use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::Error;
use super::{AIFunction, AIFunctionParameter};

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
            AIFunctionParameter::new("string", "The location to get weather for (city, state/country)", true),
        );
        params.insert(
            "units".to_string(),
            AIFunctionParameter::new("string", "Temperature units: celsius, fahrenheit, kelvin", false),
        );
        params
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<Value, Error> {
        let location = args.get("location")
            .and_then(|v| v.as_str())
            .ok_or("Missing required parameter: location")?;

        let units = args.get("units")
            .and_then(|v| v.as_str())
            .unwrap_or("celsius");
        let response = format!("Its currently 69dg {}. in {}", units, location);  // Placeholder, didnt bring over the perplexity api. 
        
        Ok(json!({
            "location": location,
            "units": units,
            "weather_data": response,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}