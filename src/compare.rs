use std::cmp::Ordering;

use crate::fightcade::ReplayInfo;

const REASONABLE_MATCH_LENGTH: f64 = 900.0;

pub fn compare_replays(a: &ReplayInfo, b: &ReplayInfo, bias_close: bool) -> Ordering {
    // Prefer ranked replays
    if a.ranked > 0 && b.ranked == 0 {
        return Ordering::Greater;
    }
    if b.ranked > 0 && a.ranked == 0 {
        return Ordering::Less;
    }

    // For ranked, use extra sorting options
    if a.ranked > 0 && b.ranked > 0 {
        // If biased, prefer close matches
        if bias_close {
            // Calculate score difference between players and return the one with the lowest
            let score_diff_a = (a.players[0].score.unwrap_or_default()
                - a.players[1].score.unwrap_or_default())
            .abs();
            let score_diff_b = (b.players[0].score.unwrap_or_default()
                - b.players[1].score.unwrap_or_default())
            .abs();
            let replay_score_difference = score_diff_a.cmp(&score_diff_b);
            if replay_score_difference != Ordering::Equal {
                return replay_score_difference;
            }
        }

        // Prefer closely ranked players
        let rank_diff_a =
            (a.players[0].rank.unwrap_or_default() - a.players[1].rank.unwrap_or_default()).abs();
        let rank_diff_b =
            (b.players[0].rank.unwrap_or_default() - b.players[1].rank.unwrap_or_default()).abs();
        let replay_rank_difference = rank_diff_a.cmp(&rank_diff_b);
        if replay_rank_difference != Ordering::Equal {
            return replay_rank_difference;
        }
    }

    // Prefer matches with a reasonable duration (5-10 min)
    let duration_a = (a.duration - REASONABLE_MATCH_LENGTH).abs();
    let duration_b = (b.duration - REASONABLE_MATCH_LENGTH).abs();
    duration_a
        .partial_cmp(&duration_b)
        .unwrap_or(Ordering::Equal)
}
