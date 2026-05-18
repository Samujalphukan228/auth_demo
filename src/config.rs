use ezconfig_rs::Config;
use serde::Deserialize;


#[derive(Config, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,
    pub brevo_api_key: String,
    pub brevo_sender_email: String,
    pub brevo_sender_name: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_app_url")]
    pub app_url: String,
}

fn default_port() -> u16 { 8080 }
fn default_app_url() -> String { "http://localhost:8080".to_string() }