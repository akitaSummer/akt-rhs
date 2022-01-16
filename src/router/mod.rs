pub mod handler;

use crate::http::{
    request::{HttpRequest, Method, Resource},
    response::HttpResponse,
};
use crate::router::handler::{ApiHandler, HttpHandler, StaticResHandler};

const STATIC_RES: &str = "/staticres";

pub struct Router {}

impl Router {
    pub async fn route(request: &HttpRequest) -> HttpResponse {
        let mut resp = HttpResponse::default();
        match request.method {
            Method::GET => {
                let Resource::Path(path) = &request.resource;
                if path.starts_with(STATIC_RES) {
                    let handler_wrapper = get_request_handler(request);
                    if let Some(handler) = handler_wrapper {
                        resp = handler.handle(request);
                        return resp;
                    }
                }
                resp.set_body("this is api response.".into());
            }
            _ => {
                resp.set_body("hello world".into());
            }
        }
        resp
    }
}

const STATIC_RES_HANDLER: StaticResHandler = StaticResHandler {};
const API_RES_HANDLER: ApiHandler = ApiHandler {};

fn get_request_handler(req: &HttpRequest) -> Option<Box<&dyn HttpHandler>> {
    match req.method {
        Method::GET => Some(Box::new(&STATIC_RES_HANDLER)),
        _ => Some(Box::new(&API_RES_HANDLER)),
    }
}
