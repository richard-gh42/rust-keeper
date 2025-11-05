use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::spawn,
};

use crate::gui::ProgrammState;

mod client;
mod data;
mod gui;
mod host;

fn main() {
    let (sendm, recvg) = channel::<Vec<data::Character>>();
    let (sendg, recvm) = channel::<gui::UserInput>();
    let native_options = eframe::NativeOptions::default();

    let main_tread_handle = spawn(move || run(sendm, recvm));

    eframe::run_native(
        "rust-keeper",
        native_options,
        Box::new(|cc| Ok(Box::new(gui::KeeperGUI::new(cc, recvg, sendg)))),
    )
    .expect("GUI crashed");
    let _ = main_tread_handle.join();
}

fn run(gui_send: Sender<Vec<data::Character>>, gui_recv: Receiver<gui::UserInput>) {
    match gui_recv.recv().unwrap() {
        gui::UserInput::Select(selection) => match selection {
            ProgrammState::Client => {
                client::main(gui_send, gui_recv);
            }
            ProgrammState::Host => {
                host::main(gui_send, gui_recv);
            }
            ProgrammState::Launch => {
                panic!("Can't select Launch");
            }
        },
        gui::UserInput::Quit => {
            return;
        }
        _ => panic!("Can only select a mode or Quit at the moment"),
    };
}
