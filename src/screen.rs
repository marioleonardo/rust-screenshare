mod net;
mod encoder;
mod decoder;
mod capture;
pub mod screen{
    
    

    use std::{default, fs::{self, File}, io::Write, path::Path, process::{Command, Output}};

use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::{BGRAFrame, Frame, FrameType, YUVFrame},
};

use std::io::{self};

use std::time::Duration;
use std::thread;

use image::{self, DynamicImage, GenericImageView, ImageBuffer, Rgba};




extern crate winapi;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::env;


use std::sync::{Arc, Mutex};

use super::net::net::{receive_screenshot,send_screenshot};
use super::capture::capture::{loopRecorder,setRecorder};
use super::decoder::decoder::decode;
use super::encoder::encoder::encode;


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

pub struct screen_state{
    frame : Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    to_redraw: Arc<Mutex<bool>>,
}
impl Default for screen_state{
    fn default() -> Self {
        Self { frame: Arc::new(Mutex::new(ImageBuffer::default())),
            to_redraw: Arc::new(Mutex::new(bool::default())),
         }
    }
}

impl screen_state {
    pub fn get_frame(&mut self)->ImageBuffer<Rgba<u8>, Vec<u8>>{
        let f = self.frame.lock().unwrap();
        f.clone()
    }

    pub fn set_frame(&mut self,frame:ImageBuffer<Rgba<u8>, Vec<u8>>){
        let mut f=self.frame.lock().unwrap();
        let mut a = self.to_redraw.lock().unwrap();
        *a=true;
        *f=frame;
    }
}

pub fn loop_logic(args:Vec<String>,mut state:screen_state) -> Result<(),  Error> {
    
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
                        let buffer_image= ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(2000, 1000, screenshot_frame.data.clone()).unwrap();
                        let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 1000, |x, y| {
                            let pixel = buffer_image.get_pixel(x, y);
                            image::Rgb([pixel[0], pixel[1], pixel[2]])
                        });
                        // let mut screenshot_clone=screenshot_clone.lock().unwrap();
                        // *screenshot_clone = rgb_img;
                        if screenshot_frame.width> 10 {
                            let (width, height, mut encoded_frames, encode_duration) = encode(&rgb_img);
                            send_screenshot(&mut encoded_frames,"localhost:7878".to_string());
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
                /*
                let mut pixels = {
                    let window_size = window.inner_size();
                    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
                    Pixels::new(WIDTH, HEIGHT, surface_texture)?
                };*/


                let screenshot = Arc::new(Mutex::new(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(WIDTH as u32, HEIGHT as u32)));
                let to_redraw = Arc::new(Mutex::new(false));
                let screenshot_clone = screenshot.clone();                
                let screenshot_clone1 = screenshot.clone();

                let to_redraw_clone = Arc::clone(&to_redraw);

                // Spawn a thread to receive screenshots
                spawn_screenshot_thread(screenshot_clone, to_redraw_clone);

                let mut fps_counter = 0;
                let mut last_fps_time = std::time::Instant::now();
                let mut i = 0;
                // event_loop.run(move | event, _, control_flow| {
                //     if let Event::RedrawRequested(_) = event {
                        
                //         pixels.render().expect("Failed to render pixels");
                //         i+=1;
                //         // println!("{}", i);
                //         fps_counter += 1;
                //         let elapsed = last_fps_time.elapsed();
                //         if elapsed >= std::time::Duration::from_secs(1) {
                //             let fps = fps_counter as f64 / elapsed.as_secs_f64();
                //             println!("FPS: {:.2}", fps);
                //             fps_counter = 0;
                //             last_fps_time = std::time::Instant::now();
                //         }
                //     }

                //     // Request the window be redrawn
                    
                //     // Handle input events
                //     if input.update(&event) {
                //         if input.close_requested() {
                //             *control_flow = ControlFlow::Exit;
                //         }
                //     }
                // });
                loop{
                    let mut to_redraw = to_redraw.lock().unwrap();
                    if *to_redraw {
                        let frame = get_frame(screenshot_clone1.clone());
                        state.set_frame(frame);

                        *to_redraw = false;
                    }

                }
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

fn get_frame(screenshot: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let screenshot = screenshot.lock().unwrap();

    let base_path = "./frames2/";
    // let new_frame = *screenshot.clone() ;
    let new_frame: ImageBuffer<Rgba<u8>, Vec<u8>> = screenshot.clone();

    // let mut frame = pixels.frame_mut();

    // frame.copy_from_slice(&new_frame);
    return new_frame;


}

fn spawn_screenshot_thread(screenshot_clone: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, to_redraw_clone: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        loop {
            let new_screenshot = receive_screenshot(WIDTH, HEIGHT, "localhost:7878".to_string()).unwrap();
            // let base_path = "./frames2/";
            // let output_path: String = format!("{}output_video.mp4", base_path);
            // if let Ok(mut screenshot_file) = fs::File::create(&output_path) {
            //     screenshot_file.write_all(&new_screenshot).unwrap();

            // } else {
            //     println!("Failed to create screenshot file");
            // }
            // extract_images_from_video(&format!("{}{}", base_path, "output_video.mp4"), "miao.png", 30) ;


            let (decode_duration, out_img) =decode(new_screenshot, 2000, 1000);
            if(out_img.width()!=5){
                        // out_img.save("./mo.png").expect("Failed to save image");
            // for i in 1..11 {
                // let file_path = format!("miao.png/frame-{:04}.png", i);
                // let new_frame = png_to_bgra_frame(file_path).unwrap();
            // print!("maio");
            let mut screenshot = screenshot_clone.lock().unwrap();
            *screenshot = out_img;
            let mut to_redraw = to_redraw_clone.lock().unwrap();
            *to_redraw = true;
            }
            
            // }
            // println!("Received videos of size: {}", new_screenshot.len());

        }
    });
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

}