use async_openai::{config::OpenAIConfig, types::{CategoryScore, ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent, ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent, ChatCompletionTool, ChatCompletionToolType, CreateChatCompletionRequest, CreateModerationRequestArgs, FunctionObject, ResponseFormat, ResponseFormatJsonSchema}, Client};
use std::{sync::Arc, collections::HashMap};
use crate::config::EnvConfig;
use crate::ai_functions::{get_all_functions, AIFunction};
use tokio::sync::Semaphore;
use serde_json::Value;
use crate::Error;

pub struct OpenAIService {
    pub(crate) client: Arc<Client<OpenAIConfig>>,
    semaphore: Arc<Semaphore>,
    functions: HashMap<String, Box<dyn AIFunction>>,
}

impl Clone for OpenAIService {
    fn clone(&self) -> Self {
        // Reinitialize functions since they can't be cloned
        let mut functions: HashMap<String, Box<dyn AIFunction>> = HashMap::new();
        for function in get_all_functions() {
            functions.insert(function.name().to_string(), function);
        }

        Self {
            client: Arc::clone(&self.client),
            semaphore: Arc::clone(&self.semaphore),
            functions,
        }
    }
}

impl OpenAIService {
    pub async fn new() -> Self {
        let config = EnvConfig::from_env();
        let openai_config = OpenAIConfig::new().with_api_key(config.keys.oai);
        let client = Client::with_config(openai_config);

        let semaphore = Arc::new(Semaphore::new(15));

        // Initialize available functions
        let mut functions: HashMap<String, Box<dyn AIFunction>> = HashMap::new();
        for function in get_all_functions() {
            functions.insert(function.name().to_string(), function);
        }

        Self { 
            client: Arc::new(client), 
            semaphore,
            functions,
        }
    }

    #[allow(dead_code)]
    pub async fn get_moderation_response(&self, input: &str) -> Result<CategoryScore, Error> {
        let permit = self.semaphore.acquire().await?;

        let request = CreateModerationRequestArgs::default()
            .input(input)
            .model("omni-moderation-latest")
            .build()?;

        let response = self.client.moderations().create(request).await?;

        drop(permit);
        Ok(response.results[0].category_scores.clone())
    }

    pub async fn get_completion_response(
        &self,
        model: &str,
        messages: Vec<ChatCompletionRequestMessage>,
        json_schema: Option<Value>,
    ) -> Result<String, Error> {
        // map None -> None, Some(schema) -> Some(ResponseFormat::JsonSchema{...})
        let response_format = json_schema.map(|schema| ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                name: "root".into(),
                description: None,
                schema: Some(schema),
                strict: None,
            },
        });

        let req = CreateChatCompletionRequest {
            model: model.to_string(),
            messages,
            response_format,
            ..Default::default()
        };

        let resp = self.client.chat().create(req).await?;
        Ok(resp
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default())
    }
    

    pub fn get_function_tools(&self) -> Vec<ChatCompletionTool> {
        self.functions
            .values()
            .map(|func| {
                let mut properties = serde_json::Map::new();
                let mut required = Vec::new();

                for (param_name, param) in func.parameters() {
                    properties.insert(param_name.clone(), serde_json::json!({
                        "type": param.param_type,
                        "description": param.description
                    }));

                    if param.required {
                        required.push(param_name);
                    }
                }

                ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
                    function: FunctionObject {
                        name: func.name().to_string(),
                        description: Some(func.description().to_string()),
                        parameters: Some(serde_json::json!({
                            "type": "object",
                            "properties": properties,
                            "required": required
                        })),
                        strict: None,
                    }
                }
            })
            .collect()
    }

    pub async fn process_openai_interactive(
        &self,
        _conversation_id: uuid::Uuid, // Remnants from DB integrations
        prompt: &str,
    ) -> Result<String, Error> {
        let chat = self.client.chat();
        let tools = self.get_function_tools();
        let sys = r#"
            Cut, to the point, and concise. Do not repeat yourself.
        "#;

        let mut messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(sys.to_string()),
                name: None,
            }),
        ];

        messages.push(ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
            content: Some(ChatCompletionRequestAssistantMessageContent::Text(prompt.to_string())),
            name: None,
            tool_calls: None,
            audio: None,
            refusal: None,
            ..Default::default()
        }));

        let mut response = chat.create(CreateChatCompletionRequest {
            model: "gpt-4o".to_owned(),
            messages: messages.clone(),
            tools: Some(tools),
            ..Default::default()
        }).await?;

        // Handle function calls
        while let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                // Add the assistant's message with function calls
                messages.push(ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        content: choice.message.content.clone().map(|c| ChatCompletionRequestAssistantMessageContent::Text(c)),
                        name: None,
                        tool_calls: Some(tool_calls.clone()),
                        audio: None,
                        refusal: None,
                        ..Default::default()
                    }
                ));

                // Execute each function call
                for tool_call in tool_calls {
                    let function = &tool_call.function;
                    let function_name = &function.name;
                    let arguments: HashMap<String, Value> = serde_json::from_str(&function.arguments)
                        .unwrap_or_default();

                    let result = match self.functions.get(function_name) {
                        Some(func) => {
                            match func.execute(arguments).await {
                                Ok(result) => serde_json::to_string(&result)?,
                                Err(e) => format!("Error: {}", e),
                            }
                        }
                        None => format!("Function '{}' not found", function_name),
                    };

                    // Add function result message
                    messages.push(ChatCompletionRequestMessage::Tool(
                        ChatCompletionRequestToolMessage {
                            content: ChatCompletionRequestToolMessageContent::Text(result),
                            tool_call_id: tool_call.id.clone(),
                        }
                    ));
                }

                // Get the next response
                response = chat.create(CreateChatCompletionRequest {
                    model: "gpt-4.1".to_owned(),
                    messages: messages.clone(),
                    tools: Some(self.get_function_tools()),
                    ..Default::default()
                }).await?;
            } else {
                break;
            }
        }

        // Function calls are done so get the final response.
        
        let response_content = response.choices[0].message.content.clone().unwrap_or_else(|| "LLM failed to respond.".to_string());
        
        Ok(response_content)
    }

    // ^ will integrate hashmap 15 message limit for future.
}