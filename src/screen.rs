pub(crate) mod net;
mod encoder;
mod decoder;
pub(crate) mod capture;
mod videowriter;
pub mod screen{
    
use chrono::prelude::*;
use std::net::TcpStream;
use std::{fs::File, io::Write, path::Path};

use image::imageops::FilterType;
use scap::frame::BGRAFrame;
//use winapi::um::winioctl::Unknown;

use std::io::{self};

use std::time::Duration;
use std::thread::{self, JoinHandle};

use image::{self, DynamicImage, GenericImageView, ImageBuffer, Rgba};
extern crate winapi;
use pixels::Error;

use std::sync::{Arc, Mutex};

use crate::enums::StreamingState;
use crate::screen::capture::capture::getMonitors;
use super::net::net::*;
//use super::net::net::{receive_screenshot,send_screenshot};
use super::capture::capture::{loopRecorder,setRecorder};
use super::decoder::decoder::decode;
use super::encoder::encoder::encode;
use super::videowriter::VideoWriter;

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1000;
const BOX_SIZE: i16 = 64;


pub struct screen_state{
    pub stream_state : Arc<Mutex<StreamingState>>,
    frame : Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    to_redraw: Arc<Mutex<bool>>,
    ip_sender: Arc<Mutex<String>>,
    ip_receiver: Arc<Mutex<String>>,
    x: Arc<Mutex<u32>>,
    y: Arc<Mutex<u32>>,
    red_factor: Arc<Mutex<u32>>,
    pub cv: std::sync::Condvar,
    pub client_stream: Arc<Mutex<Option<(TcpStream,Client)>>>,
    pub server: Arc<Mutex<Option<Server>>>,
    pub recording: Arc<Mutex<Option<bool>>>,
    pub n_monitor: Arc<Mutex<u8>>

}

impl Default for screen_state{
    fn default() -> Self {
        Self { 
            cv: std::sync::Condvar::default(),
            frame: Arc::new(Mutex::new(ImageBuffer::new(2000, 1000))),
            to_redraw: Arc::new(Mutex::new(bool::default())),
            stream_state: Arc::new(Mutex::new(StreamingState::default())),
            ip_sender: Arc::new(Mutex::new(String::default())),
            ip_receiver: Arc::new(Mutex::new(String::default())),
            x: Arc::new(Mutex::new(0)),
            y: Arc::new(Mutex::new(0)),
            red_factor: Arc::new(Mutex::new(100)),
            client_stream: Arc::new(Mutex::new(None)),
            server: Arc::new(Mutex::new(None)),
            recording: Arc::new(Mutex::new(None)),
            n_monitor : Arc::new(Mutex::new(0))
         }
    }
}

impl screen_state {
    pub fn get_frame(&self)->ImageBuffer<Rgba<u8>, Vec<u8>>{
        let f = self.frame.lock().unwrap();
        
        f.clone()
    }

    pub fn get_sc_state(&self)->StreamingState{
        let f = self.stream_state.lock().unwrap();
        
        f.clone()
    }

    pub fn get_n_monitor(&self)->u8{
        let n = self.n_monitor.lock().unwrap();

        n.clone()
    }
    
    pub fn set_n_monitor(&self, nm:u8){
        let mut n = self.n_monitor.lock().unwrap();

        *n=nm;
    }

    pub fn set_frame(&self,frame:ImageBuffer<Rgba<u8>, Vec<u8>>){
        let mut f=self.frame.lock().unwrap();
        
        let mut a = self.to_redraw.lock().unwrap();
        *a=true;
        *f=frame;
    }

    pub fn set_screen_state(&self,sc:StreamingState){
        let mut f=self.stream_state.lock().unwrap();
        *f=sc;
    }

    pub fn set_ip_rec(&self,ip:String){
        let mut f=self.ip_receiver.lock().unwrap();
        *f=ip;
    }

    pub fn set_ip_send(&self,ip:String){
        let mut f=self.ip_sender.lock().unwrap();
        *f=ip;
    }

    pub fn get_ip_sender(&self)->String{
        let ip = self.ip_sender.lock().unwrap();

        return ip.clone();
    }

    pub fn get_ip_receiver(&self)->String{
        let ip = self.ip_receiver.lock().unwrap();

        return ip.clone();
    }

    pub fn get_x(&self)->u32{
        let x = self.x.lock().unwrap();

        return x.clone();
    }
    pub fn get_y(&self)->u32{
        let y = self.y.lock().unwrap();

        return y.clone();
    }
    pub fn get_f(&self)->u32{
        let f = self.red_factor.lock().unwrap();

        return f.clone();
    }

    pub fn set_x(&self,n:u32){
        let mut x = self.x.lock().unwrap();

        *x=n;
    }
    pub fn set_y(&self,n:u32){
        let mut y = self.y.lock().unwrap();

        *y=n;
    }
    pub fn set_f(&self,n:u32){
        let mut f = self.red_factor.lock().unwrap();

        *f=n;
    }

    pub fn set_rec(&self,flag:Option<bool>){
        let mut f = self.recording.lock().unwrap();

        *f=flag;
    }

