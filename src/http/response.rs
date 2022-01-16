use std::collections::HashMap;

use crate::config::{Status, GLOBAL_MIME_CFG};

#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: String,
    pub status_text: String,
    pub headers: Option<HashMap<String, String>>,
    pub resp_body: Option<String>,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            version: String::from("HTTP/1.1"),
            status_code: String::from("200"),
            status_text: "OK".to_string(),
            headers: Some(HashMap::new()),
            resp_body: None,
        }
    }
}

impl HttpResponse {
    pub fn set_status(&mut self, status: Status) {
        let Status(status_code, status_text) = status;
        self.status_code = status_code;
        self.status_text = status_text;
    }

    pub fn set_headers(&mut self, headers: HashMap<String, String>) {
        self.headers = Some(headers);
    }

    pub fn set_body(&mut self, body: String) {
        self.resp_body = Some(body);
    }

    pub fn add_header(&mut self, key: String, value: String) {
        let headers = self.headers.as_mut().unwrap();
        headers.insert(key, value);
    }

    pub fn new(
        version: String,
        status_code: String,
        headers: Option<HashMap<String, String>>,
        resp_body: Option<String>,
    ) -> Self {
        let mut response = HttpResponse::default();
        response.version = version.to_string();
        if status_code != "200" {
            response.status_code = status_code.to_string();
            let code = status_code;
            let mime_config = &GLOBAL_MIME_CFG.get();
            let mut status_text = String::from("");
            if let Some(config) = mime_config {
                let desc = match config.get(&*code.to_string()) {
                    Some(status_desc) => status_desc,
                    None => "Unknown Status",
                };
                status_text.push_str(desc);
            };
            response.status_text = status_text;
        }
        if let Some(_) = headers {
            response.headers = headers;
        }

        if let Some(_) = resp_body {
            response.resp_body = resp_body;
        }
        response
    }

    fn get_serialized_headers(&self) -> String {
        let mut result = String::from("");
        match &self.headers {
            Some(headers) => {
                let mut keys = headers.keys().collect::<Vec<&String>>();
                keys.sort();
                keys.iter()
                    .for_each(|&k| result = format!("{}{}: {}\r\n", result, k, headers[k]));
                let content_len = if let Some(body) = &self.resp_body {
                    body.len()
                } else {
                    0
                };
                result = format!("{}{}: {}\r\n", result, "Content-Length", content_len);
            }
            None => {}
        }
        result
    }

    fn get_serialized_body(&self) -> String {
        match &self.resp_body {
            Some(body) => body.to_string(),
            None => String::from(""),
        }
    }
}

impl Into<String> for HttpResponse {
    fn into(self) -> String {
        format!(
            "{} {} {}\r\n{}\r\n{}",
            &self.version,
            &self.status_code,
            &self.status_text,
            &self.get_serialized_headers(),
            &self.get_serialized_body(),
        )
    }
}
