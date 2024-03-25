use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, RequestBuilder, Response};
use tokio::time::sleep;

lazy_static! {
    static ref HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        let header_map = HashMap::from([
            ("User-Agent", "Roblox/WinInet"),
            ("Connection", "keep-alive"),
            ("Upgrade-Insecure-Requests", "1"),
            ("Cache-Control", "max-age=0"),
            ("Origin", "https://roblox.com"),
            ("Content-Type", "application/json")
        ]);

        for (key, value) in header_map {
            headers.insert(key, HeaderValue::from_str(value).unwrap());
        }

        headers
    };

    pub static ref CLIENT: Client = Client::new();
}

pub(crate) async fn send_request(request_builder: RequestBuilder) -> Result<Response, Error> {
    request_builder.headers((&*HEADERS).clone() as HeaderMap).send().await
}

#[allow(dead_code)]
pub(crate) fn send_no_fail_request(request_builder: RequestBuilder) -> Pin<Box<dyn Future<Output=Response>>> {
    Box::pin(async move {
        let response_result = send_request(request_builder.try_clone().unwrap()).await;
        async fn failed(request_builder: RequestBuilder) -> Response {
            sleep(Duration::from_millis(100)).await;
            return send_no_fail_request(request_builder).await;
        }

        match response_result {
            Ok(response) => {
                if response.status() != 200 {
                    return failed(request_builder).await;
                }

                response
            },
            Err(_) => {
                return failed(request_builder).await;
            }
        }
    })
}