    pub fn get_rec(&self)->Option<bool>{
        let f = self.recording.lock().unwrap();

        return f.clone();
    }

    pub fn set_server(&self,server:Option<Server>){
        let mut s= self.server.lock().unwrap();

        *s=server;
    }

    pub fn set_client(&self,client:Option<(TcpStream,Client)>){
        let mut c= self.client_stream.lock().unwrap();

        *c=client;
    }

    pub fn send_to_clients(&self,v:Vec<u8>,w:u32,h:u32,state:State)->Result<(()),Error>{
        let s = self.server.lock().unwrap();
        
        if let Some(server) =s.as_ref(){
        let screenshot_data = Screenshot {
            data: v,
            width: w,  // Placeholder width
            height: h, // Placeholder height
            state,
        };
        let res = server.send_to_all_clients(&screenshot_data);
        println!("inviato: {:?}",res)
        }
        Ok(())

    }

    pub fn receive_from_server(&self)-> io::Result<Screenshot>{
        let mut c = self.client_stream.lock().unwrap();
        
        println!("start");
        if let Some(( stream,client)) = c.as_mut(){
            if client.is_connected(&stream) {
                
                if let Ok(screenshot) = client.receive_image_and_struct(stream){
                    
                    //if let State::Receiving = screenshot.state {
                    
                    return Ok(screenshot);
                    //}
                };
                
            }
        }
        println!("error");
        
        Err(io::Error::new(io::ErrorKind::Other, "Failed to receive screenshot"))
    }
}

fn stop_screen(width:u32, height:u32)->ImageBuffer<Rgba<u8>, Vec<u8>>{
    let pixel = Rgba([255u8,0u8,0u8,255u8]);
    let img = ImageBuffer::from_pixel(width,height , pixel);

    return img 
}

fn blanked_screen(width:u32, height:u32)->ImageBuffer<Rgba<u8>, Vec<u8>>{
    let pixel = Rgba([255u8,255u8,255u8,255u8]);
    let img = ImageBuffer::from_pixel(width,height , pixel);

    return img 
}

pub fn loop_logic(args:String,state:Arc<screen_state>) -> Result<(),  Error> {
    
    if args.len() > 1 {
        match args.as_str() {
            "caster" => {      
                println!("Caster mode");
            
                let screenshot_frames: Arc<Mutex<BGRAFrame>> = Arc::new(Mutex::new(BGRAFrame{width: 0, display_time:0, height: 0, data: vec![]}));
                let screenshot_frames_clone = screenshot_frames.clone();
                let monitor = getMonitors();
                let n = state.get_n_monitor();
                let recorder = setRecorder(monitor[n as usize].clone());
                let state_clone = state.clone();
                let a = std::thread::spawn(move || {
                
                thread::sleep(Duration::from_secs(2));
                //let mut paused_img:ImageBuffer::<Rgba<u8>, Vec<u8>>= ImageBuffer::default();
                
                loop {
                    let sc_state= state.get_sc_state();
            
                    match sc_state{
                        StreamingState::START => {
                            let screenshot_framex = screenshot_frames.lock().unwrap();
                            let screenshot_frame = screenshot_framex.clone();
                            drop(screenshot_framex);
                            
                            let buffer_image= ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(screenshot_frame.width as u32, screenshot_frame.height as u32, screenshot_frame.data.clone()).unwrap();
                            
                            //TODO: instead of cropping we should send variable image size and communicate the actual size with the struct sent, and then display a variable size image 
                            let buffer_image= DynamicImage::ImageRgba8(buffer_image).crop_imm(state.get_x(), state.get_y(), screenshot_frame.width as u32*state.get_f()/100, screenshot_frame.height as u32*state.get_f()/100).into_rgba8();//.resize_exact(2000, 1000, FilterType::Lanczos3).into_rgba8();
                            let dim = buffer_image.dimensions();
                            println!("{:?}",dim);
                            let rgb_img: ImageBuffer<image::Rgb<u8>, Vec<u8>> = convert_rgba_to_rgb(&buffer_image, dim.0, dim.1);

                            
                            state.set_frame(ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_fn(dim.0, dim.1, |x, y| {
                                let pixel = rgb_img.get_pixel(x, y);
                                image::Rgba([pixel[0], pixel[1], pixel[2], 255])
                            }));
                            
                            if screenshot_frame.width> 10 {
                            let (width, height, mut encoded_frames, _encode_duration) = encode(&rgb_img);
                            let ip = state.get_ip_receiver();
                            let _ = state.send_to_clients(encoded_frames,width, height,State::Receiving);
                        
                            }
                        },
                        
                        StreamingState::PAUSE =>{
                            // let buffer_image= paused_img.clone();
                            // let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 1000, |x, y| {
                            //     let pixel = buffer_image.get_pixel(x, y);
                            //     image::Rgb([pixel[0], pixel[1], pixel[2]])
                            // });
                        
                            
                            // let (_width, _height, mut encoded_frames, _encode_duration) = encode(&rgb_img);
                            // let ip = state.get_ip_receiver();
                            // let _ = send_screenshot(&mut encoded_frames,ip);
                            state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START && *s!=StreamingState::BLANK);
            
                        },
                        StreamingState::BLANK => {

                            let buffer_image= blanked_screen(2000, 1000);
                            let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 1000, |x, y| {
                                let pixel = buffer_image.get_pixel(x, y);
                                image::Rgb([pixel[0], pixel[1], pixel[2]])
                            });
                        
                            
                            let (width, height, mut encoded_frames, _encode_duration) = encode(&rgb_img);
                            
                            for _ in 0..7{
                                let ip = state.get_ip_receiver();
                                let _ = state.send_to_clients(encoded_frames.clone(),width, height,State::Blank);
                            }
                            state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START && *s!=StreamingState::PAUSE);
                        },
                        StreamingState::STOP => {

                            let buffer_image= stop_screen(2000, 1000);
                            let rgb_img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 1000, |x, y| {
                                let pixel = buffer_image.get_pixel(x, y);
                                image::Rgb([pixel[0], pixel[1], pixel[2]])
                            });
                            
                            let (width, height, mut encoded_frames, _encode_duration) = encode(&rgb_img);
                            for _ in 0..20{
                                let ip = state.get_ip_receiver();
                                let _ = state.send_to_clients(encoded_frames.clone(),width, height,State::Stop);
                            }
                            state.set_server(None);

                            break;
                        }
                    };
                }
                });
                loopRecorder(recorder,screenshot_frames_clone, state_clone);  

