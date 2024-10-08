use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bincode;

pub mod net {
    use io::Error;

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

            let clients = self.clients.clone();
            thread::spawn(move || {
                for stream in listener.incoming() {
                    if state.get_sc_state()==StreamingState::STOP{
                        drop(listener);
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
            if let Ok(stream) = TcpStream::connect(&self.server_address){
                println!("Connected to server at {}", self.server_address);
                Ok(stream)
            }
            else{
                println!("Not connected to server");
                Err(Error::last_os_error())
            } 
        }

        pub fn is_connected(&self, stream: &TcpStream) -> bool {
            stream.peer_addr().is_ok()
        }

        pub fn receive_image_and_struct(&self, stream: &mut TcpStream,state:Arc<screen_state>) -> io::Result<Screenshot> {
            const STOP_MESSAGE: &[u8] = b"STOP";
            let mut buffer = vec![0; CHUNK_SIZE];
            let mut data = Vec::new();
            loop {
                if state.get_sc_state()==StreamingState::STOP{
                    break;
                }
                if let Ok(bytes_read) = stream.read(&mut buffer){
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
                else{
                    println!("not read");
                }
                
            }
            if let Ok(screenshot) = bincode::deserialize::<Screenshot>(&data){
                println!("{:?}",screenshot.width);
                Ok(screenshot)
            }
            else{
                Err(Error::last_os_error())
            }
            
            
        }
    }
}