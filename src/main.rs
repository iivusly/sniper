
#![allow(non_snake_case)]

mod image_handler;
mod game_handler;
mod request;

use std::{env};
use std::time::{Duration, SystemTime};
use humantime::format_rfc3339_seconds;
use indicatif::ProgressBar;
use log::info;
use crate::game_handler::{Game};
use crate::image_handler::{find_from_player_tokens, get_player_image_token};

struct Settings {
    target: u64,
    place: u64,
    // token: String
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    setup_logger().unwrap();

    let settings = Settings {
        target: env::var("TARGET").unwrap().parse().unwrap(),
        place: env::var("PLACE").unwrap().parse().unwrap(),
        // token: env::var("TOKEN").unwrap().parse().unwrap()
    };

    let target_token = get_player_image_token(settings.target).await;

    info!("Player token: {}.", target_token);

    info!("Getting the list of games...");

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut games: Vec<Game> = vec![];
    let mut cursor: String = String::new();

    loop {
        let page = game_handler::get_page(settings.place, cursor.clone()).await;
        for game in page.data {
            games.push(game)
        }
        match page.nextPageCursor {
            Some(nextPage) => cursor = nextPage,
            None => break
        }
    }

    spinner.finish();

    let mut found_game: Option<Game> = None;

    info!("Total of {} games found!", games.len());
    info!("Checking all games...");

    let progress_bar = ProgressBar::new(games.len() as u64);
    for game in games {
        progress_bar.inc(1);
        let token = find_from_player_tokens(game.clone().playerTokens, target_token.clone()).await;
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
