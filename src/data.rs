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

impl From<&Character> for ComCharacter {
    fn from(value: &Character) -> Self {
        Self {
            name: value.name.to_owned(),
            stats: value.pub_stats.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Character {
    pub(crate) name: String,
    pub(crate) pub_stats: VecDeque<Stat>,
    pub(crate) pri_stats: VecDeque<Stat>,
}

impl From<&ComCharacter> for Character {
    fn from(value: &ComCharacter) -> Self {
        Self {
            name: value.name.to_owned(),
            pub_stats: value.stats.to_owned(),
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
