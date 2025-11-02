use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::data::Stat;

#[derive(Serialize, Deserialize)]
pub(crate) enum Signal {
    Update(Player),
    AskPlayer(String),
    CharAvalible,
    CharNotAvalible,
    FaultySignal,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Player {
    pub(crate) name: String,
    pub(crate) stats: VecDeque<Stat>,
}
