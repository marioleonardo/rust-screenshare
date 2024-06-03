use std::{fs::{self, File}, io::Write, path::Path, process::{Command, Output}};

use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::{BGRAFrame, Frame, FrameType, YUVFrame},
};

use std::io::{self};

use std::time::Duration;
use std::thread;

use image::{self, DynamicImage, GenericImageView, ImageBuffer, Rgba};
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

const WIDTH: u32 = 2000;
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
        fps: 2,
        targets,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        output_type: FrameType::YUVFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        source_rect: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 2000.0,
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

fn loopRecorder( recorder : Capturer, screenshot_clone: Arc<Mutex<ImageBuffer<image::Rgb<u8>, Vec<u8>>>>){

    let mut fps_counter = 0;
    let mut last_fps_time = std::time::Instant::now();
    loop {
        let mut frames:Vec<BGRAFrame> = Vec::new();
        frames.clear();
        for i in 0..200 {
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
                    if frames.len() == 0 {
                        frames.push(frame.clone());
                    }
                    if i==0{

                        print!("Creating video from frames {} {}", frames[0].data.len(), 2000*1000*4);
                        frames[0]=frame.clone();
                        let base_path = "./frames/";
                        // print!("Saving frames to {:?}", scap::capturer::Resolution::_720p.);
                        // save_frames_as_images(frames, base_path);

                        //
                        //create_video_from_images(base_path, 2000, 1000, "output_video.mp4");
                        // extract_images_from_video(&format!("{}{}", base_path, "output_video.mp4"), "miao.png", 30) ;
                        let buffer_image= ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(2000, 1000, frames[0].data.clone()).unwrap();
                        // let output_path = "./out.png";
                        // buffer.save(output_path).expect("Failed to save image");
                        //to buffer rgb
                        let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 1000, |x, y| {
                            let pixel = buffer_image.get_pixel(x, y);
                            image::Rgb([pixel[0], pixel[1], pixel[2]])
                        });
                        let mut screenshot_clone=screenshot_clone.lock().unwrap();
                        *screenshot_clone = rgb_img;


                        fps_counter += 1;
                        let elapsed = last_fps_time.elapsed();
                        if elapsed >= std::time::Duration::from_secs(1) {
                            let fps = fps_counter as f64 / elapsed.as_secs_f64();
                            println!("FPS: {:.2}", fps);
                            fps_counter = 0;
                            last_fps_time = std::time::Instant::now();
                        }
                        print!("Creating video from frames {} {}", frames[0].data.len(), 2000*1000*4);

                    }
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
                let screenshot_frames: Arc<Mutex<ImageBuffer<image::Rgb<u8>, Vec<u8>>>> = Arc::new(Mutex::new((ImageBuffer::new(0, 0))));
                let screenshot_frames_clone = screenshot_frames.clone();
                let recorder = setRecorder();
                std::thread::spawn(move || {


                    thread::sleep(Duration::from_secs(2));
                    loop {
                        let screenshot_frame = screenshot_frames.lock().unwrap();
                        if screenshot_frame.width()> 10 {
                            let (width, height, mut encoded_frames, encode_duration) = encode(&screenshot_frame);
                            send_screenshot(&mut encoded_frames);
                            print!("sent");

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
                let screenshot_clone = Arc::clone(&screenshot);
                let to_redraw_clone = Arc::clone(&to_redraw);

                // Spawn a thread to receive screenshots
                spawn_screenshot_thread(screenshot_clone, to_redraw_clone);

                let mut fps_counter = 0;
                let mut last_fps_time = std::time::Instant::now();

                event_loop.run(move |event, _, control_flow| {
                    if let Event::RedrawRequested(_) = event {
                        let screenshot: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>> = Arc::clone(&screenshot);
                        get_frame(screenshot, &mut pixels);
                        pixels.render().expect("Failed to render pixels");

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


fn decode(encoded_frames: Vec<u8>, width: u32, height: u32) -> (std::time::Duration, ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let mut decoder = Decoder::new().expect("Failed to create decoder");

    let start_decode = Instant::now();
    
    let decoded_frame = decoder.decode(&encoded_frames).expect("Failed to decode frame");
    let decode_duration = start_decode.elapsed();
    let mut rgba_buffer = vec![0u8; (width * height * 4) as usize];
    decoded_frame.unwrap().write_rgba8(&mut rgba_buffer);


    // Convert the decoded frame to an ImageBuffer
    let out_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height,  rgba_buffer)
        .expect("Failed to create image buffer");
    
    (decode_duration, out_img)
}

fn encode<'a>(rgb_img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> (u32, u32, Vec<u8>, std::time::Duration) {
    
    let (width, height) = rgb_img.dimensions();
    let yuv_buffer = YUVBuffer::from_rgb_source( RgbSliceU8::new(&rgb_img, (width as usize, height as usize)));
    // Step 1.5: Convert the RGBA image to YUV
    // print!("dimensions: {}\n", yuv_buffer.u().len()*3);

    
    // Step 2: Encode the image into video frames
    let mut encoder = Encoder::new().unwrap();

    let start_encode = Instant::now();
    let encoded_frames = encoder.encode(&yuv_buffer).expect("Failed to encode frame").to_vec();
    // print!("dimensions: {}\n",  encoded_frames.to_vec().len());

    let encode_duration = start_encode.elapsed();
    (width, height, encoded_frames, encode_duration)
}



fn get_frame(screenshot: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, pixels: &mut Pixels)  {
    let screenshot = screenshot.lock().unwrap();

    let base_path = "./frames2/";
    let new_frame =screenshot ;


    let mut frame = pixels.frame_mut();
    let frame2 =  new_frame;

    frame.copy_from_slice(&frame2);


}

fn spawn_screenshot_thread(screenshot_clone: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, to_redraw_clone: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        loop {
            let new_screenshot = receive_screenshot(WIDTH, HEIGHT).unwrap();
            // let base_path = "./frames2/";
            // let output_path: String = format!("{}output_video.mp4", base_path);
            // if let Ok(mut screenshot_file) = fs::File::create(&output_path) {
            //     screenshot_file.write_all(&new_screenshot).unwrap();

            // } else {
            //     println!("Failed to create screenshot file");
            // }
            // extract_images_from_video(&format!("{}{}", base_path, "output_video.mp4"), "miao.png", 30) ;
            let (decode_duration, out_img) =decode(new_screenshot, 2000, 1000);

            // for i in 1..11 {
                // let file_path = format!("miao.png/frame-{:04}.png", i);
                // let new_frame = png_to_bgra_frame(file_path).unwrap();
            let mut screenshot = screenshot_clone.lock().unwrap();
            *screenshot = out_img;
            let mut to_redraw = to_redraw_clone.lock().unwrap();
            *to_redraw = true;
            // }
            // println!("Received videos of size: {}", new_screenshot.len());

        }
    });
}


fn send_screenshot(screenshot: &mut  Vec<u8>) -> io::Result<()> {
    // Create a TCP stream and connect to the server
    let mut stream = TcpStream::connect("localhost:7878")?;

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
fn receive_screenshot(width: u32, height: u32) -> io::Result<Vec<u8>> {
    // Create a TCP listener and bind to port 3000
    let listener = TcpListener::bind("localhost:7878")?;

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
