use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::coms;

#[derive(Serialize, Deserialize)]
pub(crate) struct Player {
    pub(crate) name: String,
    pub(crate) pub_stats: VecDeque<Stat>,
    pub(crate) pri_stats: VecDeque<Stat>,
}

impl Player {
    fn to_coms(&self) -> coms::Player {
        coms::Player {
            name: self.name.to_owned(),
            stats: self.pub_stats.clone(),
        }
    }

    fn from_coms(player: coms::Player) -> Self {
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
