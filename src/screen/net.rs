use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
pub mod net {
    use std::time::Duration;

    use io::{Error, ErrorKind};

    use crate::{enums::StreamingState, screen::screen::screen_state};

    use super::*;

    const CHUNK_SIZE: usize = 10 * 1024;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Screenshot {
        pub data: Vec<u8>,
        pub width: u32,
        pub height: u32,
        pub state: State,
        pub line_annotation: Option<Vec<(f32, f32, f32, f32)>>,
        pub circle_annotation: Option<Vec<(f32, f32, f32, f32)>>,
        pub text_annotation: Option<Vec<(f32, f32, String)>>,
        pub color: Option<[u8; 4]>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum State {
        Sending,
        Receiving,
        Blank,
        Stop,
    }

    pub struct Server {
        ipaddress: String,
        clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    }

    impl Server {
        pub fn new(ipaddress: String) -> Self {
            Server {
                ipaddress,
                clients: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn bind_to_ip(&self, state: Arc<screen_state>) -> io::Result<()> {
            let listener = TcpListener::bind(&self.ipaddress)?;
            listener.set_nonblocking(true)?;
            let clients = Arc::clone(&self.clients);

            thread::spawn(move || {
                println!(
                    "Server started and listening on {:?}",
                    listener.local_addr()
                );
                for stream in listener.incoming() {
                    if state.get_kill_listener() {
                        println!("Listener stopping...");
                        break;
                    }

                    match stream {
                        Ok(mut stream) => {
                            let client_address = match stream.peer_addr() {
                                Ok(addr) => addr,
                                Err(_) => continue,
                            };

                            println!("New client connected: {}", client_address);
                            clients
                                .lock()
                                .unwrap()
                                .insert(client_address, stream.try_clone().unwrap());

                            let clients_clone = Arc::clone(&clients);
                            thread::spawn(move || {
                                let mut buffer = [0; 512];
                                loop {
                                    match stream.read(&mut buffer) {
                                        Ok(size) if size > 0 => {
                                            println!(
                                                "Received data from {}: {:?}",
                                                client_address,
                                                &buffer[..size]
                                            );
                                        }
                                        Ok(_) => {
                                            thread::sleep(Duration::from_millis(50));
                                        }
                                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                            thread::sleep(Duration::from_millis(50));
                                        }
                                        Err(_) => {
                                            println!("Client disconnected: {}", client_address);
                                            clients_clone.lock().unwrap().remove(&client_address);
                                            let _ = stream.shutdown(Shutdown::Both);
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        Err(_) => continue,
                    }
                }
            });

            Ok(())
        }

        pub fn send_to_all_clients(&self, screenshot: &Screenshot) -> io::Result<()> {
            const STOP_MESSAGE: &[u8] = b"STOP";

            let serialized_data = bincode::serialize(screenshot)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Serialization failed"))?;
            let mut invalid_clients = Vec::new();

            let clients = self.clients.lock().unwrap();
            for (&client_address, client_stream) in clients.iter() {
                let mut stream = client_stream.try_clone()?;
                let mut is_valid = true;

                for chunk in serialized_data.chunks(CHUNK_SIZE) {
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

        pub fn is_connected(&self, stream: &TcpStream) -> bool {
            stream.peer_addr().is_ok()
        }

        pub fn connect_to_ip(&self) -> io::Result<TcpStream> {
            const MAX_RETRIES: u32 = 5;
            const RETRY_DELAY: u64 = 2000;

            let mut attempt = 0;
            loop {
                match TcpStream::connect(&self.server_address) {
                    Ok(stream) => {
                        println!("Connected to server at {}", self.server_address);
                        return Ok(stream);
                    }
                    Err(e) => {
                        attempt += 1;
                        println!(
                            "Connection attempt {}/{} failed: {}",
                            attempt, MAX_RETRIES, e
                        );
                        if attempt >= MAX_RETRIES {
                            println!("Maximum retry attempts reached.");
                            return Err(e);
                        }
                        println!("Retrying in {} ms...", RETRY_DELAY);
                        thread::sleep(Duration::from_millis(RETRY_DELAY));
                    }
                }
            }
        }

        pub fn receive_image_and_struct(
            &self,
            stream: &mut TcpStream,
            state: Arc<screen_state>,
        ) -> io::Result<Screenshot> {
            const STOP_MESSAGE: &[u8] = b"STOP";
            let mut buffer = vec![0; CHUNK_SIZE];
            let mut data = Vec::new();

            stream.set_read_timeout(Some(Duration::from_secs(5)))?;

            loop {
                if state.get_sc_state() == StreamingState::STOP {
                    break;
                }

                match stream.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            println!("Connection lost, attempting reconnection...");
                            let mut new_stream = self.connect_to_ip()?;
                            *stream = new_stream;
                            continue;
                        }
                        if buffer[..bytes_read].ends_with(STOP_MESSAGE) {
                            data.extend_from_slice(&buffer[..bytes_read - STOP_MESSAGE.len()]);
                            break;
                        } else {
                            data.extend_from_slice(&buffer[..bytes_read]);
                        }
                    }
                    Err(ref e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            || e.kind() == io::ErrorKind::TimedOut =>
                    {
                        println!("Timeout or no data available, retrying...");
                        state.set_reconnect(true);
                        break;
                    }
                    Err(e) => {
                        println!("Error: {}. Attempting reconnection...", e);
                        let mut new_stream = self.connect_to_ip()?;
                        *stream = new_stream;
                        continue;
                    }
                }
            }

            bincode::deserialize::<Screenshot>(&data).map_err(|e| {
                println!("Failed to deserialize screenshot: {:?}", e);
                io::Error::new(io::ErrorKind::InvalidData, "Deserialization failed")
            })
        }
    }
}
