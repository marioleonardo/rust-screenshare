#![deny(clippy::all)]
// #![forbid(unsafe_code)]
pub mod screen{
extern crate image;
extern crate winapi;

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winapi::shared::windef::HGDIOBJ;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use image::{ImageBuffer, Rgba};
use std::net::UdpSocket;
use std::ptr::null_mut;
use std::mem::zeroed;
use winapi::um::winuser::{GetDC, ReleaseDC, GetDesktopWindow};
use winapi::um::wingdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectA, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD, SRCCOPY};
use std::env;
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};

// use ffmpeg_next as ffmpeg;
// use ffmpeg::{codec, format, software::scaling::Context as Scaler, util::frame::video::Video};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

pub fn capture_screenshot() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    use std::ptr::null_mut;
    use std::mem::zeroed;

    unsafe {
        let hdc_screen = GetDC(GetDesktopWindow());
        let hdc_mem = CreateCompatibleDC(hdc_screen);

        let width = 1920;
        let height = 1080;

        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        SelectObject(hdc_mem, hbitmap as *mut winapi::ctypes::c_void);

        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);

        let mut bmp = zeroed::<BITMAP>();
        GetObjectA(hbitmap as HGDIOBJ, std::mem::size_of::<BITMAP>() as _, &mut bmp as *mut _ as _);

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negative to indicate a top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }; 1],
        };

        let buf_size = (bmp.bmWidth * bmp.bmHeight * 4) as usize;
        let mut buffer = Vec::with_capacity(buf_size);
        buffer.set_len(buf_size);

        GetDIBits(hdc_mem, hbitmap, 0, height as u32, buffer.as_mut_ptr() as *mut _, &mut bmp_info, DIB_RGB_COLORS);

        DeleteObject(hbitmap as _);
        DeleteDC(hdc_mem);
        ReleaseDC(GetDesktopWindow(), hdc_screen);

        ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap()
    }
}
fn main() -> Result<(),  Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "caster" => {
                let mut fps_counter = 0;
                let mut last_fps_time = std::time::Instant::now();
                loop {
                    let mut screenshot = capture_screenshot();
                    send_screenshot(&mut screenshot);
                    fps_counter += 1;
                    let elapsed = last_fps_time.elapsed();
                    if elapsed >= std::time::Duration::from_secs(1) {
                        let fps = fps_counter as f64 / elapsed.as_secs_f64();
                        println!("FPS: {:.2}", fps);
                        fps_counter = 0;
                        last_fps_time = std::time::Instant::now();
                    }
                }
            }
            "receiver" => {
                println!("Receiver mode");
                env_logger::init();
                let event_loop = EventLoop::new();
                let mut input = WinitInputHelper::new();

                let window = {
                    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
                    WindowBuilder::new()
                        .with_title("Hello Pixels")
                        .with_inner_size(size)
                        .with_min_inner_size(size)
                        .build(&event_loop)
                        .unwrap()
                };

                let mut pixels = {
                    let window_size = window.inner_size();
                    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
                    Pixels::new(WIDTH, HEIGHT, surface_texture)?
                };

                let screenshot = Arc::new(Mutex::new(ImageBuffer::new(WIDTH, HEIGHT)));
                let to_redraw = Arc::new(Mutex::new(false));
                let screenshot_clone = Arc::clone(&screenshot);
                let to_redraw_clone = Arc::clone(&to_redraw);

                // Spawn a thread to receive screenshots
                thread::spawn(move || {
                    loop {
                        let new_screenshot = receive_screenshot(WIDTH, HEIGHT).unwrap();
                        let mut screenshot = screenshot_clone.lock().unwrap();
                        *screenshot = new_screenshot;
                        let mut to_redraw = to_redraw_clone.lock().unwrap();
                        *to_redraw = true;
                    }
                });

                let mut fps_counter = 0;
                let mut last_fps_time = std::time::Instant::now();

                event_loop.run(move |event, _, control_flow| {
                    if let Event::RedrawRequested(_) = event {
                        let screenshot = screenshot.lock().unwrap();
                        // let frame = pixels.frame_mut();
                        // frame.copy_from_slice(&screenshot);
                        // pixels.render().expect("Failed to render pixels");

                        fps_counter += 1;
                        let elapsed = last_fps_time.elapsed();
                        if elapsed >= std::time::Duration::from_secs(1) {
                            let fps = fps_counter as f64 / elapsed.as_secs_f64();
                            println!("FPS: {:.2}", fps);
                            fps_counter = 0;
                            last_fps_time = std::time::Instant::now();
                        }
                    }

                    // Request the window be redrawn
                    let mut to_redraw = to_redraw.lock().unwrap();
                    if *to_redraw {
                        window.request_redraw();
                        *to_redraw = false;
                    }

                    // Handle input events
                    if input.update(&event) {
                        if input.close_requested() {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                });
            }
            _ => {
                println!("Invalid mode");
                return Ok(());
            }
        }
    } else {
        println!("No mode specified");
        return Ok(());
    }

    Ok(())
}

// /// Function to encode a screenshot using ffmpeg
// fn encode(screenshot: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<u8>, Box<dyn Error>> {
//     ffmpeg::init().unwrap();

//     let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::MPEG4).unwrap();
//     let mut context = ffmpeg::format::output_as("test.mp4", "mp4").unwrap();
//     let global_header = context.format().flags().contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER);

