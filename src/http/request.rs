use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpStream;

pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    UNKNOWN,
}

impl From<&str> for Method {
    fn from(method: &str) -> Self {
        let lowercase_method = method.to_lowercase();
        let res = lowercase_method.as_str();
        match res {
            "get" => Method::GET,
            "post" => Method::POST,
            "put" => Method::PUT,
            "delete" => Method::DELETE,
            _ => Method::UNKNOWN,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Version {
    HTTP1_1,
    HTTP2_0,
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Resource {
    Path(String),
}

impl From<&str> for Version {
    fn from(ver: &str) -> Self {
        let ver_in_lowercase = ver.to_lowercase();
        match ver_in_lowercase.as_str() {
            "http/1.1" => Version::HTTP1_1,
            "http/2.0" => Version::HTTP2_0,
            _ => Version::UNKNOWN,
        }
    }
}

pub struct HttpRequest {
    pub method: Method,
    pub version: Version,
    pub resource: Resource,
    pub headers: Option<HashMap<String, String>>,
    pub msg_body: Option<String>,
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self {
            method: Method::GET,
            version: Version::HTTP1_1,
            resource: Resource::Path(String::from("/")),
            headers: None,
            msg_body: None,
        }
    }
}

impl HttpRequest {
    pub async fn from(stream: &mut TcpStream) -> Self {
        let mut reader = BufReader::<&mut TcpStream>::new(stream);
        let mut request = HttpRequest::default();
        let mut headers = HashMap::<String, String>::new();
        let mut content_len = 0;
        let mut is_req_line = true;
        // 读取header
        loop {
            let mut line = String::from("");
            reader.read_line(&mut line).await.unwrap();
            // 第一行
            if is_req_line {
                if line.is_empty() && is_req_line {
                    return HttpRequest::default();
                }
                // 解析请求信息
                let (method, resource, version) = process_req_line(line.as_str());
                request.method = method;
                request.resource = resource;
                request.version = version;
                is_req_line = false;
            } else if line.contains(":") {
                // 解析header
                let (key, value) = process_request_header(line.as_str());
                headers.insert(key.clone(), value.clone().trim().to_string());
                if key == "Content-Length" {
                    content_len = value.trim().parse::<usize>().unwrap();
                }
            } else if line == String::from("\r\n") {
                // header 与 body之间存在空行，header结束
                break;
            }
        }
        request.headers = Some(headers);
        if content_len > 0 {
            let mut buf = vec![0 as u8; content_len];
            let buf_slice = buf.as_mut_slice();
            // 读取请求体，注意，这里不能在使用stream进行读取，否则会一直卡在这里，要继续用reader进行读取.
            // BufReader::read(&mut reader, buf_slice).await.unwrap();
            reader.read(buf_slice).await.unwrap();
            request.msg_body = Some(String::from_utf8_lossy(buf_slice).to_string());
        }
        request
    }
}

fn process_request_header(line: &str) -> (String, String) {
    let mut seg_iter = line.split(":");
    (
        seg_iter.next().unwrap().into(),
        seg_iter.next().unwrap().into(),
    )
}

fn process_req_line(line: &str) -> (Method, Resource, Version) {
    let mut segments = line.split_whitespace();
    (
        segments.next().unwrap().into(),
        Resource::Path(segments.next().unwrap().to_string()),
        segments.next().unwrap().into(),
    )
}
