use super::{AIFunction, AIFunctionParameter};
use crate::{config::EnvConfig, Error};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Deserialize)]
struct WebsiteToMdArgs {
    url: String,
}

#[derive(Serialize)]
struct WebsiteToMdResponse {
    url: String,
    markdown: String,
    title: Option<String>,
}

pub struct WebsiteToMdFunction;

#[async_trait]
impl AIFunction for WebsiteToMdFunction {
    fn name(&self) -> &'static str {
        "website_to_md"
    }

    fn description(&self) -> &'static str {
        "Convert a website to markdown format using Firecrawl"
    }

    fn parameters(&self) -> HashMap<String, AIFunctionParameter> {
        let mut params = HashMap::new();
        params.insert(
            "url".to_string(),
            AIFunctionParameter::new(
                "string",
                "The URL of the website to convert to markdown",
                true,
            ),
        );
        params
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<Value, Error> {
        let args: WebsiteToMdArgs =
            serde_json::from_value(serde_json::Value::Object(args.into_iter().collect()))?;

        let config = EnvConfig::from_env();
        let client = firecrawl::FirecrawlApp::new(&config.firecrawl_key).map_err(|e| {
            Error::from(format!("Failed to create Firecrawl client: {}", e))
        })?;

        let options = firecrawl::scrape::ScrapeOptions {
            formats: Some(vec![firecrawl::scrape::ScrapeFormats::Markdown]),
            ..Default::default()
        };

        let scrape_result = client
            .scrape_url(&args.url, options)
            .await
            .map_err(|e| Error::from(format!("Failed to scrape URL: {}", e)))?;

        let markdown = scrape_result.markdown.unwrap_or_else(|| "No content found".to_string());
        let title = None; // Metadata access might require different approach

        let response = WebsiteToMdResponse {
            url: args.url,
            markdown,
            title,
        };

        Ok(json!(response))
    }
}