//     let mut stream = context.add_stream(codec).unwrap();
//     {
//         let mut encoder = stream.codec().encoder().video().unwrap();
//         encoder.set_width(WIDTH);
//         encoder.set_height(HEIGHT);
//         encoder.set_format(ffmpeg::format::Pixel::RGB24);
//         encoder.set_time_base((1, 25));
//         if global_header {
//             encoder.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
//         }
//     }
//     context.write_header().unwrap();

//     let mut scaler = Scaler::get(
//         ffmpeg::SoftwareScaleAlgorithm::Lanczos3,
//         WIDTH,
//         HEIGHT,
//         ffmpeg::format::Pixel::RGBA,
//         WIDTH,
//         HEIGHT,
//         ffmpeg::format::Pixel::RGB24,
//     ).unwrap();
//     let mut frame = Video::empty();

//     for frame_index in 0..60 {
//         // Create frame from the screenshot
//         frame.set_width(WIDTH);
//         frame.set_height(HEIGHT);
//         frame.set_format(ffmpeg::format::Pixel::RGB24);
//         scaler.run(screenshot, &mut frame).unwrap();

//         let mut output = vec![0; frame.encoded().unwrap().len()];
//         {
//             let mut packet = ffmpeg::util::packet::Packet::empty();
//             stream.codec().encoder().video().unwrap()
//                 .encode(&frame, &mut packet)
//                 .unwrap();

//             packet.rescale_ts(stream.time_base(), context.time_base());
//             packet.write(&mut context).unwrap();
//         }
//         context.write_frame(&stream, &frame, &mut output).unwrap();
//     }
//     Ok(context.io().as_vec().to_vec())
// }

// /// Function to decode a screenshot using ffmpeg
// fn decode(encoded_data: &[u8]) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
//     ffmpeg::init().unwrap();

//     let mut context = ffmpeg::format::input(&encoded_data[..]).unwrap();
//     let mut decoder = context.stream(0).unwrap().codec().decoder().video().unwrap();

//     let mut scaler = Scaler::get(
//         ffmpeg::SoftwareScaleAlgorithm::Lanczos3,
//         decoder.width(),
//         decoder.height(),
//         decoder.format(),
//         WIDTH,
//         HEIGHT,
//         ffmpeg::format::Pixel::RGBA,
//     ).unwrap();

//     let mut frame = Video::empty();
//     let mut output_frame = Video::empty();
//     let mut packet = ffmpeg::util::packet::Packet::empty();

//     packet.read(context.io().as_ref()).unwrap();
//     decoder.decode(&packet, &mut frame).unwrap();
//     scaler.run(&frame, &mut output_frame).unwrap();

//     let buffer: Vec<u8> = output_frame.data(0).to_vec();
//     Ok(ImageBuffer::from_raw(WIDTH, HEIGHT, buffer).unwrap())
// }

// Function to send a screenshot over TCP
fn send_screenshot(screenshot: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> io::Result<()> {
    // Create a TCP stream and connect to the server
    let mut stream = TcpStream::connect("192.168.37.45:7878")?;

    // Convert the image buffer to a byte array
    let frame_bytes = screenshot.clone().into_raw();

    // Send the start message
    stream.write_all(b"START_PHOTO")?;

    // Send the image data in chunks of size 1024 bytes
    let chunk_size = 1024;
    for chunk in frame_bytes.chunks(chunk_size) {
        stream.write_all(chunk)?;
    }

    // Send the end message
    //stream.write_all(b"END_PHOTO")?;


    Ok(())
}

// Function to receive a screenshot over TCP
fn receive_screenshot(width: u32, height: u32) -> io::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    // Create a TCP listener and bind to port 3000
    let listener = TcpListener::bind("127.0.0.1:3000")?;

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


    // Create an image buffer from the received data
    let received_image = ImageBuffer::from_raw(width, height, buffer)
        .expect("Failed to create image buffer from received data");

    Ok(received_image)
}




fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}


// use image::{ImageBuffer, Rgba};
// use std::net::UdpSocket;
// use std::io::{self, Write};

// // Function to send a frame to the server
// fn send_frame(socket: &UdpSocket, server_addr: &str, frame: ImageBuffer<Rgba<u8>, Vec<u8>>) -> io::Result<()> {
//     // Convert the image buffer to a byte array
//     let frame_bytes = frame.into_raw();

//     // Send the frame to the server
//     socket.send_to(&frame_bytes, server_addr)?;

//     Ok(())
// }

// // Function to receive a response from the server
// fn receive_response(socket: &UdpSocket) -> io::Result<()> {
//     // Buffer to store received data
//     let mut buffer = [0u8; 1024];

//     // Receive response from the server
//     let (size, src_addr) = socket.recv_from(&mut buffer)?;

//     // Print the response from the server
//     println!(
//         "Received response from {}: {}",
//         src_addr,
//         String::from_utf8_lossy(&buffer[..size])
//     );

//     Ok(())
// }

// fn main() {
//     // Create a UDP socket and bind to any available port
//     let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
//     let server_addr = "127.0.0.1:8000";

//     // Create a test image buffer (replace with actual image buffer)
//     let width = 256;
//     let height = 256;
//     let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
//     for (x, y, pixel) in img.enumerate_pixels_mut() {
//         *pixel = Rgba([x as u8, y as u8, (x + y) as u8, 255]);
//     }

//     // Send and receive frame
//     send_frame(&socket, server_addr, img).expect("Failed to send frame");
//     receive_response(&socket).expect("Failed to receive response");
// }
}