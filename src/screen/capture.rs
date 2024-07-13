pub mod capture{
    use scap::{
        capturer::{Area, Capturer, Options, Point, Size},
        frame::{BGRAFrame, Frame, FrameType},
    };
    use std::sync::{Arc, Mutex};

    use crate::{enums::StreamingState, screen::screen::screen_state};
    
    pub fn setRecorder() -> Capturer{

        let targets = scap::get_targets();
        println!("🎯 Targets: {:?}", targets);
    
        // #4 Create Options
        let options = Options {
            fps: 10,
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
    
    pub fn loopRecorder( mut recorder : Capturer, screenshot_clone: Arc<Mutex<BGRAFrame>>, state: Arc<screen_state>){
    
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
    
                        match state.get_sc_state(){
                            StreamingState::START => {
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
                            },
                            StreamingState::PAUSE => {
                                state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START);
                            },
                            StreamingState::BLANK => {
                                state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START);
                            },
                            StreamingState::STOP => {}
                        }
                        
                    }
                }
            }
            if state.get_sc_state()==StreamingState::STOP{
                recorder.stop_capture();
                break;
            }
        }
    }
}