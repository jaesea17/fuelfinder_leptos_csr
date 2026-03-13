pub struct BaseUrl;
impl BaseUrl{
    pub fn get_base_url() -> String {
        option_env!("BASE_URL")
            .map(|res| res.to_string())
            .unwrap_or_else(|| "".to_string())
    }
}
