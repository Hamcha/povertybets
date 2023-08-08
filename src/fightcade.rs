use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::compare::compare_replays;

const FIGHTCADE_API_ENDPOINT: &str = "https://www.fightcade.com/api/";
const APIQUARK: &str = "searchquarks";

#[derive(Serialize, Debug)]
struct ReplayRequest {
    req: &'static str,
    best: bool,
    since: i64,
    offset: u64,
    limit: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    gameid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Player {
    pub name: String,
    pub country: String,
    pub rank: Option<i8>,
    pub score: Option<i8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReplayInfo {
    pub quarkid: String,
    pub channelname: String,
    pub date: i64,
    pub duration: f64,
    pub emulator: String,
    pub gameid: String,
    pub num_matches: i64,
    pub players: [Player; 2],
    pub ranked: i8,
    pub replay_file: String,
    pub realtime_views: Option<i64>,
    pub saved_views: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct ReplayResults {
    results: Vec<ReplayInfo>,
    count: i64,
}

#[derive(Deserialize, Debug)]
pub struct ReplayResponse {
    results: ReplayResults,
}

async fn get_replay_data(req: &ReplayRequest) -> anyhow::Result<ReplayResponse> {
    let res = reqwest::Client::new()
        .post(FIGHTCADE_API_ENDPOINT)
        .json(req)
        .send()
        .await?;
    Ok(res.json().await?)
}

pub async fn fetch_replays_for_game(
    gameid: String,
    since: DateTime<Utc>,
    limit: u16,
    bias_close: bool,
) -> anyhow::Result<Vec<ReplayInfo>> {
    let req = ReplayRequest {
        req: APIQUARK,
        best: true,
        since: since.timestamp(),
        offset: 0,
        limit: limit * 8, // for each replay we want, get at least 8 to choose from
        gameid: Some(gameid.clone()),
    };

    let mut all_replays = vec![];
    let best_replays = get_replay_data(&req).await?;
    info!(
        "[{}] found {} best replays",
        gameid, best_replays.results.count
    );
    all_replays.extend(best_replays.results.results);

    // If we haven't got enough replays yet, fetch the latest
    if all_replays.len() < limit.into() {
        let latest_replays = get_replay_data(&ReplayRequest { best: false, ..req }).await?;
        info!(
            "[{}] found {} latest replays",
            gameid, latest_replays.results.count
        );
        all_replays.extend(latest_replays.results.results);
    }

    // Sort replays by quality
    all_replays.sort_by(|a, b| compare_replays(a, b, bias_close));

    // Take only the required number of replays
    Ok(all_replays.into_iter().take(limit.into()).collect())
}
