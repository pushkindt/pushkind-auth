use pushkind_common::models::config::CommonServerConfig;

#[derive(Clone)]
/// Basic configuration shared across handlers.
pub struct ServerConfig {
    pub domain: String,
    pub common_config: CommonServerConfig,
}
