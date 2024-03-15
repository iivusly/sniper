use serde::{Deserialize, Serialize};
use crate::request::{CLIENT, send_no_fail_request};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub playerToken: String,
    pub id: u64,
    pub name: String,
    pub displayName: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub maxPlayers: u64,
    pub playing: u64,
    pub playerTokens: Vec<String>,
    pub players: Vec<Player>,
    pub fps: f32,
    pub ping: Option<u64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub previousPageCursor: Option<String>,
    pub nextPageCursor: Option<String>,
    pub data: Vec<Game>
}

pub async fn get_page(game_id: u64, cursor: String) -> Page {
    let url = format!("https://games.roblox.com/v1/games/{}/servers/Public?cursor={}&sortOrder=Desc&excludeFullGames=false&limit=100", game_id, cursor);
    let response = send_no_fail_request(CLIENT.get(url)).await;
    let page = response.json::<Page>().await.unwrap();
    page
}