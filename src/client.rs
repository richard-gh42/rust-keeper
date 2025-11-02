use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

use eframe::App;
use egui::Visuals;

use crate::{coms::Player, data, gui::UserInput};

pub(crate) fn main(mut gui_send: Sender<Vec<data::Player>>, mut gui_recv: Receiver<UserInput>) {}
