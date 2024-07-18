use std::{borrow::{Borrow, BorrowMut}, fs::{self, File}, io::Write, path::Path, process::{Command, Output}};

use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::{BGRAFrame, Frame, FrameType, YUVFrame},
};

use std::io::{self};

use std::time::Duration;

use crossbeam_channel::bounded;
use std::thread;

use image::{self, buffer, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use openh264::{decoder::Decoder, encoder::Encoder, formats::{RGBSource, RgbSliceU8, YUVBuffer, YUVSource}};
use std::time::Instant;



extern crate winapi;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::env;
use std::net::{TcpListener, TcpStream};
use std::io::{ Read};
use std::sync::{Arc, Mutex};

// use ffmpeg_next as ffmpeg;
// use ffmpeg::{codec, format, software::scaling::Context as Scaler, util::frame::video::Video};

const WIDTH: u32 = 500;
const HEIGHT: u32 = 1000;
const BOX_SIZE: i16 = 64;

// use ac_ffmpeg::codec::{Context as CodecContext, video_encoder::VideoFrame};
// use ac_ffmpeg::format::output::{Output, Stream};
// use ac_ffmpeg::format::demuxer;
// use ac_ffmpeg::time::Timestamp;
// use ac_ffmpeg::Encoder;
// use ac_ffmpeg::Packet;


fn setRecorder() -> Capturer{

    let targets = scap::get_targets();
    println!("🎯 Targets: {:?}", targets);

    // #4 Create Options
    let options = Options {
        fps: 10,
        targets,
        show_cursor: true,
        show_highlight: false,
        excluded_targets: None,
        output_type: FrameType::YUVFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        source_rect: Some(Area {
            origin: Point { x: 500.0, y: 0.0 },
            size: Size {
                width: 500.0,
                height: 1000.0,
            },
        }),
        ..Default::default()
    };

    // #5 Create Recorder
    let mut recorder = Capturer::new(options);

    // #6 Start Capture
    recorder.start_capture();

    recorder
}

fn loopRecorder( recorder : Capturer, screenshot_clone: Arc<Mutex<BGRAFrame>>){

    let mut fps_counter = 0;
    let mut last_fps_time = std::time::Instant::now();
    loop {
        let mut frames:Vec<BGRAFrame> = Vec::new();
        frames.clear();
        for i in 0..20 {
            let frame = recorder.get_next_frame().expect("Error");

            match frame {
                Frame::YUVFrame(frame) => {

                    println!(
                        "Recieved YUV frame {} of width {} and height {} and pts {}",
                        i, frame.width, frame.height, frame.display_time
                    );
                }
                Frame::BGR0(frame) => {
                    println!(
                        "Received BGR0 frame of width {} and height {}",
                        frame.width, frame.height
                    );
                }
                Frame::RGB(frame) => {

                    println!(
                        "Recieved RGB frame of width {} and height {}",
                        frame.width, frame.height
                    );
                }
                Frame::RGBx(frame) => {
                    println!(
                        "Recieved RGBx frame of width {} and height {}",
                        frame.width, frame.height
                    );
                }
                Frame::XBGR(frame) => {
                    println!(
                        "Recieved xRGB frame of width {} and height {}",
                        frame.width, frame.height
                    );
                }
                Frame::BGRx(frame) => {
                    println!(
                        "Recieved BGRx frame of width {} and height {}",
                        frame.width, frame.height
                    );
                }
                Frame::BGRA(frame) => {

                    let mut screenshot_clone=screenshot_clone.lock().unwrap();
                    *screenshot_clone = frame;



                    fps_counter += 1;
                    let elapsed = last_fps_time.elapsed();
                    if elapsed >= std::time::Duration::from_secs(1) {
                        let fps = fps_counter as f64 / elapsed.as_secs_f64();
                        println!("FPS: {:.2}", fps);
                        fps_counter = 0;
                        last_fps_time = std::time::Instant::now();
                    }
                    // println!("frame {}", 2000*1000*4);

                    // }
                    // println!(
                    //     "Recieved BGRA frame {} of width {} and height {} ",
                    //     i,
                    //     frame.width,
                    //     frame.height
                    // );
                    // Save frame to a folder


                }
            }
        }
        
    }
}

fn main() -> Result<(),  Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "caster" => {
                let screenshot_frames: Arc<Mutex<BGRAFrame>> = Arc::new(Mutex::new(BGRAFrame{width: 0, display_time:0, height: 0, data: vec![]}));
                let screenshot_frames_clone = screenshot_frames.clone();
                // let screenshot_to_send = Arc::new(Mutex::new(true));
                // let screenshot_to_send_clone = screenshot_to_send.clone();

                let recorder = setRecorder();
                std::thread::spawn(move || {


                    thread::sleep(Duration::from_secs(2));
                    let connection = get_connection_connect("localhost:7878".to_string()).unwrap();
                    loop {
                        let screenshot_framex = screenshot_frames.lock().unwrap();
                        let screenshot_frame = screenshot_framex.clone();
                        drop(screenshot_framex);
                        // print!("Creating video from frames {} {}", screenshot_frame.data.len(), 2000*1000*4);
                        // let base_path = "./frames/";
                        // print!("Saving frames to {:?}", scap::capturer::Resolution::_720p.);
                        // save_frames_as_images(frames, base_path);

                        //
                        //create_video_from_images(base_path, 2000, 1000, "output_video.mp4");
                        // extract_images_from_video(&format!("{}{}", base_path, "output_video.mp4"), "miao.png", 30) ;
                        // let output_path = "./out.png";
                        // buffer.save(output_path).expect("Failed to save image");
                        //to buffer rgb
                        //compress a  2000 x 1000 vec u8 to a 800x400 vec u8
                        
                        let buffer_image= ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(WIDTH, HEIGHT, screenshot_frame.data.clone()).unwrap();
                        // let compressed_image = image::imageops::resize(&buffer_image, WIDTH, HEIGHT, image::imageops::FilterType::Lanczos3);
                        let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(WIDTH, HEIGHT, |x, y| {
                            let pixel = buffer_image.get_pixel(x, y);
                            image::Rgb([pixel[0], pixel[1], pixel[2]])
                        });

                        // let mut screenshot_clone=screenshot_clone.lock().unwrap();
                        // *screenshot_clone = rgb_img;
                        if screenshot_frame.width> 10 {
                            let (width, height, mut encoded_frames, encode_duration) = encode(&rgb_img);
                            send_screenshot(&mut encoded_frames, connection.borrow());
                            // print!("sent");

                        }

                        // let base_path = "./frames/";

                        // let mut screenshot = fs::read(format!("{}output_video.mp4", base_path));

                    }
                });
                loopRecorder(recorder,screenshot_frames_clone);
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


                let screenshot = Arc::new(Mutex::new(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(WIDTH as u32, HEIGHT as u32)));
                let to_redraw = Arc::new(Mutex::new(false));
                let screenshot_clone = screenshot.clone();                
                let screenshot_clone1 = screenshot.clone();

                let to_redraw_clone = Arc::clone(&to_redraw);

                // Spawn a thread to receive screenshots
                let mut stream = get_connection_listener("localhost:7878".to_string()).unwrap();
                spawn_screenshot_thread(screenshot_clone, to_redraw_clone, stream);

                let mut fps_counter = 0;
                let mut last_fps_time = std::time::Instant::now();
                let mut i = 0;
                event_loop.run(move | event, _, control_flow| {
                    if let Event::RedrawRequested(_) = event {
                        get_frame(screenshot_clone1.clone(), &mut pixels);
                        pixels.render().expect("Failed to render pixels");
                        i+=1;
                        // println!("{}", i);
                        fps_counter += 1;
                        let elapsed = last_fps_time.elapsed();
                        if elapsed >= std::time::Duration::from_secs(1) {
                            let fps = fps_counter as f64 / elapsed.as_secs_f64();
                            println!("FPS MIAO: {:.2}", fps);
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


fn decode(encoded_frames: Vec<u8>, width: u32, height: u32) -> (std::time::Duration, ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let mut decoder = Decoder::new().expect("Failed to create decoder");

    let start_decode = Instant::now();
    let start_decode2 = Instant::now();
    
    let decoded_frame = decoder.decode(&encoded_frames).expect("Failed to decode frame");
    let decode_duration = start_decode.elapsed();
    let mut rgba_buffer = vec![0u8; (width * height * 4) as usize];
    if(decoded_frame.is_none()){
        return (decode_duration, ImageBuffer::<Rgba<u8>, Vec<u8>>::new(5, height));
    }
    decoded_frame.unwrap().write_rgba8(&mut rgba_buffer);
    
    
    
    // Convert the decoded frame to an ImageBuffer
    let out_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height,  rgba_buffer)
    .expect("Failed to create image buffer");

    let decode_duration2 = start_decode2.elapsed();
    // println!("decode {}",decode_duration2.as_millis());
    (decode_duration, out_img)
}

fn encode<'a>(rgb_img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> (u32, u32, Vec<u8>, std::time::Duration) {
    let start_encode = Instant::now();
    let (width, height) = rgb_img.dimensions();
    let yuv_buffer = YUVBuffer::from_rgb_source( RgbSliceU8::new(&rgb_img, (width as usize, height as usize)));
    // Step 1.5: Convert the RGBA image to YUV
    // print!("dimensions: {}\n", yuv_buffer.u().len()*3);
    
    
    // Step 2: Encode the image into video frames
    let mut encoder = Encoder::new().unwrap();

    let encode_duration = start_encode.elapsed();
    let encoded_frames = encoder.encode(&yuv_buffer).expect("Failed to encode frame").to_vec();
    // print!("dimensions: {}\n",  encoded_frames.to_vec().len());

    // print!("encode {}",encode_duration.as_millis());
    (width, height, encoded_frames, encode_duration)
}



fn get_frame(screenshot: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, pixels: &mut Pixels)  {
    let screenshot = screenshot.lock().unwrap();

    let base_path = "./frames2/";
    // let new_frame = *screenshot.clone() ;
    let new_frame: ImageBuffer<Rgba<u8>, Vec<u8>> = screenshot.clone();

    let mut frame = pixels.frame_mut();

    frame.copy_from_slice(&new_frame);
}

fn spawn_screenshot_thread(screenshot_clone: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, to_redraw_clone: Arc<Mutex<bool>>, stream:TcpStream) {
    let (sender, receiver) = crossbeam_channel::bounded(1);

    let receiver_thread = thread::spawn(move || {
        let mut fps_counter = 0;
        let mut last_fps_time = std::time::Instant::now();
        let mut i = 0;
                
        let stream1= stream.borrow();  
        loop {
            if let Ok(new_screenshot) = receive_screenshot(WIDTH, HEIGHT, stream1) {
                // println!("Received screenshot of size: {}", new_screenshot.len());
                sender.send(new_screenshot).unwrap();
                fps_counter+=1;
                // println!("{}", i);
                let elapsed = last_fps_time.elapsed().as_secs_f64();
                if elapsed >= std::time::Duration::from_secs(1).as_secs_f64() {
                    let fps: f64 = fps_counter as f64 / elapsed;
                    println!("FPS MIAO: {:.2}", fps);
                    fps_counter = 0;
                    last_fps_time = std::time::Instant::now();
                }
            }
        }
    });

    let decoder_thread = thread::spawn(move || {
        loop {
            if let Ok(new_screenshot) = receiver.recv() {
                let (decode_duration, out_img) = decode(new_screenshot, WIDTH, HEIGHT);
                
                if out_img.width() != 5 {
                    let mut screenshot = screenshot_clone.lock().unwrap();
                    *screenshot = out_img;

                    let mut to_redraw = to_redraw_clone.lock().unwrap();
                    *to_redraw = true;

                    drop(screenshot);
                    drop(to_redraw);

                    // println!("Decoded frame in {} ms", decode_duration.as_millis());
                }
            }
        }
    });

    // receiver_thread.join().unwrap();
    // decoder_thread.join().unwrap();
}

fn get_connection_connect(address:String) -> io::Result<TcpStream>{
    let mut stream = TcpStream::connect(address);
    stream

}

fn send_screenshot(screenshot: &mut  Vec<u8>, mut stream:&TcpStream) -> io::Result<()> {
    // Create a TCP stream and connect to the server

    // Convert the image buffer to a byte array
    let frame_bytes = screenshot.clone();

    // Send the start message
    stream.write_all(b"START_PHOTO")?;
    // println!("Sending screenshot of size: {}", frame_bytes.len());

    // Send the image data in chunks of size 1024 bytes
    let chunk_size = 1024;

    for chunk in frame_bytes.chunks(chunk_size) {
        stream.write_all(chunk)?;
    }
    

    // Send the end message
    stream.write_all(b"END_PHOTO")?;


    Ok(())
}

fn get_connection_listener(address:String) -> io::Result<TcpStream> {
    let listener = TcpListener::bind("localhost:7878")?;

    // Wait for a connection
    let (mut stream, _addr) = listener.accept()?;

    return Ok(stream);
}

// Function to receive a screenshot over TCP
fn receive_screenshot(width: u32, height: u32, mut stream:&TcpStream) -> io::Result<Vec<u8>> {
    // Create a TCP listener and bind to port 3000
    

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




    // println!("Sending screenshot of size: {}", buffer.len());
    // Create an image buffer from the received data
    let received_image =buffer;

    Ok(received_image)
}
pub fn save_frames_as_images(frames: Vec<BGRAFrame>, base_path: &str) -> io::Result<()> {
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("{}frame-{:04}.raw", base_path, i);
        let mut file = File::create(&filename)?;
        file.write_all(&frame.data)?;
    }
    Ok(())
}

// pub fn create_video_from_images(base_path: &str, width: i32, height: i32, output_filename: &str) -> io::Result<()> {
//     // if !Path::new(base_path).exists() {
//     //     return Err(io::Error::new(io::ErrorKind::NotFound, "Base path does not exist"));
//     // }

//     let output = Command::new("ffmpeg")
//         .current_dir(base_path)
//         .arg("-pixel_format")
//         .arg("bgra")
//         .arg("-video_size")
//         .arg(format!("{}x{}", width, height))
//         .arg("-framerate")
//         .arg("30")
//         .arg("-start_number")
//         .arg("0")
//         .arg("-i")
//         .arg(".\\frame-%04d.raw")
//         .arg("-c:v")
//         .arg("libx264")
//         .arg("-y") // Overwrite output file if it exists
//         .arg(output_filename)
//         .output()?;

//     if !output.status.success() {
//         eprintln!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr));
//     }

//     Ok(())
// }

pub fn png_to_bgra_frame<P: AsRef<Path>>(path: P) -> io::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let img = image::open(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    // let img = img.to_rgba8();
    // let (width, height) = img.dimensions();

    // let mut data = img.into_raw();
    // print!("Data length: {}", data.len());
    // Ok(img)

    Ok(img.to_rgba8())
}


// pub fn extract_images_from_video(video_path: &str, output_path: &str, frame_rate: i32) -> io::Result<()> {
//     // Check if the video file exists
//     if !Path::new(video_path).exists() {
//         return Err(io::Error::new(io::ErrorKind::NotFound, "Video file does not exist"));
//     }

//     // Create the output directory if it doesn't exist
//     if !Path::new(output_path).exists() {
//         fs::create_dir_all(output_path)?;
//     }

//     // Run ffmpeg command to extract images
//     let output: Output = Command::new("ffmpeg")
//         .arg("-i")
//         .arg(video_path)
//         .arg("-vf")
//         .arg(format!("fps={}", frame_rate))
//         .arg(format!("{}/frame-%04d.png", output_path))
//         .arg("-y")
//         .arg("-compression_level")
//         .arg("10000")
//         .output()?;

//     if !output.status.success() {
//         eprintln!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr));
//         return Err(io::Error::new(io::ErrorKind::Other, "FFmpeg command failed"));
//     }

//     Ok(())
// }
