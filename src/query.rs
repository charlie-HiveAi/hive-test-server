use serde::Deserialize;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PORT: u16 = std::env::var("PORT").unwrap_or_default().parse::<u16>().unwrap_or(4000);
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ImageServerQuery {
    pub mime: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub last_modified: Option<i64>,
    pub redirect: Option<u32>,
    pub status: Option<u16>
}

pub const CACHED: i64 = 1;
pub const NOT_CACHED: i64 = 0;

impl ImageServerQuery {
    pub fn new(mime: &str, width: u32, height: u32, last_modified: i64, redirect: u32, status: u16) -> Self {
        Self {
            mime: Some(mime.to_string()),
            width: Some(width),
            height: Some(height),
            last_modified: Some(last_modified),
            redirect: Some(redirect),
            status: Some(status)
        }
    }

    fn into_query_string(&self) -> String {
        let mut parameters = vec![];
        if self.mime.is_some() {
            let param = format!("mime={}", self.mime.as_ref().unwrap().to_string());
            parameters.push(param);
        }
        if self.width.is_some() {
            let param = format!("width={}", self.width.as_ref().unwrap().to_string());
            parameters.push(param);
        }
        if self.height.is_some() {
            let param = format!("height={}", self.height.as_ref().unwrap().to_string());
            parameters.push(param);
        }
        if self.last_modified.is_some() {
            let param = format!("last_modified={}", self.last_modified.as_ref().unwrap().to_string());
            parameters.push(param);
        }
        if self.redirect.is_some() {
            let param = format!("redirect={}", self.redirect.as_ref().unwrap().to_string());
            parameters.push(param);
        }
        if self.status.is_some() {
            let param = format!("status={}", self.status.as_ref().unwrap().to_string());
            parameters.push(param);
        }
        if parameters.len() == 0 {
            String::new()
        } else {
            let mut query_string = "?".to_string();
            query_string.push_str(parameters.join("&").as_str());
            query_string
        }
    }
}

pub fn make_test_server_url(query: &ImageServerQuery) -> String {
    let port = std::env::var("PORT").unwrap_or_default().parse::<u16>().unwrap_or(4000);
    format!(
        "http://127.0.0.1:{}/api/image{}",
        port,
        query.into_query_string()
    )
}

pub fn make_redirect_test_server_url(query: &ImageServerQuery) -> String {
    let mut new_query = query.clone();
    new_query.redirect = Some(*query.redirect.as_ref().unwrap() - 1);
    make_test_server_url(&new_query)
}