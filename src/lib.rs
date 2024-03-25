mod image_handler;
mod game_handler;
mod request;

use std::error;
pub use crate::image_handler::{find_from_player_tokens, get_player_image_token, get_player_image_tokens};
pub use crate::game_handler::{get_page, Game};

pub async fn find_player_game(user_id: u64, place_id: u64) -> Result<Option<Game>, Box<dyn error::Error>> {
    let target_token = get_player_image_token(user_id).await?;

    let mut games: Vec<Game> = vec![];
    let mut cursor: String = String::new();

    loop {
        let page = get_page(place_id, &cursor).await?;

        for game in page.data {
            games.push(game);
        }

        match page.next_page_cursor {
            Some(next_page) => cursor = next_page,
            None => break
        }
    }

    let mut found_game: Option<Game> = None;

    for game in games {
        let token = find_from_player_tokens(&game.player_tokens, &target_token, 0).await;

        match token {
            Some(_) => {
                found_game = Some(game);
                break
            }
            None => continue
        }
    }

    Ok(found_game)
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_ID: u64 = 1225757256; // https://www.roblox.com/users/1225757256/profile
    const PLACE_ID: u64 = 189707; // https://www.roblox.com/games/189707/Natural-Disaster-Survival

    #[tokio::test]
    async fn test_get_player_image_token() -> Result<(), reqwest::Error> {
        let player_image_token = get_player_image_token(USER_ID).await?;

        println!("Player image token: {}", player_image_token);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_page() -> Result<(), reqwest::Error> {
        let cursor = String::new();
        let page = get_page(PLACE_ID, &cursor).await;
        let next_cursor = page?.next_page_cursor;

        println!("Next Page: {:?}", next_cursor);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_player_image_tokens() -> Result<(), reqwest::Error> {
        let cursor = String::new();
        let page = get_page(PLACE_ID, &cursor).await?;
        let image_tokens = get_player_image_tokens(&page.data.first().unwrap().player_tokens).await?;

        println!("Player image urls: {:?}", image_tokens.data.iter().map(|response| response.image_url.clone()).collect::<Vec<String>>());

        Ok(())
    }
}
