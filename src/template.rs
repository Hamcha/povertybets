use askama::Template;
use std::collections::HashMap;

use crate::fightcade::ReplayInfo;

#[derive(Template)]
#[template(path = "template.html")]
pub struct HTMLTemplate {
    pub replays: HashMap<String, Vec<ReplayInfo>>,
}

mod filters {
    use askama::Result;

    // Strip everything after ( and trim it
    pub fn clean_name(s: &str) -> Result<String> {
        if let Some((start, _)) = s.split_once('(') {
            Ok(start.trim().to_string())
        } else {
            Ok(s.to_string())
        }
    }
}
