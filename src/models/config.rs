#[derive(Clone)]
/// Basic configuration shared across handlers.
pub struct ServerConfig {
    pub secret: String,
    pub domain: String,
}
