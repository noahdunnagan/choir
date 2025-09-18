use crate::ai_functions::{get_all_functions, AIFunction};
use crate::modules::openai::OpenAIService;
use crate::types::tchoir::{get_choir_agent_response_schema, ChoirAgentResponse, ChoirRequest};
use crate::Error;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
};
use log::{error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ChoirService {
    openai_service: Arc<OpenAIService>,
    ai_functions: Vec<Box<dyn AIFunction>>,
}

impl ChoirService {
    pub fn new(openai_service: Arc<OpenAIService>) -> Self {
        Self {
            openai_service,
            ai_functions: get_all_functions(),
        }
    }

    pub async fn run_choir(&self, request: &ChoirRequest) -> Result<String, Error> {
        let model = "gpt-4o";

        info!("Gathering initial data with AI functions.");
        let enriched_query = self.enrich_query_with_functions(&request.query).await?;
        info!("Data gathering complete.");

        info!("Getting a plan of action.");
        let task_master_response = self.get_task_master_response(model, &enriched_query).await?;
        info!("Plan of action received.");

        info!("Delegating to agents.");
        let agents = self.run_agents(model, &task_master_response).await?;
        info!("Agents finished.");

        for agent in agents.iter() {
            info!("Thoughts: {}", agent.thoughts);
        }

        info!("Getting assessment from chorus master.");
        let assessment = self
            .get_assessment(&model, &agents, &request.json_schema)
            .await?;
        info!("Assessment: {:?}", assessment);

        let final_res = self.openai_service.get_completion_response(
            model,
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(
                        r#"
                        You are the final summary agent. Your job is to provide the user with a direct, accurate answer to their question.
                        You have access to webpage content and analysis from multiple expert agents.
                        Be specific and factual. If you can answer the user's question directly, do so.
                        Do not say "the agents didn't find" unless you're absolutely certain the information isn't in the data provided.
                        "#.to_string(),
                    ),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(
                        format!("User's original query: {}\n\nExpert analysis: {}\n\nDetailed agent responses: {:#?}",
                            request.query,
                            assessment,
                            agents.iter().map(|agent| agent.detailed_response.clone()).collect::<Vec<_>>()
                        ),
                    ),
                    name: None,
                }),
            ],
            None
        ).await?;


        Ok(final_res)
    }

    async fn get_task_master_response(&self, model: &str, query: &str) -> Result<String, Error> {
        self.openai_service.get_completion_response(
            model,
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(r#"
                    You are the task master coordinating 5 expert agents.
                    Given the user's query and any data we've gathered, create 5 distinct analytical approaches:
                    1. Direct analysis approach
                    2. Critical evaluation approach
                    3. Contextual analysis approach
                    4. Creative interpretation approach
                    5. Comprehensive synthesis approach

                    Be specific about what each agent should focus on. Each approach must be unique.
                    "#.to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(query.to_string()),
                    name: None,
                })
            ],
            None,
        ).await
    }

    async fn run_agents(
        &self,
        model: &str,
        task_master_response: &str,
    ) -> Result<Vec<ChoirAgentResponse>, Error> {
        let agents_prompts = [
            r#"You are Agent 1: Direct Analysis Expert. Focus on the first assigned approach.
            You MUST respond with a valid JSON object containing exactly these three fields:
            - "detailed_response": Your comprehensive analysis (multiple paragraphs)
            - "short_overview": Brief 2-3 sentence summary
            - "thoughts": Your analytical thoughts and reasoning
            Be precise and methodical in your analysis."#,
            r#"You are Agent 2: Critical Evaluator. Focus on the second assigned approach.
            You MUST respond with a valid JSON object containing exactly these three fields:
            - "detailed_response": Your comprehensive evaluation (multiple paragraphs)
            - "short_overview": Brief 2-3 sentence summary
            - "thoughts": Your critical thoughts and concerns
            Question assumptions and identify potential issues."#,
            r#"You are Agent 3: Context Specialist. Focus on the third assigned approach.
            You MUST respond with a valid JSON object containing exactly these three fields:
            - "detailed_response": Your contextual analysis (multiple paragraphs)
            - "short_overview": Brief 2-3 sentence summary
            - "thoughts": Your thoughts on context and connections
            Consider broader context, connections, and underlying patterns."#,
            r#"You are Agent 4: Creative Interpreter. Focus on the fourth assigned approach.
            You MUST respond with a valid JSON object containing exactly these three fields:
            - "detailed_response": Your creative interpretation (multiple paragraphs)
            - "short_overview": Brief 2-3 sentence summary
            - "thoughts": Your innovative thoughts and perspectives
            Think creatively while staying grounded in facts."#,
            r#"You are Agent 5: Synthesis Expert. Focus on the fifth assigned approach.
            You MUST respond with a valid JSON object containing exactly these three fields:
            - "detailed_response": Your comprehensive synthesis (multiple paragraphs)
            - "short_overview": Brief 2-3 sentence summary
            - "thoughts": Your integrative thoughts and conclusions
            Integrate different viewpoints and provide comprehensive analysis."#,
        ];

        let agent_futures = agents_prompts.iter().map(|prompt| {
            self.openai_service.get_completion_response(
                model,
                vec![
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(
                            prompt.to_string(),
                        ),
                        name: None,
                    }),
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(
                            task_master_response.to_string(),
                        ),
                        name: None,
                    }),
                ],
                Some(get_choir_agent_response_schema()),
            )
        });

        let results = futures::future::join_all(agent_futures).await;

        let mut successful_agents = Vec::new();

        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(json) => {
                    match serde_json::from_str::<ChoirAgentResponse>(&json) {
                        Ok(agent_response) => successful_agents.push(agent_response),
                        Err(e) => {
                            error!("Agent {} failed to parse JSON: {}", i + 1, e);
                            error!("Agent {} raw response: {}", i + 1, json);
                            successful_agents.push(ChoirAgentResponse::empty());
                        }
                    }
                }
                Err(e) => {
                    error!("Agent {} failed to respond: {}", i + 1, e);
                    successful_agents.push(ChoirAgentResponse::empty());
                }
            }
        }

        Ok(successful_agents)
    }

    async fn get_assessment(
        &self,
        model: &str,
        agents: &[ChoirAgentResponse],
        json_schema: &Option<serde_json::Value>,
    ) -> Result<String, Error> {
        self.openai_service.get_completion_response(
            model,
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(r#"
                    You are chorus.
                    There have been 5 distinctly unique sub agents. Each of these agents have been given a task to solve.
                    You are to assess the results of each agent and determine the best course of action.
                    Your personal goal is to think VERY hard and weigh the general pros and cons of each approach. Finally, decide on a final course of action with the best result.
                    Your result may be anything. You are only allowed to go off of information provided by the sub agents.
                    Do not repeat yourself.
                    It can be a combination of approaches. It can be a single approach.
                    Think hard. Think VERY hard.
                    Your response should look as follows:
                    IN MARKDOWN FORMAT!
                    A brief overview of all results. Generally just to make sure its known.
                    The actual answer, the absolute best answer that you decided on. Keep this to the point and in a format the user would understand.
                    More info.
                    Post game thoughts on each agents approach.
                    "#.to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(format!(
                        "Agent 1: {}\nAgent 2: {}\nAgent 3: {}\nAgent 4: {}\nAgent 5: {}",
                        &agents[0].detailed_response,
                        &agents[1].detailed_response,
                        &agents[2].detailed_response,
                        &agents[3].detailed_response,
                        &agents[4].detailed_response
                    )),
                    name: None,
                })
            ],
            json_schema.clone(),
        ).await
    }

    async fn enrich_query_with_functions(&self, query: &str) -> Result<String, Error> {
        // Check if query contains URLs
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        let urls: Vec<&str> = url_regex.find_iter(query).map(|m| m.as_str()).collect();

        if urls.is_empty() {
            return Ok(query.to_string());
        }

        info!("Found {} URLs in query, fetching content...", urls.len());
        let mut enriched_content = query.to_string();

        for url in urls {
            if let Some(website_function) = self.ai_functions.iter().find(|f| f.name() == "website_to_md") {
                let mut args = HashMap::new();
                args.insert("url".to_string(), Value::String(url.to_string()));

                match website_function.execute(args).await {
                    Ok(result) => {
                        if let Some(markdown) = result.get("markdown") {
                            if let Some(markdown_str) = markdown.as_str() {
                                enriched_content.push_str(&format!(
                                    "\n\n--- Content from {} ---\n{}",
                                    url, markdown_str
                                ));
                                info!("Successfully fetched content from {}", url);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch content from {}: {}", url, e);
                        // Dont exit so we can continue with other URLs even if one fails
                    }
                }
            }
        }

        Ok(enriched_content)
    }
}
