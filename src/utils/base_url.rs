pub struct BaseUrl;
impl BaseUrl{
    pub fn get_base_url() -> String {
        option_env!("BASE_URL")
            .map(|res| res.to_string())
            .unwrap_or_else(|| {
                if cfg!(debug_assertions) {
                    "http://localhost:8080".to_string()
                } else {
                    "".to_string()
                }
            })
    }
}
