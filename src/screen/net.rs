/*pub mod net{
    
    use std::net::{TcpListener, TcpStream};
    use std::io::{self,Read};
    use std::io::Write;
    
    pub fn send_screenshot(screenshot: &mut  Vec<u8>,ipaddress:String) -> io::Result<()> {
        // Create a TCP stream and connect to the server
        let mut stream = TcpStream::connect(ipaddress)?;
    
        // Convert the image buffer to a byte array
        let frame_bytes = screenshot.clone();
    
        // Send the start message
        stream.write_all(b"START_PHOTO")?;
        println!("Sending screenshot of size: {}", frame_bytes.len());
    
        // Send the image data in chunks of size 1024 bytes
        let chunk_size = 1024;
        for chunk in frame_bytes.chunks(chunk_size) {
            stream.write_all(chunk)?;
        }
    
        // Send the end message
        stream.write_all(b"END_PHOTO")?;
    
    
        Ok(())
    }

    // Function to receive a screenshot over TCP
pub fn receive_screenshot(_width: u32, _height: u32, ipaddress:String) -> io::Result<Vec<u8>> {
    // Create a TCP listener and bind to port 3000
    let listener = TcpListener::bind(ipaddress)?;

    // Wait for a connection
    let (mut stream, _addr) = listener.accept()?;

    let mut buffer = Vec::new();
    let mut ready = false;

    // Buffer to store received data in chunks of size 1024 bytes
    let mut chunk = [0u8; 1024];

    // Read data from the stream
    loop {
        match stream.read(&mut chunk) {
            Ok(size) if size > 0 => {
                if &chunk[0..size] == b"START_PHOTO" {
                    ready = true;
                    continue; // Skip appending the start message
                }
                if &chunk[0..size] == b"END_PHOTO" && ready {
                    break; // Exit the loop if end message is received after start message
                }
                if ready {
                    buffer.extend_from_slice(&chunk[..size]);
                }
            }

            Ok(_) => break, // Stop receiving on empty read
            Err(e) => return Err(e), // Return error
        }
    }

    println!("Receiving screenshot of size: {}", buffer.len());
    // Create an image buffer from the received data
    let received_image =buffer;

    Ok(received_image)
}
}
    */

use std::io;
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

pub mod net {
    use super::*;

    // Tamaño máximo de un fragmento en bytes (10KB)
    const CHUNK_SIZE: usize = 10 * 1024;

    pub struct Server {
        ipaddress: String,
        clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    }

    pub struct Screenshot {
        pub data: Vec<u8>,
        pub width: u32,
        pub height: u32,
        pub state: State,
    }

    pub enum State {
        Sending,
        Receiving,
    }

    impl Server {
        pub fn new(ipaddress: String) -> Self {
            Server {
                ipaddress,
                clients: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn bind_to_ip(&self) -> io::Result<()> {
            let listener = TcpListener::bind(&self.ipaddress)?;

            let clients = self.clients.clone();
            thread::spawn(move || {
                for stream in listener.incoming() {
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

            let mut invalid_clients = Vec::new();
            let clients = self.clients.lock().unwrap();
            for (&client_address, client_stream) in clients.iter() {
                let mut stream = client_stream.try_clone()?;
                let mut is_valid = true;
                for chunk in screenshot.data.chunks(CHUNK_SIZE) {
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
            let screenshot = Screenshot {
                data,
                width: 1920,  // Placeholder width
                height: 1080, // Placeholder height
                state: State::Receiving,
            };
            Ok(screenshot)
        }
    }
}
/* 
pub fn receive_screenshot(width: u32, height: u32, ipaddress: String) -> io::Result<Vec<u8>> {
    let client = net::Client::new(ipaddress.clone(), ipaddress);
    let mut stream = client.connect_to_ip()?;
    if client.is_connected(&stream) {
        let screenshot = client.receive_image_and_struct(&mut stream)?;
        if let net::State::Receiving = screenshot.state {
            println!("Received a screenshot with resolution: {}x{}", width, height);
            return Ok(screenshot.data);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Failed to receive screenshot"))
}

pub fn send_screenshot(screenshot: &mut Vec<u8>, ipaddress: String) -> io::Result<()> {
    let server = net::Server::new(ipaddress.clone());
    server.bind_to_ip()?;
    let screenshot_data = net::Screenshot {
        data: screenshot.clone(),
        width: 1920,  // Placeholder width
        height: 1080, // Placeholder height
        state: net::State::Sending,
    };
    server.send_to_all_clients(&screenshot_data)
}
    */