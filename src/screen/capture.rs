// pub mod capture{
    
//     use scap::{
//         capturer::{Area, Capturer, Options, Point, Size},
//         frame::{BGRAFrame, Frame, FrameType}, Target,
//     };
//     use std::sync::{Arc, Mutex};

//     use crate::{enums::StreamingState, screen::screen::screen_state};
    
//     pub fn getMonitors()-> Vec<Target>{
//         let targets = scap::get_all_targets();


//         let targets = targets.iter().filter_map(|x| {
//             if let Target::Display(_) = x {
//                 Some(x.clone())
//             } else {
//                 None
//             }
//         }).collect::<Vec<Target>>();

//         return targets;
//     }
    
//     #[cfg(target_os = "linux")]
//     pub fn setRecorder(target:u8) -> Capturer{

//         let targets = scap::get_all_targets();
//         println!("🎯 Targets: {:?}", targets);
    
//         // #4 Create Options
//         let options = Options {
//             fps: 10,
//             //target: Some(target),
//             show_cursor: true,
//             show_highlight: true,
//             excluded_targets: None,
//             output_type: FrameType::BGRAFrame,
//             output_resolution: scap::capturer::Resolution::_720p,
            
//             ..Default::default()
//         };
    
//         // #5 Create Recorder
//         let mut recorder = Capturer::new(options);
    
//         // #6 Start Capture
//         recorder.start_capture();
    
//         recorder
//     }

//     #[cfg(target_os = "windows")]
//     pub fn setRecorder(target:Target) -> Capturer{

//         let targets = scap::get_all_targets();
//         println!("🎯 Targets: {:?}", targets);
    
//         // #4 Create Options
//         let options = Options {
//             fps: 10,
//             target: Some(target),
//             show_cursor: true,
//             show_highlight: true,
//             excluded_targets: None,
//             output_type: FrameType::BGRAFrame,
//             output_resolution: scap::capturer::Resolution::_720p,
            
//             ..Default::default()
//         };
    
//         // #5 Create Recorder
//         let mut recorder = Capturer::new(options);
    
//         // #6 Start Capture
//         recorder.start_capture();
    
//         recorder
//     }

//     #[cfg(target_os = "macos")]
//     pub fn setRecorder(target:u8) -> Capturer{

//         let targets = scap::get_all_targets();
//         println!("🎯 Targets: {:?}", targets);
    
//         // #4 Create Options
//         let options = Options {
//             fps: 10,
//             //target: Some(target),
//             show_cursor: true,
//             show_highlight: true,
//             excluded_targets: None,
//             output_type: FrameType::BGRAFrame,
//             output_resolution: scap::capturer::Resolution::_720p,
            
//             ..Default::default()
//         };
    
//         // #5 Create Recorder
//         let mut recorder = Capturer::new(options);
    
//         // #6 Start Capture
//         recorder.start_capture();
    
//         recorder
//     }
//     pub fn loopRecorder( mut recorder : Capturer, screenshot_clone: Arc<Mutex<BGRAFrame>>, state: Arc<screen_state>){
    
//         let mut fps_counter = 0;
//         let mut last_fps_time = std::time::Instant::now();
//         loop {
//             let mut frames:Vec<BGRAFrame> = Vec::new();
//             frames.clear();
            
//             for i in 0..20 {
//                 let frame = recorder.get_next_frame().expect("Error");
    
//                 match frame {
//                     Frame::YUVFrame(frame) => {
    
//                         println!(
//                             "Recieved YUV frame {} of width {} and height {} and pts {}",
//                             i, frame.width, frame.height, frame.display_time
//                         );
//                     }
//                     Frame::BGR0(frame) => {
//                         println!(
//                             "Received BGR0 frame of width {} and height {}",
//                             frame.width, frame.height
//                         );
//                     }
//                     Frame::RGB(frame) => {
    
//                         println!(
//                             "Recieved RGB frame of width {} and height {}",
//                             frame.width, frame.height
//                         );
//                     }
//                     Frame::RGBx(frame) => {
//                         println!(
//                             "Recieved RGBx frame of width {} and height {}",
//                             frame.width, frame.height
//                         );
//                     }
//                     Frame::XBGR(frame) => {
//                         println!(
//                             "Recieved xRGB frame of width {} and height {}",
//                             frame.width, frame.height
//                         );
//                     }
//                     Frame::BGRx(frame) => {
//                         println!(
//                             "Recieved BGRx frame of width {} and height {}",
//                             frame.width, frame.height
//                         );
//                         match state.get_sc_state(){
                            
//                             StreamingState::START => {
//                                 let mut bgra_data = Vec::with_capacity((frame.width * frame.height * 4) as usize);

//                                 // Iterate over the BGRx data in chunks of 4 (BGRx format)
//                                 for chunk in frame.data.chunks_exact(4) {
//                                     // Copy B, G, R channels and insert alpha (255)
//                                     bgra_data.push(chunk[0]); // Blue
//                                     bgra_data.push(chunk[1]); // Green
//                                     bgra_data.push(chunk[2]); // Red
//                                     bgra_data.push(255);      // Alpha (Opaque)
//                                 }

//                                 let mut bgra_frame = BGRAFrame {
//                                     display_time: frame.display_time,
//                                     width: frame.width,
//                                     height: frame.height,
//                                     data: bgra_data,
//                                 };

//                                 for chunk in bgra_frame.data.chunks_exact_mut(4) {
//                                     // Swap the first (blue) and third (red) elements
//                                     chunk.swap(0, 2);
//                                 }
//                                 let mut screenshot_clone=screenshot_clone.lock().unwrap();
//                                 *screenshot_clone = bgra_frame.clone();

//                                 fps_counter += 1;
//                                 let elapsed = last_fps_time.elapsed();
//                                 if elapsed >= std::time::Duration::from_secs(1) {
//                                 let fps = fps_counter as f64 / elapsed.as_secs_f64();
//                                 println!("FPS: {:.2}", fps);
//                                 fps_counter = 0;
//                                 last_fps_time = std::time::Instant::now();
//                                 }
                            
//                             },
//                             StreamingState::PAUSE => {
//                                 state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START);
//                             },
//                             StreamingState::BLANK => {
//                                 state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START);
//                             },
//                             StreamingState::STOP => {}
//                         }
//                     }
//                     Frame::BGRA(mut frame) => {
    
//                         match state.get_sc_state(){
//                             StreamingState::START => {
//                                 for chunk in frame.data.chunks_exact_mut(4) {
//                                     // Swap the first (blue) and third (red) elements
//                                     chunk.swap(0, 2);
//                                 }
//                                 let mut screenshot_clone=screenshot_clone.lock().unwrap();
//                                 *screenshot_clone = frame.clone();

//                                 fps_counter += 1;
//                                 let elapsed = last_fps_time.elapsed();
//                                 if elapsed >= std::time::Duration::from_secs(1) {
//                                 let fps = fps_counter as f64 / elapsed.as_secs_f64();
//                                 println!("FPS: {:.2}", fps);
//                                 fps_counter = 0;
//                                 last_fps_time = std::time::Instant::now();
//                                 }
                            
//                             },
//                             StreamingState::PAUSE => {
//                                 state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START);
//                             },
//                             StreamingState::BLANK => {
//                                 state.cv.wait_while(state.stream_state.lock().unwrap(), |s| *s!=StreamingState::START);
//                             },
//                             StreamingState::STOP => {}
//                         }
                        
//                     }
//                 }
//             }
//             if state.get_sc_state()==StreamingState::STOP{
//                 recorder.stop_capture();
//                 break;
//             }
//         }
//     }
// }