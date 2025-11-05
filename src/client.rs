use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

use eframe::App;
use egui::Visuals;

use crate::{data::Character, gui::UserInput};

pub(crate) fn main(mut gui_send: Sender<Vec<Character>>, mut gui_recv: Receiver<UserInput>) {}
