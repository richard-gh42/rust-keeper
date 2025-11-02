use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender, channel},
    thread::{JoinHandle, spawn},
};

use ciborium::{from_reader, into_writer};

use crate::{
    coms::{self, Signal},
    data,
    gui::UserInput,
};

pub(crate) fn main(gui_send: Sender<Vec<data::Player>>, gui_recv: Receiver<UserInput>) {
    let (connection_send, connection_recv) = channel::<(JoinHandle<()>, ConComMain)>();
    let listener_handle = spawn(move || listen(connection_send));
}

/// Listens for incoming connections
fn listen(send: Sender<(JoinHandle<()>, ConComMain)>) {
    // Binds to a OS assinged port
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind to port");
    println!("Listens on {}", listener.local_addr().unwrap());
    // For every incoming connection start a handler and send the handle and Comunications chanel to main
    for res in listener.incoming() {
        match res {
            Ok(stream) => {
                println!(
                    "Successfuly established connection to {}",
                    &stream.peer_addr().unwrap()
                );
                let (sendh, recvm) = channel::<ConSigFromClient>();
                let (sendm, recvh) = channel::<ConSigToClient>();
                send.send((
                    spawn(move || {
                        handler(
                            stream,
                            ConComHand {
                                send: sendh,
                                recv: recvh,
                            },
                        )
                    }),
                    ConComMain {
                        send: sendm,
                        recv: recvm,
                    },
                ))
                .unwrap();
            }
            Err(e) => println!("There was an error during a connection atempt: {}", e),
        }
    }
}

/// Handles a connection
fn handler(mut stream: TcpStream, main_com: ConComHand) {
    // Save peer adress
    let peer = stream.peer_addr().unwrap();
    // coms::Player Selection
    let mut player_selected = false;
    // Repeat until player is selected
    while !player_selected {
        // Look for AskPlayer Signal
        match from_reader::<Signal, &mut TcpStream>(&mut stream).expect("Invalid package recieved")
        {
            Signal::AskPlayer(character) => {
                // Ask main if the player exists
                let _ = main_com.send.send(ConSigFromClient::Select(character));
                match main_com.recv.recv().unwrap() {
                    // Tell client whether player exists
                    ConSigToClient::AwnserChar(availible) => {
                        if availible {
                            into_writer(&Signal::CharAvalible, &mut stream);
                            player_selected = true;
                        } else {
                            into_writer(&Signal::CharNotAvalible, &mut stream);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                // Tell client their Signal was Faulty
                into_writer(&Signal::FaultySignal, &mut stream);
            }
        }
    }
    loop {
        // Check for Signal from Main
        match main_com.recv.try_recv() {
            Ok(sig) => {
                // Handle Signal
                match sig {
                    ConSigToClient::SendCharData(character) => {
                        match into_writer(&character, &mut stream) {
                            Ok(_) => {}
                            Err(_) => {
                                println!("Connection to {} lost", peer);
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let _ = main_com.send.send(ConSigFromClient::Closed);
}

/// This enum facilitates communication between the main thread and the connection handlers
struct ConComMain {
    send: Sender<ConSigToClient>,
    recv: Receiver<ConSigFromClient>,
}

struct ConComHand {
    send: Sender<ConSigFromClient>,
    recv: Receiver<ConSigToClient>,
}

/// Signals from the connection handller to the main thread
enum ConSigToClient {
    // Awnser to char selection
    AwnserChar(bool),
    // Send data to client
    SendCharData(coms::Player),
}

/// Signals from the main thread to the connection handlers
enum ConSigFromClient {
    // select char
    Select(String),
    // Connection closed
    Closed,
}
