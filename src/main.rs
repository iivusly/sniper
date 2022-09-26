
#![allow(non_snake_case)]

use reqwest::{Client, RequestBuilder, Response};
use serde::{Serialize, Deserialize};
use std::{env};

struct Settings {
    target: u64,
    place: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    id: String,
    maxPlayers: u64,
    playing: u64,
    playerTokens: Vec<String>,
    // players: IMPLEMENT,
    fps: f32,
    ping: u64
}

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
struct BatchResponse {
    requestId: String,
    errorCode: u64,
    errorMessage: String,
    targetId: u64,
    state: String,
    imageUrl: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Batch {
    data: Vec<BatchResponse>
}

#[derive(Debug, Serialize, Deserialize)]
struct Page {
    previousPageCursor: Option<String>,
    nextPageCursor: Option<String>,
    data: Vec<Game>
}

async fn req(req_builder: RequestBuilder) -> Response {
    req_builder
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Cache-Control", "max-age=0")
        .header("TE", "Trailers")
        .header("Content-Type", "application/json")
        .send().await.unwrap()
}

/*async fn parse<'a, T: serde::de::Deserialize<'a>>(resp: Response<Body>) -> T {
    let vec = body::to_bytes(resp.into_body()).await.unwrap().to_vec();
    let slice: &'a [u8] = vec.as_slice();

    serde_json::from_slice::<T>(slice).unwrap()
}*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();

    let settings = Settings {
        target: env::var("TARGET").unwrap().parse().unwrap(),
        place: env::var("PLACE").unwrap().parse().unwrap(),
    };

    let client = Client::new();

    let avatar_req_url = format!(
        "https://www.roblox.com/headshot-thumbnail/image?userId={}&width=48&height=48&format=png",
        settings.target
    );

    let resp = req(client.get(avatar_req_url)).await;
    let parsed = url::Url::parse(resp.url().as_str()).unwrap();
    let target_imageid = parsed.path().split("/").collect::<Vec<&str>>()[1];

    println!("ImageID: {}. Finding player...", target_imageid);

    let mut next_page: String = "".to_string();

    let mut found_gameid: Option<String> = None;

    loop {
        let url = format!("https://games.roblox.com/v1/games/{}/servers/Public?cursor={}&sortOrder=Desc&excludeFullGames=false", settings.place, next_page);
        let resp = req(client.get(url)).await;
        let page = resp.json::<Page>().await?;

        for game in page.data {
            let mut batch_req: Vec<BatchRequest> = vec![];
            for player_token in game.playerTokens {
                batch_req.push(BatchRequest{ requestId: format!("0:{}:AvatarHeadshot:48x48:png:regular", player_token), r#type: "AvatarHeadShot".to_string(), targetId: 0, token: player_token, format: "png".to_string(), size: "48x48".to_string() });
            }

            let data = serde_json::to_string_pretty(&batch_req).unwrap();

            let request = req(client.post("https://thumbnails.roblox.com/v1/batch").body(data)).await;
            let req = request.json::<Batch>().await.unwrap();

            for data in req.data {
                if data.state != "Completed" {
                    println!("{:?}", data);
                }

                let parsed = url::Url::parse(data.imageUrl.as_str()).unwrap();
                let imageid =  parsed.path().split("/").collect::<Vec<&str>>()[1];

                if imageid == target_imageid {
                    found_gameid = Some(game.id.clone());
                    break;
                }
            }

            if found_gameid.is_some() {
                break;
            }
        }

        if page.nextPageCursor.is_none() || found_gameid.is_some() {
            break;
        }

        next_page = page.nextPageCursor.unwrap();
        
    }

    if found_gameid.is_some() {
        println!("Game found! GameID: {}", found_gameid.unwrap());
    } else {
        println!("User not found :(");
    }

    Ok(())
}
