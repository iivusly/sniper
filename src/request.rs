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

pub async fn send_request(request_builder: RequestBuilder) -> Response {
    request_builder.headers((&*HEADERS).clone()).send().await.unwrap()
}

pub fn send_no_fail_request(request_builder: RequestBuilder) -> Pin<Box<dyn Future<Output=Response>>> {
    Box::pin(async move {
        let response = send_request(request_builder.try_clone().unwrap()).await;
        if response.status() != 200 {
            sleep(Duration::from_millis(100)).await;
            return send_no_fail_request(request_builder).await;
        }
        response
    })
}
