use dotenv;
use std::env;

pub struct EnvConfig {
    pub port: i32,
    pub api_key: String,
    pub oai_key: String,
    pub firecrawl_key: String
}

impl EnvConfig {
    // Get from env
    fn get_env(key: &str) -> String {
        env::var(key).unwrap_or_else(|_| panic!("Environment variable {} not set", key))
    }

    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let port: i32 = Self::get_env("PORT").parse().unwrap_or(8081);
        let api_key = Self::get_env("API_KEY");
        let oai_key = Self::get_env("OAI_KEY");
        let firecrawl_key = Self::get_env("FC_KEY");

        EnvConfig {
            port,
            api_key,
            oai_key,
            firecrawl_key
        }
    }
}
