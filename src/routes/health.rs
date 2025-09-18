use crate::modules::openai::OpenAIService;
use crate::require_api_key;
use crate::response;
use actix_web::{get, web, HttpResponse};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
};
use schemars::{schema_for, JsonSchema};

#[get("")]
async fn health(req: actix_web::HttpRequest, data: web::Data<OpenAIService>) -> HttpResponse {
    require_api_key!(&req);

    #[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
    struct HealthSchema {
        status: String,
        code: u64,
        message: String,
    }

    let d = data.get_completion_response(
    "gpt-4o",
    vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(r#"
                You are a fake health check endpoint. You make fake data. You respond with strictly json data only.
                "#.to_string()),
                name: None,
            }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text("What is the status of the server?".to_string()),
                name: None,
            })
    ],
    Some(
        serde_json::to_value(schema_for!(HealthSchema)).unwrap()
    )
    ).await;

    println!("{:?}", d);

    HttpResponse::Ok().json(response::make_query_response(
        true,
        Some(&"Endpoints are healthy!"),
        None,
        Some("Server is healthy!"),
    ))
}
