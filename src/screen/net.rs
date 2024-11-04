use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bincode;

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
        // pub fn bind_to_ip(&self, state: Arc<screen_state>) -> io::Result<()> {
        //     let listener = TcpListener::bind(&self.ipaddress)?;
        //     let clients = self.clients.clone();
        //     let state_clone = state.clone();
        
        //     // Spawning the listener thread
        //     let listener_thread = thread::spawn(move || {
        //         println!("Listener created {:?}", listener.local_addr());
        
        //         // Set the listener to non-blocking mode to allow for checking the kill signal
        //         listener.set_nonblocking(true)?;
        
        //         loop {
        //             if state_clone.get_kill_listener() {
        //                 println!("Listener shutting down");
        //                 break;
        //             }
        
        //             match listener.accept() {
        //                 Ok((mut stream, client_address)) => {
        //                     println!("New client connected: {}", client_address);
        //                     clients.lock().unwrap().insert(client_address, stream.try_clone().unwrap());
        //                     let clients_clone = clients.clone();
        //                     let state_clone = state_clone.clone();
        
        //                     // Spawning a thread to handle the client connection
        //                     thread::spawn(move || {
        //                         let mut buffer = [0; 512];
        //                         while !state_clone.get_kill_listener() {
        //                             match stream.read(&mut buffer) {
        //                                 Ok(size) if size > 0 => {
        //                                     // Process data...
        //                                 },
        //                                 Ok(_) | Err(_) => break, // Client disconnected or error occurred
        //                             }
        //                         }
        //                         println!("Client disconnected: {}", client_address);
        //                         clients_clone.lock().unwrap().remove(&client_address);
        //                         let _ = stream.shutdown(Shutdown::Both);
        //                     });
        //                 }
        //                 Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        //                     // No incoming connection, continue the loop
        //                     thread::sleep(std::time::Duration::from_millis(100));
        //                 }
        //                 Err(e) => {
        //                     println!("Connection failed: {}", e);
        //                 }
        //             }
        //         }
        
        //         drop(listener); // Explicitly drop the listener
        //         Ok::<(), io::Error>(())
        //     });
            
        //     listener_thread.join();
        //     println!("bind to ip dead");
        //     Ok(())
        // }
        pub fn bind_to_ip(&self, state: Arc<screen_state>) -> io::Result<()> {
            let listener = TcpListener::bind(&self.ipaddress)?;

            let clients = self.clients.clone();
            listener.set_nonblocking(true)?;
            thread::spawn(move || {
                println!("listener created {:?}",listener.local_addr());
                for stream in listener.incoming() {
                    thread::sleep(Duration::from_millis(50));
                    if state.get_kill_listener()==true{
                        state.set_kill_listener(false);
                        println!("drop listener");
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
                                loop {
                                    match stream.read(&mut buffer) {
                                        Ok(size) if size > 0 => {
                                            // Process the data received (you can add your own processing logic here)
                                            println!("Received data from {}: {:?}", client_address, &buffer[..size]);
                                        }
                                        Ok(_) => {
                                            // No data received, just continue the loop
                                            thread::sleep(Duration::from_millis(50)); // Sleep for a short time to avoid busy waiting
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                            // No data available right now, yield to other threads
                                            thread::sleep(Duration::from_millis(50)); // Sleep for a short time to avoid busy waiting
                                        }
                                        Err(_) => {
                                            // An error occurred, assume the client has disconnected
                                            println!("Client disconnected: {}", client_address);
                                            clients_clone.lock().unwrap().remove(&client_address);
                                            let _ = stream.shutdown(Shutdown::Both);
                                            break;
                                        }
                                    }
                                }
                            });
                        },
                        Err(e) => {
                            // println!("Connection failed: {}", e);
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
                   
                    match stream.write_all(chunk) {
                        Ok(T) =>{},
                        Err(e) =>  {
                            println!("{:?}",e);
                            is_valid = false;
                            break;
                        }
                        
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

        pub fn receive_image_and_struct(&self, stream: &mut TcpStream, state: Arc<screen_state>) -> io::Result<Screenshot> {
            const STOP_MESSAGE: &[u8] = b"STOP";
            let mut buffer = vec![0; CHUNK_SIZE];
            let mut data = Vec::new();
            
            // Set a read timeout of 50 milliseconds
            stream.set_read_timeout(Some(Duration::from_millis(150)))?;
            
            loop {
                if state.get_sc_state() == StreamingState::STOP {
                    break;
                }
                
                match stream.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            println!("zero bytes read");
                            break;
                        }
                        if buffer[..bytes_read].ends_with(STOP_MESSAGE) {
                            println!("stop message bytes read");
                            data.extend_from_slice(&buffer[..bytes_read - STOP_MESSAGE.len()]);
                            break;
                        } else {
                            println!("bytes read");
                            data.extend_from_slice(&buffer[..bytes_read]);
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                        println!("error kind: {:?}_ {:?}", e,e.kind());
                        // buffer.clear();
                        // data.clear();
                        println!("Read timed out or non-blocking error occurred, retrying...");
                        continue;
                    }
                    Err(e) => {
                        // Handle other types of errors
                        println!("not read");
                        return Err(e);
                    }
                }
            }
            
            // Attempt to deserialize the data into a Screenshot
            match bincode::deserialize::<Screenshot>(&data) {
                Ok(screenshot) => {
                    println!("{:?}", screenshot.width);
                    Ok(screenshot)
                }
                Err(e) => {
                    println!("screenshot error");
                    Err(Error::last_os_error())
                },
            }
        }
    }
}