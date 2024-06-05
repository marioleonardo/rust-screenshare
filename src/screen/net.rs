pub mod net{
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

    println!("Sending screenshot of size: {}", buffer.len());
    // Create an image buffer from the received data
    let received_image =buffer;

    Ok(received_image)
}
}