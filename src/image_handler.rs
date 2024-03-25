use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::request::{CLIENT, send_no_fail_request, send_request};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchRequest {
    request_id: String,
    #[serde(rename = "type")]
    type_field: String,
    target_id: u64,
    token: String,
    format: String,
    size: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResponse {
    pub request_id: String,
    pub error_code: u64,
    pub error_message: String,
    pub target_id: u64,
    pub state: String,
    pub image_url: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Batch {
    pub data: Vec<BatchResponse>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Thumbnail {
    target_id: u64,
    state: String,
    image_url: String,
    version: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ThumbnailBatch {
    data: Vec<Thumbnail>
}

fn parse_token_from_url(url: String) -> String {
    let parsed_url = Url::parse(url.as_str()).unwrap();
    let image_id = parsed_url.path().split("/").collect::<Vec<&str>>()[1].to_string();
    let token = image_id.split("-").collect::<Vec<&str>>()[2].to_string();
    token
}

pub fn find_from_player_tokens(player_tokens: Vec<String>, target_token: String) -> Pin<Box<dyn Future<Output = Option<BatchResponse>>>> {
pub fn find_from_player_tokens<'a>(player_tokens: &'a Vec<String>, target_token: &'a String, retry_level: usize) -> Pin<Box<dyn Future<Output = Option<BatchResponse>> + 'a>> {
    Box::pin(async move {
        let batch_request: Vec<BatchRequest> = player_tokens.iter().map(|token| BatchRequest {
            request_id: format!("0:{}:AvatarHeadshot:150x150:png:regular", token), 
            type_field: "AvatarHeadShot".to_string(),
            target_id: 0,
            token: token.to_string(),
            format: "png".to_string(),
            size: "150x150".to_string(),
        }).collect();

        let request = send_no_fail_request(
            CLIENT.post("https://thumbnails.roblox.com/v1/batch").json(&batch_request)
        ).await;

        if request.status() == 429 {

        }

        let batch = request.json::<Batch>().await.unwrap();

        let mut bad_tokens: Vec<String> = vec![];

        for response in batch.data {
            match response.state.as_str() {
                "Completed" => {
                    let token = parse_token_from_url(response.image_url.clone());
                    if token == target_token.clone() {
                        return Some(response);
                    }
                },
                "Pending" => pending_tokens.push(response.request_id.split(":").collect::<Vec<&str>>()[1].to_string()),
                _ => continue
            }
        }

        if !pending_tokens.is_empty() && retry_level < 3 { // TODO: make retry less
            // sleep(Duration::from_secs(10)).await;
            return find_from_player_tokens(&pending_tokens, target_token, retry_level + 1).await;
        }

        None
    })
}


pub async fn get_player_image_token(user_id: u64) -> String {
    let url = format!(
        "https://thumbnails.roblox.com/v1/users/avatar-bust?userIds={}&size=48x48&format=Png&isCircular=false",
        user_id
    );

    let response = send_request(CLIENT.get(url)).await;
    let thumbnails = response.json::<ThumbnailBatch>().await.unwrap();
    let token = parse_token_from_url((&thumbnails.data[0].image_url).to_string());

    token
}