                a.join().unwrap();  
            
            }
            "receiver" => {
                println!("Receiver mode");

                let screenshot = Arc::new(Mutex::new(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(WIDTH as u32, HEIGHT as u32)));
                let to_redraw = Arc::new(Mutex::new(false));
                let screenshot_clone = screenshot.clone();                
                let screenshot_clone1 = screenshot.clone(); 
                let to_redraw_clone = Arc::clone(&to_redraw);

                // Spawn a thread to receive screenshots
                let state_clone= state.clone();
                let a = spawn_screenshot_thread(screenshot_clone, to_redraw_clone, state_clone);
                
                loop{
                    if state.get_sc_state()==StreamingState::STOP{
                        break;
                    }
                    let mut to_redraw = to_redraw.lock().unwrap();
                    if *to_redraw {
                        
                        let frame = get_frame(screenshot_clone1.clone());
                        
                        state.set_frame(frame);

                        *to_redraw = false;
                        
                    }
                    
                }
                println!("exit rec");
                a.join().unwrap();
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
    println!("Exit");
    Ok(())
}

fn get_frame(screenshot: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let screenshot = screenshot.lock().unwrap();
    
    let new_frame: ImageBuffer<Rgba<u8>, Vec<u8>> = screenshot.clone();
    
    return new_frame;


}

fn convert_rgba_to_rgb(
    sub_image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
    height: u32,
) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut rgb_img = ImageBuffer::new(width, height);
    for (x, y, pixel) in sub_image.enumerate_pixels() {
        rgb_img.put_pixel(x, y, image::Rgb([pixel[0], pixel[1], pixel[2]]));
    }
    rgb_img
}

fn spawn_screenshot_thread(screenshot_clone: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>, to_redraw_clone: Arc<Mutex<bool>>,state:Arc<screen_state>)->JoinHandle<()> {
    thread::spawn(move || {
        let now: DateTime<Local> = Local::now();

        // Format the time as a human-readable string
        let formatted_time = now.format("video_%Y_%m_%d__%H_%M_%S.h264").to_string();
        let mut video_writer = VideoWriter::new(100, formatted_time);
        //TODO this recording flag arrive from state and is setted by the user
        loop { 
            match state.get_sc_state(){
                StreamingState::STOP => {
                    if let Some(recording) = state.get_rec() {
                        if recording{ video_writer.write_to_file();}
                    }
                    state.set_frame(blanked_screen(2000, 1000));
                    break;
                },
                StreamingState::START =>{
                    let ip = state.get_ip_sender();
                    let new_screenshot = state.receive_from_server().unwrap();
                    println!("{:?}",new_screenshot.state);
                    if let Some(recording) = state.get_rec() {

                        if recording {
                            video_writer.add_frame(new_screenshot.data.clone());
                        }
                        else{
                            video_writer.write_to_file();
                            state.set_rec(None);
                        }
                    }
                    
                    let (_decode_duration, out_img) =decode(new_screenshot.data, new_screenshot.width, new_screenshot.height);
                    
                    if new_screenshot.state==State::Stop{
                        state.set_screen_state(StreamingState::STOP);
                        state.set_frame(blanked_screen(2000, 1000));
                        break;
                    }
                    let mut screenshot = screenshot_clone.lock().unwrap();
                    *screenshot = out_img;
                    let mut to_redraw = to_redraw_clone.lock().unwrap();
                    *to_redraw = true;
            
                      
                    
                },

                _ => {}
            };
                    
        }
    })
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