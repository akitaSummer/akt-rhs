use std::{env, fs};

use crate::config::{MimeType, GLOBAL_MIME_CFG, GLOBAL_STATUSES, OCTECT_STREAM};
use crate::http::{
    request::{HttpRequest, Resource},
    response::HttpResponse,
};
use crate::router::STATIC_RES;

pub trait HttpHandler {
    fn handle(&self, request: &HttpRequest) -> HttpResponse;
}

#[derive(Default)]
pub struct StaticResHandler {}

impl HttpHandler for StaticResHandler {
    fn handle(&self, request: &HttpRequest) -> HttpResponse {
        let mut resp = HttpResponse::default();
        let Resource::Path(ref path) = request.resource;
        let real_path = &path[STATIC_RES.len()..];
        let mut runtime_dir = env::current_dir().unwrap();
        runtime_dir.push("public");
        real_path
            .split("/")
            .into_iter()
            .for_each(|s| runtime_dir.push(s));
        let res_content = fs::read_to_string(runtime_dir.to_str().unwrap());
        match res_content {
            Ok(content) => {
                resp.resp_body = Some(content);
            }
            _ => {
                let mut path_buf = env::current_dir().unwrap();
                path_buf.push("public/404.html");
                let not_found_page_path = path_buf.to_str().unwrap();
                resp.resp_body = Some(fs::read_to_string(not_found_page_path).unwrap());
                resp.add_header("Content-Type".into(), "text/html".into());
                let statuses = GLOBAL_STATUSES.get().unwrap();
                let status = statuses.get("404").unwrap();
                resp.set_status(status.clone());
                return resp;
            }
        }

        let content_type = match path.split("/").last() {
            Some(res_name) => match res_name.split(".").last() {
                Some(ext) => GLOBAL_MIME_CFG.get().map(|entries| {
                    if let Some(tp) = entries.get(ext) {
                        tp.clone()
                    } else {
                        OCTECT_STREAM.into()
                    }
                }),
                None => Some(OCTECT_STREAM.into()),
            }
            .unwrap(),
            _ => OCTECT_STREAM.into(),
        };

        resp.add_header("Content-Type".into(), content_type);

        resp
    }
}

#[derive(Default)]
pub struct ApiHandler {}

impl HttpHandler for ApiHandler {
    fn handle(&self, request: &HttpRequest) -> HttpResponse {
        todo!("to implement")
    }
}
