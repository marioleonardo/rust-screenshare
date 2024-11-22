use std::io::{self, Write};
use std::sync::Arc;
use enums::StreamingState;
use screen::screen::screen_state;

mod screen;
mod enums;

use screen::net::net::*;

fn main() {
    let mut input = String::new();
    println!("Enter 0 for server or 1 for client:");
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let choice: u32 = input.trim().parse().expect("Invalid input");

    if choice == 0 {
        println!("Enter the IP address to bind the server:");
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let ipaddress = input.trim().to_string();

        let server = Server::new("127.0.0.1:9848".to_string());
        let state = screen_state::default();
        state.set_screen_state(StreamingState::START);
        
        let state = Arc::new(state);
        
        server.bind_to_ip(state).expect("Failed to bind server to IP");
        while(true){
            // println!("Server is running");

            let _ = server.send_to_all_clients(&Screenshot {
                data: vec![1; 100 * 1024],
                width: 20,
                height: 20,
                state: State::Sending,
                line_annotation: None,
                circle_annotation: None,
                text_annotation: None,
            });
        }
    } else if choice == 1 {
        println!("Enter the server IP address to connect:");
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let server_address = input.trim().to_string();

        let client = Client::new("127.0.0.1:7879".to_string(), "127.0.0.1:9848".to_string());
        let mut stream = client.connect_to_ip().expect("Failed to connect to server");

        let state = Arc::new(screen_state::default());
        loop {
            match client.receive_image_and_struct(&mut stream, state.clone()) {
                Ok(screenshot) => {
                    println!("Received screenshot with width: {}", screenshot.width);
                }
                Err(e) => {
                    println!("Error receiving screenshot: {}", e);
                    break;
                }
            }
        }
    } else {
        println!("Invalid choice");
    }
}
