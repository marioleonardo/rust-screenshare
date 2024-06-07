mod net;
mod encoder;
mod decoder;
mod capture;
pub mod screen{
    
    

use std::{fs::File, io::Write, path::Path};

use scap::frame::BGRAFrame;

use std::io::{self};

use std::time::Duration;
use std::thread;

use image::{self, ImageBuffer, Rgba};
extern crate winapi;
use pixels::Error;

use std::sync::{Arc, Mutex};

use super::net::net::{receive_screenshot,send_screenshot};
use super::capture::capture::{loopRecorder,setRecorder};
use super::decoder::decoder::decode;
use super::encoder::encoder::encode;

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1000;
const BOX_SIZE: i16 = 64;

#[derive(Clone)]
pub struct screen_state{
    frame : Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    to_redraw: Arc<Mutex<bool>>,
}
impl Default for screen_state{
    fn default() -> Self {
        Self { frame: Arc::new(Mutex::new(ImageBuffer::new(2000, 1000))),
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

pub fn loop_logic(args:String,mut state:Arc<Mutex<screen_state>>) -> Result<(),  Error> {
    
    if args.len() > 1 {
        match args.as_str() {
            "caster" => {
                let screenshot_frames: Arc<Mutex<BGRAFrame>> = Arc::new(Mutex::new(BGRAFrame{width: 0, display_time:0, height: 0, data: vec![]}));
                let screenshot_frames_clone = screenshot_frames.clone();

                let recorder = setRecorder();
                std::thread::spawn(move || {


                    thread::sleep(Duration::from_secs(2));
                    loop {
                        let screenshot_framex = screenshot_frames.lock().unwrap();
                        let screenshot_frame = screenshot_framex.clone();
                        drop(screenshot_framex);
                        
                        let buffer_image= ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(2000, 1000, screenshot_frame.data.clone()).unwrap();
                        let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 1000, |x, y| {
                            let pixel = buffer_image.get_pixel(x, y);
                            image::Rgb([pixel[0], pixel[1], pixel[2]])
                        });
                        
                        if screenshot_frame.width> 10 {
                            let (_width, _height, mut encoded_frames, _encode_duration) = encode(&rgb_img);
                            let _ = send_screenshot(&mut encoded_frames,"localhost:7878".to_string());
                            
                        }

                    }
                });
                loopRecorder(recorder,screenshot_frames_clone);
            }
            "receiver" => {
                println!("Receiver mode");
                //env_logger::init();

                let screenshot = Arc::new(Mutex::new(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(WIDTH as u32, HEIGHT as u32)));
                let to_redraw = Arc::new(Mutex::new(false));
                let screenshot_clone = screenshot.clone();                
                let screenshot_clone1 = screenshot.clone(); 
                let to_redraw_clone = Arc::clone(&to_redraw);

                // Spawn a thread to receive screenshots
                
                spawn_screenshot_thread(screenshot_clone, to_redraw_clone);
                
                loop{
                    
                    let mut to_redraw = to_redraw.lock().unwrap();
                    if *to_redraw {
                        let mut st = state.lock().unwrap();
                        
                        //println!("sto comunicando");
                        let frame = get_frame(screenshot_clone1.clone());
                        
                        st.set_frame(frame);

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
    
    let new_frame: ImageBuffer<Rgba<u8>, Vec<u8>> = screenshot.clone();
    
    return new_frame;


}

fn spawn_screenshot_thread(screenshot_clone: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, to_redraw_clone: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        loop {
            let new_screenshot = receive_screenshot(WIDTH, HEIGHT, "localhost:7878".to_string()).unwrap();

            let (_decode_duration, out_img) =decode(new_screenshot, 2000, 1000);
            if out_img.width()!=5 {
            
            let mut screenshot = screenshot_clone.lock().unwrap();
            *screenshot = out_img;
            let mut to_redraw = to_redraw_clone.lock().unwrap();
            *to_redraw = true;
            
            }            
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


pub fn png_to_bgra_frame<P: AsRef<Path>>(path: P) -> io::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let img = image::open(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(img.to_rgba8())
}

}