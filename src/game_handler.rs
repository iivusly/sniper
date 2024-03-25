use serde::{Deserialize, Serialize};
use crate::request::{CLIENT, send_request};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub player_token: String,
    pub id: u64,
    pub name: String,
    pub display_name: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub max_players: u64,
    pub playing: u64,
    pub player_tokens: Vec<String>,
    pub players: Vec<Player>,
    pub fps: f32,
    pub ping: Option<u64>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: Vec<Game>
}

pub async fn get_page(game_id: u64, cursor: &String) -> Result<Page, reqwest::Error> {
    let url = format!("https://games.roblox.com/v1/games/{}/servers/Public?cursor={}&sortOrder=Desc&excludeFullGames=false&limit=100", game_id, cursor);
    let response = send_request(CLIENT.get(url)).await?;
    let page = response.json::<Page>().await?;
    Ok(page)
}