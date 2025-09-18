use crate::config::EnvConfig;
use crate::response;
use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;
use urlencoding;

pub struct WebUtils;

impl WebUtils {
    // Extract the API key from the request headers

    pub fn extract_api_key(req: &HttpRequest) -> Option<String> {
        req.headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|value| {
                value
                    .strip_prefix("Bearer ")
                    .or_else(|| value.strip_prefix("Bearer%20"))
            })
            .map(|s| s.to_string())
    }

    // Check the api key against the configured api key.

    pub fn check_api_key(req: &HttpRequest) -> bool {
        let cfg = match req.app_data::<web::Data<Arc<EnvConfig>>>() {
            Some(cfg) => cfg.get_ref().clone(),
            None => return false,
        };

        let token = match Self::extract_api_key(req) {
            Some(t) => t,
            None => return false,
        };

        token == cfg.api_key
    }

    pub fn require_api_key(req: &HttpRequest) -> Option<HttpResponse> {
        if WebUtils::check_api_key(req) {
            None
        } else {
            Some(
                HttpResponse::Unauthorized().json(response::make_query_response::<()>(
                    false,
                    None,
                    Some("Unauthorized"),
                    None,
                )),
            )
        }
    }

    #[allow(dead_code)]
    pub fn decode_all(input: &str) -> Option<String> {
        urlencoding::decode(input).ok().map(|cow| cow.into_owned())
    }
}
