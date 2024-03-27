mod image_handler;
mod game_handler;
mod request;

use std::time::Duration;
use clap::Parser;
use indicatif::ProgressBar;
use log::info;
use crate::game_handler::{Game};
use crate::image_handler::{find_from_player_tokens, get_player_image_token};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // The target player id: https://www.roblox.com/users/(THE_ID_HERE)/profile
    target: u64,
    // The target place id: https://www.roblox.com/games/(THE_ID_HERE)/
    place: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::try_init().unwrap();

    let settings = Cli::parse();

    let target_token = get_player_image_token(settings.target).await?;

    info!("Player token: {}.", target_token);

    info!("Getting the list of games...");

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut games: Vec<Game> = vec![];
    let mut cursor: String = String::new();

    loop {
        let page = game_handler::get_page(settings.place, &cursor).await?;

        for game in page.data {
            games.push(game);
        }

        match page.next_page_cursor {
            Some(nextPage) => cursor = nextPage,
            None => break
        }

        spinner.set_message(format!("Games found: {}", games.len()));
    }

    spinner.finish_with_message(format!("Total of {} games found!", games.len()));

    let mut found_game: Option<Game> = None;

    info!("Checking all games...");

    let progress_bar = ProgressBar::new(games.len() as u64);
    for game in games {
        progress_bar.inc(1);
        let token = find_from_player_tokens(&game.player_tokens, &target_token, 0).await;
        match token {
            Some(_) => {
                found_game = Some(game);
                break;
            },
            None => {}
        }
    }

    progress_bar.finish();

    match found_game {
        Some(target_game) => {
            info!("Join code: Roblox.GameLauncher.joinGameInstance({}, '{}')", settings.place, target_game.clone().id);
        },
        None => info!("User not found, possibly could not fetch image!")
    }

    Ok(())
}
