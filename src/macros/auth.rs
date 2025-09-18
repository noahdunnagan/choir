#[macro_export]
macro_rules! require_api_key {
    ($req:expr) => {
        if let Some(resp) = crate::utils::webutils::WebUtils::require_api_key($req) {
            return resp;
        }
    };
}
