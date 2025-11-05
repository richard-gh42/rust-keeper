use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread::{JoinHandle, spawn},
};

use ciborium::{from_reader, into_writer};

use crate::{
    data::{Character, ComCharacter, Signal},
    gui::UserInput,
};

pub(crate) fn main(gui_send: Sender<Vec<Character>>, gui_recv: Receiver<UserInput>) {
    let passwd = "moin".to_string();
    let (connection_send, connection_recv) = channel::<(JoinHandle<()>, ConComMain)>();
    let listener_handle = spawn(move || listen(connection_send, passwd));

    let mut new_connections: Vec<(JoinHandle<()>, ConComMain)> = Vec::with_capacity(4);
    let mut active_connections: HashMap<String, (JoinHandle<()>, ConComMain)> = HashMap::new();

    loop {
        // add new connections from listener
        match connection_recv.try_recv() {
            Ok(con) => {
                new_connections.push(con);
            }
            Err(_) => {}
        }
        // Get name for new connenctions and add them to active connections
        for i in 0..new_connections.len() {
            match new_connections.get(i).unwrap().1.recv.try_recv() {
                Ok(sig) => match sig {
                    ConSigFromClient::SelectName(name) => {
                        if active_connections.contains_key(&name) {
                            // if the name already exists refuse
                            match new_connections
                                .get(i)
                                .unwrap()
                                .1
                                .send
                                .send(ConSigToClient::AwnserChar(false))
                            {
                                Err(_) => {
                                    // Remove if thresd has closed
                                    new_connections.remove(i);
                                }
                                _ => {}
                            }
                        } else {
                            // If the name doesn't exist add connection to active connections and
                            match new_connections
                                .get(i)
                                .unwrap()
                                .1
                                .send
                                .send(ConSigToClient::AwnserChar(true))
                            {
                                Err(_) => {
                                    // Remove if thresd has closed
                                    new_connections.remove(i);
                                }
                                Ok(_) => {
                                    active_connections.insert(name, new_connections.remove(i));
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Err(_) => todo!(),
            }
        }
    }
}

/// Listens for incoming connections
fn listen(send: Sender<(JoinHandle<()>, ConComMain)>, passwd: String) {
    // Binds to a OS assinged port
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind to port");
    println!("Listens on {}", listener.local_addr().unwrap());
    // For every incoming connection start a handler and send the handle and Comunications chanel to main
    for res in listener.incoming() {
        match res {
            Ok(stream) => {
                // Chanel from the handler to the central thread
                let (sendh, recvm) = channel::<ConSigFromClient>();
                // Chanel from the central thread to the handler
                let (sendm, recvh) = channel::<ConSigToClient>();
                // for password checking
                let passwd = passwd.clone();
                //spawn thread and send it and the chanel handles to the cantral thread.
                send.send((
                    spawn(move || {
                        handler(
                            stream,
                            ConComHand {
                                send: sendh,
                                recv: recvh,
                            },
                            passwd,
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
fn handler(mut stream: TcpStream, main_com: ConComHand, passwd: String) {
    // Save peer adress
    let peer = stream.peer_addr().unwrap();
    println!("Incomming connection from {}", peer.ip());
    // Ask for Password if one exists
    if passwd.len() > 0 {
        // send request for password to client
        into_writer(&Signal::AskPasswd, &mut stream).unwrap_or_else(|_| {
            println!("Connection to {} lost", peer.ip());
            return;
        });
        // handle response
        match from_reader::<Signal, TcpStream>(
            stream.try_clone().expect("Failed cloning stream refrence"),
        ) {
            Ok(sig) => match sig {
                Signal::Passwd(passwd_attempt) => {
                    // Only continue if password is correct
                    if passwd == passwd_attempt {
                        match into_writer(&Signal::Ok, &mut stream) {
                            Err(_) => {
                                println!("Connection to {} lost", peer.ip());
                                return;
                            }
                            _ => {}
                        };
                    } else {
                        // If the password is false, relay that and close the connection
                        match into_writer(&Signal::Err, &mut stream) {
                            Err(_) => {
                                println!("Connection to {} lost", peer.ip());
                                return;
                            }
                            _ => {}
                        };
                        println!(
                            "Incomming connection from {} refused due to wrong password",
                            peer.ip()
                        );
                        return;
                    }
                }
                _ => {
                    // if client doesn't respond with a password, close connection
                    match into_writer(&Signal::Err, &mut stream) {
                        Err(_) => {
                            println!("Connection to {} lost", peer.ip());
                            return;
                        }
                        _ => {}
                    };
                    println!(
                        "Incoming connection from {} refused due to signal missmatch",
                        peer.ip()
                    );
                    return;
                }
            },
            Err(_) => {
                // if no adequate response is forthcomming close the connection
                match into_writer(&Signal::Err, &mut stream) {
                    Err(_) => {
                        println!("Connection to {} lost", peer.ip());
                        return;
                    }
                    _ => {}
                };
                println!(
                    "Incoming connection from {} refused due to connection error",
                    peer.ip()
                );
                return;
            }
        }
    }

    // Set client Name
    loop {
        // get Name from client
        match from_reader::<Signal, TcpStream>(
            stream.try_clone().expect("Execeded file descriptor limit"),
        ) {
            Ok(sig) => match sig {
                Signal::SetName(name) => {
                    // Request central thread to set name for client
                    main_com
                        .send
                        .send(ConSigFromClient::SelectName(name))
                        .unwrap();
                    match main_com.recv.recv().unwrap() {
                        ConSigToClient::AwnserChar(allowed) => {
                            // If name is set Send afirmation to client and continue to
                            if allowed {
                                into_writer(&Signal::NameResponse(true), &mut stream)
                                    .unwrap_or_else(|_| {
                                        println!("Connection to {} lost", peer.ip());
                                        return;
                                    });
                                break;
                            } else {
                                println!("{} requested existing name", peer.ip());
                                match into_writer(&Signal::NameResponse(false), &mut stream) {
                                    Err(_) => {
                                        println!("Connection to {} lost", peer.ip());
                                        return;
                                    }
                                    _ => {}
                                };
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    println!("Received unexpected Signal from {}", peer.ip());
                    match into_writer(&Signal::Err, &mut stream) {
                        Err(_) => {
                            println!("Connection to {} lost", peer.ip());
                            return;
                        }
                        _ => {}
                    };
                }
            },
            Err(_) => {
                println!("Recieved faulty Signal from {}", peer.ip());
                match into_writer(&Signal::Err, &mut stream) {
                    Err(_) => {
                        println!("Connection to {} lost", peer.ip());
                        return;
                    }
                    _ => {}
                };
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
                        match into_writer(&Signal::Update(character), &mut stream) {
                            Err(_) => {
                                println!("Connection to {} lost", peer);
                                break;
                            }
                            _ => {}
                        }
                    }
                    ConSigToClient::Close => {
                        break;
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
    SendCharData(ComCharacter),
    Close,
}

/// Signals from the main thread to the connection handlers
enum ConSigFromClient {
    // select name
    SelectName(String),
    // Connection closed
    Closed,
}
