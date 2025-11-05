use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) enum Signal {
    Update(ComCharacter),
    SetName(String),
    NameResponse(bool),
    AskPasswd,
    Passwd(String),
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ComCharacter {
    pub(crate) name: String,
    pub(crate) stats: VecDeque<Stat>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Character {
    pub(crate) name: String,
    pub(crate) pub_stats: VecDeque<Stat>,
    pub(crate) pri_stats: VecDeque<Stat>,
}

impl Character {
    fn to_coms(&self) -> ComCharacter {
        ComCharacter {
            name: self.name.to_owned(),
            stats: self.pub_stats.clone(),
        }
    }

    fn from_coms(player: ComCharacter) -> Self {
        Self {
            name: player.name,
            pub_stats: player.stats,
            pri_stats: VecDeque::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Stat {
    pub(crate) name: String,
    pub(crate) value: StatValue,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum StatValue {
    Bool(bool),
    Num(i64),
    Str(String),
}
