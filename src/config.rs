pub struct ClientConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

impl ClientConfig {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self {
            base_url,
            username,
            password,
        }
    }
}
