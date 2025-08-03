use actix_web::{post, HttpResponse, web};
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessageContent};
use log::{info, error};
use crate::require_api_key;
use crate::response;
use crate::config;
use crate::types::tchoir;
use crate::modules::openai::OpenAIService;
use tokio;
use crate::types::tchoir::{get_choir_agent_response_schema, ChoirAgentResponse};


const PERMISSION_LEVEL: config::PermissionLevel = config::PermissionLevel::Public;

#[post("")]
async fn choir(
    req: actix_web::HttpRequest,
    body: web::Json<tchoir::ChoirRequest>,
    data: web::Data<OpenAIService>
) -> HttpResponse {
    require_api_key!(&req, PERMISSION_LEVEL);
    
    // TODO: Sanitize user input. (I dont trust you.)
    let model = "gpt-4o";

    info!("Getting a plan of action.");
    // First get a plan of action.
    let res = data.get_completion_response(
        &model,
        vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(r#"
                You are task master. Clipped, to the point, extremely concise while also being clear about details.
                You are in charge of 5 distinctly unique sub agents.
                Each of these agents will be using all of their knowledge to solve a problem with you. You are to coordinate the approaches at solving the problem.
                Each agents name is just their number nothing else.
                You figure out the problem at hand and determine 5 distinctly unique directions and approaches for each agent to arrive at the optimal conclusion.
                You are not solving it, you are delegating the task to those sub agents.
                Your approaches must be distinctly unique. Do not repeat yourself.
                Make them clear which is which the more information the better. 
                Be clipped, to the point, extremely concise. Do not repeat yourself.
                "#.to_string()),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(body.query.clone()),
                name: None,
            })
        ],
        None,
    ).await;

    info!("Plan of action received.");
    let task_master_response = res.unwrap_or("Task Master failed to generate task.".to_string());

    info!("Delegating to agents.");
    // Next, each agent.
    let (agent1, agent2, agent3, agent4, agent5) = tokio::join!(
        data.get_completion_response(
            &model, 
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text("You are agent 1. Respond in strict JSON format. You approach tasks cautiously focusing on whats known and safe nothing risky. There are 5 tasks provided and you are to solve the first one ONLY with the first approach. Do not overlap with the other agent's task.".to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(task_master_response.clone()),
                    name: None,
                })
            ],
            Some(get_choir_agent_response_schema()),
        ),
        data.get_completion_response(
            &model, 
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text("You are agent 2. Respond in strict JSON format. You approach tasks with reckless determination. You take the risk for the chance at finding the best solution. There are 5 tasks provided and you are to solve the second one ONLY with the first approach. Do not overlap with the other agent's task.".to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(task_master_response.clone()),
                    name: None,
                })
            ],
            Some(get_choir_agent_response_schema()),
        ),
        data.get_completion_response(
            &model, 
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text("You are agent 3. Respond in strict JSON format. You approach tasks with a balance between caution and recklessness. Balance the both. There are 5 tasks provided and you are to solve the third one ONLY with the first approach. Do not overlap with the other agent's task.".to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(task_master_response.clone()),
                    name: None,
                })
            ],
            Some(get_choir_agent_response_schema()),
        ),
        data.get_completion_response(
            &model, 
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text("You are agent 4. Respond in strict JSON format. You approach tasks thinking outside of the box. You are not bound by the rules of the game and you believe you have a new way of doing things. Be enthusiastic and creative. There are 5 tasks provided and you are to solve the fourth one ONLY with the first approach. Do not overlap with the other agent's task.".to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(task_master_response.clone()),
                    name: None,
                })
            ],
            Some(get_choir_agent_response_schema()),
        ),
        data.get_completion_response(
            &model, 
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text("You are agent 5. Respond in strict JSON format. There are 5 tasks provided and you are to solve the fifth one ONLY with the first approach. Do not overlap with the other agent's task.".to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(task_master_response.clone()),
                    name: None,
                })
            ],
            Some(get_choir_agent_response_schema()),
        ),
    );

    // Parse the agent responses.
    let agents_response = [agent1, agent2, agent3, agent4, agent5];
    let agents: Vec<ChoirAgentResponse> = agents_response
        .into_iter()
        .enumerate()
        .map(|(i, res)| {
            match res {
                Ok(json) => serde_json::from_str::<ChoirAgentResponse>(&json)
                    .unwrap_or_else(|e| {
                        error!("Agent {} failed to parse JSON: {}", i + 1, e);
                        ChoirAgentResponse::empty()
                    }),
                Err(e) => {
                    error!("Agent {} failed to respond: {}", i + 1, e);
                    ChoirAgentResponse::empty()
                }
            }
        })
        .collect();


    info!("Getting assessment from chorus master.");
    // Finally, have an assessment agent.
    let assessment = data.get_completion_response(
        &model, 
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
        body.json_schema.clone(),
    ).await; 
    info!("Assessment: {:?}", assessment);

    // TODO: Parse the response and determine the best course of action.
    


    match assessment {
        Ok(r) => {
            info!("Assessment successful, returning response.");
            return HttpResponse::Ok().json(
                response::make_query_response(
                    true,
                    Some(&r),
                    None,
                    Some("Model returned a valid response!")
                )
            );
        },
        Err(e) => error!("{}", e), // idc rn
    }

    HttpResponse::Ok().json(
        response::make_query_response(
            true,
            Some(&"Endpoints are healthy!"),
            None,
            Some("Server is healthy!")
        )
    )
}