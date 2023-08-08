use askama::Template;
use chrono::{Duration, Utc};
use clap::Parser;
use serde_json::{from_slice, to_string};
use std::collections::HashMap;
use tokio::{
    fs::{read, File},
    io::AsyncWriteExt,
    spawn,
};
use tracing::{info, Level};

use crate::fightcade::fetch_replays_for_game;
use crate::template::HTMLTemplate;

mod compare;
mod fightcade;
mod template;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to game list file
    #[arg(short, long, value_name = "FILE", default_value = "games.txt")]
    game_list: String,

    /// How many replays to fetch per game
    #[arg(short, long, default_value = "5")]
    limit: u16,

    /// How many days back to look for
    #[arg(short = 'd', long, default_value = "7")]
    since_days: u16,

    /// Prefer close matches
    #[arg(long)]
    bias_close: bool,

    /// If specified, save match data to JSON
    #[arg(long, value_name = "FILE")]
    json_out: Option<String>,

    /// If specified, save match data to HTML page
    #[arg(long, value_name = "FILE", default_value = "rendered.html")]
    html_out: Option<String>,

    /// If specified, use provided JSON file instead of fetching data
    #[arg(long, value_name = "FILE")]
    json_in: Option<String>,
}

// Game list is a text file with game IDs, one per line, separated by newlines
// Blank lines are allowed (and ignored) and comments are allowed using #
// Example:
//
// # comment (ignored)
// game_name # some comment, also ignored
// game_name2
//
// game_name3
fn read_game_list(text: String) -> Vec<String> {
    // Split by lines
    text.split('\n')
        // Remove comments
        .map(|x| {
            if let Some((first, _)) = x.split_once('#') {
                first
            } else {
                x
            }
        })
        // Trim whitespace
        .map(|x| x.trim())
        // Remove empty lines
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let matches = match cli.json_in {
        Some(input_file) => {
            // Read from JSON file
            let contents = read(input_file).await?;
            from_slice(&contents)?
        }
        None => {
            // Read game list from file
            let contents = read(cli.game_list).await?;
            let list = read_game_list(String::from_utf8_lossy(&contents).parse()?);
            let min_time = Utc::now() - Duration::days(cli.since_days.into());
            // Fetch replays for each game and put it into a map (GameReplayList)
            let mut game_replays = HashMap::new();
            let mut tasks = vec![];

            for game in list {
                let task = spawn(fetch_replays_for_game(
                    game.clone(),
                    min_time,
                    cli.limit,
                    cli.bias_close,
                ));
                tasks.push((game, task));
            }

            for (game, task) in tasks {
                let replays = task.await?;
                game_replays.insert(game.clone(), replays?);
            }
            game_replays
        }
    };

    if let Some(output) = cli.json_out {
        let mut file = File::create(&output).await?;
        file.write_all(to_string(&matches)?.as_bytes()).await?;
        info!("Saved matches to {}", output);
    }

    if let Some(output) = cli.html_out {
        let tpl = HTMLTemplate { replays: matches };
        let mut file = File::create(&output).await?;
        file.write_all(tpl.render()?.as_bytes()).await?;
        info!("Saved matches to {}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_game_list() {
        let text = "# RANKED
sfiii3nr1 # Street Fighter III 3rd Strike: Fight for the Future
kof98 # The King of Fighters '98
kof2002 # The King of Fighters 2002
wakuwak7 # Waku Waku 7
garou # Garou - Mark of the Wolves
samsh5sp # Samurai Shodown V Special
sf2ce # Street Fighter II' - Champion Edition
sfa2u # Street Fighter Alpha 2
sgemf # Super Gem Fighter Mini Mix
jojobanr1 # JoJo's Bizarre Adventure: Heritage for the Future
vsavj # Vampire Savior

# RANKED BUT UNKNOWN
dankuga # Dan-Ku-Ga
rotd # Rage of the Dragons
";

        let games = super::read_game_list(text.to_string());
        assert_eq!(
            games,
            [
                "sfiii3nr1",
                "kof98",
                "kof2002",
                "wakuwak7",
                "garou",
                "samsh5sp",
                "sf2ce",
                "sfa2u",
                "sgemf",
                "jojobanr1",
                "vsavj",
                "dankuga",
                "rotd"
            ]
        );
    }
}
