use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::request::{CLIENT, send_no_fail_request, send_request};

#[derive(Debug, Serialize, Deserialize)]
struct BatchRequest {
    requestId: String,
    r#type: String,
    targetId: u64,
    token: String,
    format: String,
    size: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchResponse {
    pub requestId: String,
    pub errorCode: u64,
    pub errorMessage: String,
    pub targetId: u64,
    pub state: String,
    pub imageUrl: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Batch {
    data: Vec<BatchResponse>
}

#[derive(Debug, Serialize, Deserialize)]
struct Thumbnail {
    targetId: u64,
    state: String,
    imageUrl: String,
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
    Box::pin(async move {
        let batch_request: Vec<BatchRequest> = player_tokens.iter().map(|token| BatchRequest {
            requestId: format!("0:{}:AvatarHeadshot:150x150:png:regular", token),
            r#type: "AvatarHeadShot".to_string(),
            targetId: 0,
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
            if response.state != "Completed" {
                bad_tokens.push(response.requestId.split(":").collect::<Vec<&str>>()[1].to_string());
                continue;
            }

            let token = parse_token_from_url(response.imageUrl.clone());
            if token == target_token {
                return Some(response);
            }
        }

        if !bad_tokens.is_empty() {
            // sleep(Duration::from_secs(10)).await;
            // return find_from_player_tokens(bad_tokens, target_token).await;
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
    let token = parse_token_from_url((&thumbnails.data[0].imageUrl).to_string());

    token
}
