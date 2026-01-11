
// pub const BASE_URL: &str = "https://close-siusan-jaesea-21c201ce.koyeb.app";

pub struct BaseUrl;
impl BaseUrl{
    const BASE_URL: &str = "https://close-siusan-jaesea-21c201ce.koyeb.app";
    // const BASE_URL: &str = "http://localhost:8000";
    pub fn get_base_url() -> String {
        option_env!("BASE_URL").map(|res| res.to_string()).unwrap_or_else(|| Self::BASE_URL.to_string())
    }
}
