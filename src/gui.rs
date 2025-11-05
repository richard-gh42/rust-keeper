use eframe::App;

use egui::{CentralPanel, Visuals};

use std::{
    net::SocketAddr,
    sync::mpsc::{Receiver, Sender},
};

use crate::data;

/// This is the struckt implememnting the GUI
pub(crate) struct KeeperGUI {
    players: Vec<data::Character>,
    state: ProgrammState,
    recv: Receiver<Vec<data::Character>>,
    send: Sender<UserInput>,
}

impl KeeperGUI {
    /// Creates new instance
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        recv: Receiver<Vec<data::Character>>,
        send: Sender<UserInput>,
    ) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());
        Self {
            players: Vec::new(),
            state: ProgrammState::Launch,
            recv: recv,
            send: send,
        }
    }

    /// Handles the GUI at game app launch
    fn launch(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let mut selection = SelectMode::Client;
            ui.radio_value(&mut selection, SelectMode::Client, "CLient");
            ui.radio_value(&mut selection, SelectMode::Host, "Host");
            if ui.button("Continue").clicked() {
                match selection {
                    SelectMode::Client => {
                        self.state = ProgrammState::Client;
                        let _ = self.send.send(UserInput::Select(ProgrammState::Client));
                    }
                    SelectMode::Host => {
                        self.state = ProgrammState::Host;
                        let _ = self.send.send(UserInput::Select(ProgrammState::Host));
                    }
                }
            }
        });
    }

    /// Handles the GUI of the host
    fn host(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {}

    /// Handles the GUI of the client
    fn client(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {}
}

impl App for KeeperGUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.state {
            ProgrammState::Launch => self.launch(ctx, frame),
            ProgrammState::Client => self.client(ctx, frame),
            ProgrammState::Host => self.host(ctx, frame),
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.send.send(UserInput::Quit);
    }
}

pub(crate) enum ProgrammState {
    Launch,
    Host,
    Client,
}

pub(crate) enum UserInput {
    Select(ProgrammState),
    Connect(SocketAddr),
    SetPasswd(String),
    Quit,
}

#[derive(PartialEq)]
enum SelectMode {
    Host,
    Client,
}
