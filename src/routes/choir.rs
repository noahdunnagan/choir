use crate::modules::choir::ChoirService;
use crate::require_api_key;
use crate::response;
use crate::types::tchoir;
use actix_web::{post, web, HttpResponse};
use log::{error, info};

#[post("")]
async fn choir(
    req: actix_web::HttpRequest,
    body: web::Json<tchoir::ChoirRequest>,
    service: web::Data<ChoirService>,
) -> HttpResponse {
    require_api_key!(&req);

    match service.run_choir(&body).await {
        Ok(r) => {
            info!("Assessment successful, returning response.");
            HttpResponse::Ok().json(response::make_query_response(
                true,
                Some(&r),
                None,
                Some("Model returned a valid response!"),
            ))
        }
        Err(e) => {
            error!("Choir service failed: {}", e);
            HttpResponse::InternalServerError().json(response::make_query_response::<()>(
                false,
                None,
                Some("An internal error occurred."),
                None,
            ))
        }
    }
}
