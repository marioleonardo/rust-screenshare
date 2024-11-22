use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bincode;
pub mod net {
    use super::*;
    use std::sync::mpsc::{channel, Sender, Receiver};

    const CHUNK_SIZE: usize = 10 * 1024;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Screenshot {
        pub data: Vec<u8>,
        pub width: u32,
        pub height: u32,
        pub state: State,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum State {
        Sending,
        Receiving,
        Blank,
        Stop
    }

    pub struct Server {
        ipaddress: String,
        clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
        listener: Option<TcpListener>,  // Optional listener to allow rebinding
        shutdown_sender: Option<Sender<()>>,  // Channel to signal shutdown
    }

    impl Server {
        pub fn new(ipaddress: String) -> Self {
            Server {
                ipaddress,
                clients: Arc::new(Mutex::new(HashMap::new())),
                listener: None,
                shutdown_sender: None,
            }
        }

        // Method to bind the server and listen for connections
        pub fn bind_to_ip(&mut self) -> io::Result<()> {
            if let Some(ref listener) = self.listener {
                // Server is already bound
                println!("Server is already bound to {}", self.ipaddress);
                return Ok(());
            }

            let listener = TcpListener::bind(&self.ipaddress)?;
            self.listener = Some(listener.try_clone()?);

            let (shutdown_sender, shutdown_receiver) = channel();
            self.shutdown_sender = Some(shutdown_sender);

            let clients = self.clients.clone();
            let listener_clone = listener.try_clone()?;

            thread::spawn(move || {
                for stream in listener_clone.incoming() {
                    if let Ok(()) = shutdown_receiver.try_recv() {
                        println!("Server shutting down.");
                        break;
                    }

                    match stream {
                        Ok(mut stream) => {
                            let client_address = stream.peer_addr().unwrap();
                            println!("New client connected: {}", client_address);
                            
                            clients.lock().unwrap().insert(client_address, stream.try_clone().unwrap());
                            let clients_clone = clients.clone();

                            thread::spawn(move || {
                                let mut buffer = [0; 512];
                                while match stream.read(&mut buffer) {
                                    Ok(size) if size > 0 => true,
                                    _ => false,
                                } {}
                                println!("Client disconnected: {}", client_address);
                                clients_clone.lock().unwrap().remove(&client_address);
                                let _ = stream.shutdown(Shutdown::Both);
                            });
                        },
                        Err(e) => {
                            println!("Connection failed: {}", e); 
                        },
                    }
                }
            });
            Ok(())
        }

        // Method to stop the server and close the listener
        pub fn stop(&mut self) -> io::Result<()> {
            if let Some(ref shutdown_sender) = self.shutdown_sender {
                // Signal the server to shut down
                shutdown_sender.send(()).unwrap();
            }

            if let Some(ref listener) = self.listener {
                // Close the listener
                println!("Closing the server listener...");
                drop(listener);
                self.listener = None;
            }

            // Clear clients
            let mut clients = self.clients.lock().unwrap();
            for (addr, client_stream) in clients.iter() {
                let _ = client_stream.shutdown(Shutdown::Both);
                println!("Closed connection to client: {}", addr);
            }
            clients.clear();

            Ok(())
        }

        // Method to send a screenshot to all connected clients
        pub fn send_to_all_clients(&self, screenshot: &Screenshot) -> io::Result<()> {
            const STOP_MESSAGE: &[u8] = b"STOP";

            let serialized_data = bincode::serialize(screenshot);
            let mut invalid_clients = Vec::new();
            let clients = self.clients.lock().unwrap();
            for (&client_address, client_stream) in clients.iter() {
                let mut stream = client_stream.try_clone()?;
                let mut is_valid = true;
                for chunk in serialized_data.as_ref().unwrap().chunks(CHUNK_SIZE) {
                    if stream.write_all(chunk).is_err() {
                        is_valid = false;
                        break;
                    }
                }
                if is_valid {
                    if stream.write_all(STOP_MESSAGE).is_err() {
                        is_valid = false;
                    }
                }
                if !is_valid {
                    invalid_clients.push(client_address);
                }
            }
            drop(clients);

            // Remove invalid clients
            let mut clients = self.clients.lock().unwrap();
            for client_address in invalid_clients {
                clients.remove(&client_address);
                println!("Removed invalid client: {}", client_address);
            }

            Ok(())
        }
    }

    pub struct Client {
        ipaddress: String,
        server_address: String,
    }

    impl Client {
        pub fn new(ipaddress: String, server_address: String) -> Self {
            Client {
                ipaddress,
                server_address,
            }
        }

        pub fn connect_to_ip(&self) -> io::Result<TcpStream> {
            let stream = TcpStream::connect(&self.server_address)?;
            println!("Connected to server at {}", self.server_address);
            Ok(stream)
        }

        pub fn is_connected(&self, stream: &TcpStream) -> bool {
            stream.peer_addr().is_ok()
        }

        pub fn receive_image_and_struct(&self, stream: &mut TcpStream) -> io::Result<Screenshot> {
            const STOP_MESSAGE: &[u8] = b"STOP";
            let mut buffer = vec![0; CHUNK_SIZE];
            let mut data = Vec::new();
            loop {
                let bytes_read = stream.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                if buffer[..bytes_read].ends_with(STOP_MESSAGE) {
                    data.extend_from_slice(&buffer[..bytes_read - STOP_MESSAGE.len()]);
                    break;
                } else {
                    data.extend_from_slice(&buffer[..bytes_read]);
                }
            }
            let screenshot: Screenshot = bincode::deserialize(&data).unwrap();
            Ok(screenshot)
        }
    }
}
