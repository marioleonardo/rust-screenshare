#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod screen;
mod enums;
use std::default;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{env, thread};
use screen::screen::loop_logic;
use screen::screen::screen_state;
use winapi::shared::winerror::SEC_E_ONLY_HTTPS_ALLOWED;
use std::mem::needs_drop;
use local_ip_address::local_ip;
use image::{ImageBuffer, Rgba};
use eframe::egui::{self, Button, Color32, ColorImage, Key, KeyboardShortcut, ModifierNames, Modifiers, PointerButton, Pos2, TextBuffer};
//use screen::screen;
use crate::enums::StreamingState;
use screen::net::net::*;

#[derive(PartialEq, Debug, Default)]
enum CastRecEnum { 
    #[default]
    Caster, 
    Receiver 
}


#[derive(Default)]
enum Pages{
    #[default]
    HOME,
    CASTER,
    SETTING,
    RECEIVER,

}

#[derive(Default,PartialEq)]
enum Drawing{
    #[default]
    NONE,
    LINE,
    CIRCLE,
    TEXT,
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    let mut style: egui::Style = (*ctx.style()).clone();
    style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 15.0; // Cambia la dimensione del font a 24
    
    ctx.set_style(style);

    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../Hack-Regular.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
     
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());
    
    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

fn handle_mouse_input(ui: &egui::Ui, annotations: &mut Vec<(egui::Pos2, egui::Pos2)>, is_drawing:&mut bool) {

    ui.input(|i|{
        for event in &i.raw.events{
            if let egui::Event::PointerButton { pos, button, pressed, modifiers: _ }= event{
                if *pressed && *button==PointerButton::Primary{

                    
                    *is_drawing=true;
                    annotations.push((*pos,*pos)); 
                    
                }
                else if !*pressed {
                    
                    *is_drawing=false;
                }

            };
            if let egui::Event::PointerMoved(pos) = event{
                if *is_drawing==true{
                    
                    if let Some(last_ann) = annotations.last_mut() {
                    
                        last_ann.1 = *pos;
                    }
                    
                    
                }
                
            }

        }   
    }); 
}

fn handle_text_input(ui: &egui::Ui, annotations: &mut Vec<(egui::Pos2, String)>, is_drawing:&mut bool) {

    ui.input(|i|{
        for event in &i.raw.events{
            if let egui::Event::PointerButton { pos, button, pressed, modifiers: _ }= event{
                
                if *pressed && *button==PointerButton::Primary{
                    annotations.push((*pos,String::new()));
                    
                }

            };
            if let egui::Event::Text(text)= event{
                if let Some(last_ann) = annotations.last_mut() {
                    
                    last_ann.1.push_str(&text);
                }
            }
        }   
    }); 
}


fn main() -> eframe::Result<()> {
    
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
    renderer: eframe::Renderer::Wgpu,
    ..Default::default()
    };

    eframe::run_native(
        "Streaming Application",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    ) 
    
}

#[derive(Default)]
struct MyApp {
    current_page: Pages,
    texture: Option<egui::TextureHandle>,
    screenshot: Option<ColorImage>,
    temp_shortcut: Option<KeyboardShortcut>,
    start_shortcut: Option<KeyboardShortcut>,
    pause_shortcut:Option<KeyboardShortcut>,
    blank_shortcut:Option<KeyboardShortcut>,
    stop_shortcut:Option<KeyboardShortcut>,
    insert_shortcut_start:bool,
    insert_shortcut_pause:bool,
    insert_shortcut_blank:bool,
    insert_shortcut_stop:bool,
    my_enum: CastRecEnum,
    server_address: String,
    state: Arc<screen_state>,
    flag_thread: bool,
    x: String,
    y: String,
    f: String,
    line_annotations: Vec<(egui::Pos2, egui::Pos2)>,
    circle_annotations: Vec<(egui::Pos2, egui::Pos2)>,
    text_annotation: Vec<(egui::Pos2,String)>,
    is_drawing :bool,
    drawings: Drawing,
}

impl MyApp{

    fn start_cast_function(&mut self){

        if !self.flag_thread{
            let my_local_ip = local_ip().unwrap();
            self.state.set_ip_rec(my_local_ip.to_string()+":7878");

            let server = Server::new(my_local_ip.to_string()+":7878");
            let _ = server.bind_to_ip();

            self.state.set_server(Some(server));

            self.state.set_screen_state(StreamingState::START);
            self.flag_thread=true;

            let state_clone = self.state.clone();
            std::thread::spawn(move || {
            
            let _ = loop_logic("caster".to_string(), state_clone);

            }); 
        }
        else{
            self.state.set_screen_state(StreamingState::START);
            self.state.cv.notify_all();
        }
    }

    fn start_rec_function(&mut self){
        let client = Client::new(self.server_address.clone(), self.server_address.clone());
        if let Ok(stream) = client.connect_to_ip(){
            self.state.set_client(Some((stream, client)));
            self.state.set_ip_send(self.server_address.clone());
            let state_clone = self.state.clone();
        
            if !self.flag_thread{
            self.state.set_screen_state(StreamingState::START);
            self.flag_thread=true;

            std::thread::spawn(move || {
            println!("loop logic");
            let _ = loop_logic("receiver".to_string(), state_clone);

            });
            }
            else{
            self.state.set_screen_state(StreamingState::START);
            self.state.cv.notify_all();  
            }
        }
        else{
            self.current_page = Pages::HOME;
        }

        
    }
    
    fn screenshot(&mut self)->ColorImage{
        let mut st = self.state.clone();
        
        let img = st.get_frame();

        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels)
    }

    fn take_setting_icon(&self,path: &str)->ColorImage{
        let img = image::open(path).expect("Image does not exist");

        let img_buf = img.into_rgba8();
        let (height,width) = img_buf.dimensions();
        let pixels = img_buf.into_raw();

        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels)
    }

    fn get_mod_simbol(&self,modifier:Modifiers)->String{
        let modnames = ModifierNames::NAMES;
        let modnamesformat = modnames.format(&modifier,false);
        modnamesformat
    }

}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        setup_custom_fonts(&ctx);

        egui::CentralPanel::default().show(ctx, |ui| {

            let mut shapes:Vec<egui::Shape> = Vec::new();
    
            let button_width = ui.available_width()/5.0;
            let button_height = ui.available_height()/8.0;
            if let Some(screenshot) = &self.screenshot{
            
            self.texture = Some(ui.ctx().load_texture(
                "screenshot",
                screenshot.clone(),
                Default::default(),
            ));
            }
            else {
                self.texture=None;
            }
            let setting_img = Some(ui.ctx().load_texture(
                "settings",
                self.take_setting_icon("settings.png").clone(),
                Default::default(),
            )).unwrap();

            let back_img = Some(ui.ctx().load_texture(
                "back",
                self.take_setting_icon("back.png").clone(),
                Default::default(),
            )).unwrap();

            match self.current_page {

                Pages::HOME =>{

                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center),|ui| {
                        ui.heading("ScreenCast Application");

                        ui.add_space(30.0);

                        ui.label("Seleziona Modalità Operativa:");

                        ui.add_space(30.0);

                    });
                    
                    ui.horizontal(|ui|{
                        let cast_button = egui::Button::new("Caster").min_size(egui::vec2(ui.available_width()/2.0, button_height/2.0));
                        if ui.add(cast_button).clicked(){
                            self.my_enum=CastRecEnum::Caster;
                        };

                        let rec_button = egui::Button::new("Receiver").min_size(egui::vec2(ui.available_width(), button_height/2.0));
                        if ui.add(rec_button).clicked(){
                            self.my_enum=CastRecEnum::Receiver;
                        };
                    });

                    ui.add_space(30.0);

                    ui.horizontal(|ui|{
                        match self.my_enum{
                            CastRecEnum::Caster => {
                                let main_button = egui::Button::new("Condividi schermo").min_size(egui::vec2(ui.available_width(), button_height/2.0));
                                if ui.add(main_button).clicked(){
                                    self.current_page = Pages::CASTER; 
                                };
                            },
                            CastRecEnum::Receiver => {
                                ui.horizontal(|ui| {
    
                                    ui.label("IP Server:");
                                    //if self.server_address==""{self.server_address="192.168.88.107:7878".to_string()};
                                    
                                    ui.text_edit_singleline(&mut self.server_address);

                                    
                                });
                                let main_button = egui::Button::new("Visualizza straming").min_size(egui::vec2(ui.available_width(), button_height/2.0));
                                if ui.add(main_button).clicked(){
                                    self.current_page = Pages::RECEIVER;

                                    self.start_rec_function();
                                };
                            },
                        }
    
                    });

                    let color = match self.my_enum {
                        CastRecEnum::Caster => egui::Color32::RED,
                        CastRecEnum::Receiver => egui::Color32::GREEN,
                    };
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new(format!("{:?} è selezionato", self.my_enum)).color(color));
            
                },
                Pages::RECEIVER=>{

                    ui.horizontal(|ui|{
                        
                        let stop_button = egui::Button::new("Stop Streaming").min_size(egui::vec2(button_width,button_height));
                        if ui.add(stop_button).clicked() {
                            self.state.set_screen_state(StreamingState::STOP);
                            self.screenshot=None;
                            self.flag_thread=false;
                            self.current_page= Pages::HOME;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui|{
                            let setting_button =egui::ImageButton::new((setting_img.id(),egui::vec2(button_height/1.7,button_height/1.7))).rounding(30.0);
                        if ui.add(setting_button).clicked() {
                            self.current_page= Pages::SETTING;
                        }
                        let back_button =egui::ImageButton::new((back_img.id(),egui::vec2(button_height/1.7,button_height/1.7))).rounding(30.0);
                        if ui.add(back_button).clicked() {
                            self.state.set_screen_state(StreamingState::STOP);
                            self.screenshot=None;
                            self.flag_thread=false;
                            self.current_page= Pages::HOME;
                        }
                        });
                    });

                    if let Some(texture) = self.texture.as_ref() {
                        ui.image((texture.id(), ui.available_size()));
                    }  else {
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::BottomUp), |ui|{
                            ui.spinner();
                        });
                    }

                    if self.state.get_sc_state() == StreamingState::STOP{
                        self.screenshot=None;
                        self.flag_thread=false;
                        self.current_page=Pages::HOME;

                    }
                    else{
                        self.screenshot= Some(self.screenshot());
                    }

                    //HANDLE SHORTCUTS
                    ui.input(|i|{
                        for event in &i.raw.events {
                            if let egui::Event::Key { key, pressed,modifiers, .. } = event{
                                if *pressed{   
                                    self.temp_shortcut=Some(egui::KeyboardShortcut::new(modifiers.clone(), key.clone()));
                                }
                            
                            }  
                        }
                        //check inserted shorcut
                        if let Some(sct) = self.temp_shortcut{
                            if let Some(sc) = self.stop_shortcut{
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::STOP);
                                    self.current_page= Pages::HOME;
                                    self.temp_shortcut=None;
                                }
                            }
                        }
                        
                        self.temp_shortcut=None;
                             
                    });

                    ctx.request_repaint();
                },
                Pages::CASTER => {

                    //HANDLE BUTTONS
                    ui.horizontal(|ui| {
                        
                        let start_button= egui::Button::new(match self.state.get_sc_state(){
                            StreamingState::START => "Start Streaming",
                            StreamingState::PAUSE => "Resume Streming",
                            StreamingState::BLANK => "Resume Streaming",
                            StreamingState::STOP => "Start Streaming",
                        }).min_size(egui::vec2(button_width, button_height));
                        if ui.add(start_button).clicked() {
                            self.start_cast_function();
                        }
                        
                        let pause_button= egui::Button::new("Pause Streaming").min_size(egui::vec2(button_width, button_height));
                        if ui.add(pause_button).clicked() {
                            self.state.set_screen_state(StreamingState::PAUSE);
                            self.state.cv.notify_all();
                        }
        
                        let blank_button = egui::Button::new("Blank Streaming").min_size(egui::vec2(button_width,button_height));
                        if ui.add(blank_button).clicked() {
                            self.state.set_screen_state(StreamingState::BLANK);
                            self.state.cv.notify_all();
                        }
        
                        let stop_button = egui::Button::new("Stop Streaming").min_size(egui::vec2(button_width,button_height));
                        if ui.add(stop_button).clicked() {
                            self.state.set_screen_state(StreamingState::STOP);
                            self.state.set_server(None);
                            self.screenshot=None;
                            self.flag_thread=false;
                            self.current_page= Pages::HOME;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui|{
                            let setting_button =egui::ImageButton::new((setting_img.id(),egui::vec2(button_height/1.7,button_height/1.7))).rounding(30.0);
                        if ui.add(setting_button).clicked() {
                            self.current_page= Pages::SETTING;
                        }
                        let back_button =egui::ImageButton::new((back_img.id(),egui::vec2(button_height/1.7,button_height/1.7))).rounding(30.0);
                        if ui.add(back_button).clicked() {
                            self.state.set_screen_state(StreamingState::STOP);
                            self.screenshot=None;
                            self.state.set_server(None);
                            self.flag_thread=false;
                            self.current_page= Pages::HOME;
                        }
                        });
                        
        
                    });

                    //Stream Customization
                    ui.add_space(20.0);
                    ui.horizontal(|ui|{
                        ui.label("Customize screen trasmetted");
                    });
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min),|ui|{
                        ui.horizontal(|ui|{
                            ui.label("X coordinate");
                            ui.add_space(90.0);
                            self.x = self.state.get_x().to_string();
                            if ui.text_edit_singleline(&mut self.x).changed(){
                                if let Ok(n) = self.x.parse::<u32>(){
                                    if n<2000{
                                        self.state.set_x(n);
                                    }
                                    else{
                                        self.state.set_x(1999);
                                    }
                                }
                                else if self.x=="".to_string(){
                                    self.x=0.to_string();
                                    self.state.set_x(0);
                                }
                                else{
                                    ui.label("Invalid value");
                                }
                            };
                        });
                        
                        ui.horizontal(|ui|{
                            ui.label("Y coordinate");
                            ui.add_space(90.0);
                            self.y = self.state.get_y().to_string();
                            if ui.text_edit_singleline(&mut self.y).changed(){
                                if let Ok(n) = self.y.parse::<u32>(){
                                    if n<100{
                                        self.state.set_y(n);
                                    }
                                    else{
                                        self.state.set_y(999); 
                                    }
                                }
                                else if self.y=="".to_string(){
                                    self.y=0.to_string();
                                    self.state.set_y(0);
                                }
                                else{
                                    ui.label("Invalid value");
                                }
                            };
                        });

                        ui.horizontal(|ui|{
                            ui.label("Screen reduction (%)");
                            ui.add_space(20.0);
                            self.f = self.state.get_f().to_string();
                            if ui.text_edit_singleline(&mut self.f).changed(){
                                if let Ok(n) = self.f.parse::<u32>(){
                                    if n<=100 && n>0 {self.state.set_f(n);}
                                }
                                else if self.f=="".to_string(){
                                    self.f=0.to_string();
                                    self.state.set_f(100);
                                }
                                else{
                                    ui.label("Invalid value");
                                }
                            };
                        });
                    });
                    ui.add_space(10.0);
                    let my_local_ip = local_ip().unwrap();
                    ui.horizontal(|ui|{
                        ui.label("Your Server IP: ");
                        ui.label(my_local_ip.to_string()+":7878");
                    });

                    //visualize screen state
                    /* 
                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui|{
                        match self.state.get_sc_state(){
                            StreamingState::START => {
                                ui.label("Trasmitting...");
                            },
                            StreamingState::PAUSE => {
                                ui.label("Streaming Pause...");
                            },
                            StreamingState::BLANK => {
                                ui.label("Streaming Blank...");
                            },
                            StreamingState::STOP => {},
                        };
                    });
                    */
                    if self.state.get_sc_state()==StreamingState::START{
                        self.screenshot= Some(self.screenshot());
                    }

                    if let Some(texture) = self.texture.as_ref() {

                        match self.drawings{
                            Drawing::NONE => {},
                            Drawing::LINE => {

                                handle_mouse_input(ui, &mut self.line_annotations, &mut self.is_drawing);
 
                            },
                            Drawing::CIRCLE => {

                                handle_mouse_input(ui, &mut self.circle_annotations, &mut self.is_drawing);

                            },
                            Drawing::TEXT => {

                                handle_text_input(ui, &mut self.text_annotation, &mut self.is_drawing);

                            },
                        }

                        for &(start, end) in &self.line_annotations {
                            println!("start: {:?}, end: {:?}",start,end);
                            shapes.push(egui::Shape::line_segment(
                            [start,end],
                            egui::Stroke::new(2.0, Color32::RED),
                        ));
                        }

                        for &(start, end) in &self.circle_annotations {
                                
                            shapes.push(egui::Shape::circle_stroke(
                                start, 
                                ((start.x-end.x).powi(2)+(start.y-end.y).powi(2)).sqrt(),
                                egui::Stroke::new(2.0, Color32::RED)));
                            }

                        for (start, text) in &mut self.text_annotation {
                            
                            ui.fonts(|f|{
                                let t = egui::Shape::text(f, *start, egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(15.0), Color32::RED);
                                shapes.push(t);
                            });
                            
                            
                            }
                        
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center),|ui|{
                            ui.image((texture.id(), egui::vec2(ui.available_width()*4.0/5.0,ui.available_height())));

                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui|{

                                ui.horizontal(|ui|{
                                    ui.label("Annotations");
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui|{
                                    let draw_line_button = egui::Button::new("Line").min_size(ui.available_size()).fill(
                                        if self.drawings==Drawing::LINE{
                                            Color32::GREEN
                                        }
                                        else{
                                            Color32::GRAY
                                        }
                                    );
                                    if ui.add(draw_line_button).clicked(){
                                    if self.drawings==Drawing::LINE{
                                        self.drawings=Drawing::NONE;
                                    }
                                    else{
                                        self.drawings=Drawing::LINE;
                                    } 
                                    }
                            
                                });

                                ui.horizontal(|ui|{
                                    let draw_circle_button = egui::Button::new("Circle").min_size(ui.available_size()).fill(
                                        if self.drawings==Drawing::CIRCLE{
                                            Color32::GREEN
                                        }
                                        else{
                                            Color32::GRAY
                                        }
                                    );
                                    if ui.add(draw_circle_button).clicked(){
                                    
                                    if self.drawings==Drawing::CIRCLE{
                                        self.drawings=Drawing::NONE;
                                    }
                                    else{
                                        self.drawings=Drawing::CIRCLE;
                                    }
                                    
                                    }
                                });

                                ui.horizontal(|ui|{
                                    let draw_text_button = egui::Button::new("Text").min_size(ui.available_size()).fill(
                                        if self.drawings==Drawing::TEXT{
                                            Color32::GREEN
                                        }
                                        else{
                                            Color32::GRAY
                                        }
                                    );
                                    if ui.add(draw_text_button).clicked(){
                                    
                                    if self.drawings==Drawing::TEXT{
                                        self.drawings=Drawing::NONE;
                                    }
                                    else{
                                        self.drawings=Drawing::TEXT;
                                    }
                                    
                                    }
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui|{
                                    let clear_button = egui::Button::new("Clear All").min_size(ui.available_size()).fill(Color32::GRAY);
                                    if ui.add(clear_button).clicked(){
                                        self.line_annotations.clear();
                                        self.circle_annotations.clear();
                                        self.text_annotation.clear();
                                        shapes.clear();
                                    }
                                });

                            });
                            
                            let painter = ui.painter();
                            if self.drawings!=Drawing::NONE{
                                painter.extend(shapes);

                            }
                            
                        });

                    }  else {
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::BottomUp), |ui|{
                            ui.spinner();
                        });
                    }
                     
                    //HANDLE SHORTCUTS
                    ui.input(|i|{
                        for event in &i.raw.events {
                            if let egui::Event::Key { key, pressed,modifiers, .. } = event{
                                if *pressed{   
                                    self.temp_shortcut=Some(egui::KeyboardShortcut::new(modifiers.clone(), key.clone()));
                                }
                            
                            }
                            
                        }
                        //check inserted shorcut
                        if let Some(sct) = self.temp_shortcut{
                            if let Some(sc) = self.start_shortcut{
                                if sct == sc {
                                    self.start_cast_function();
                                    self.temp_shortcut=None;
                                }
                            }
                            if let Some(sc) = self.pause_shortcut{
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::PAUSE);
                                    self.temp_shortcut=None;
                                }
                            }
                            if let Some(sc) = self.blank_shortcut{
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::BLANK);
                                    self.temp_shortcut=None;
                                }
                            }
                            if let Some(sc) = self.stop_shortcut{
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::STOP);
                                    self.current_page= Pages::HOME;
                                    self.temp_shortcut=None;
                                }
                            }
                        }
                        
                        self.temp_shortcut=None;
                             
                    });
                    ctx.request_repaint();
        
                },
                Pages::SETTING => {
                    
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui|{
                        
                        ui.spacing_mut().item_spacing.y = button_width/4.0;
                        ui.heading("Shotcut Settings");
                        
                        
                        ui.horizontal(|ui|{
                            //Visualize shortcut buttons
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_start{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            ui.add_space(30.0);
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_blank && !self.insert_shortcut_pause && !self.insert_shortcut_stop{
                                self.insert_shortcut_start=!self.insert_shortcut_start;
                                if !self.insert_shortcut_start {

                                    self.start_shortcut=self.temp_shortcut.clone();
                                    self.temp_shortcut=None;
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.start_shortcut=None;
                            }
                            
                            //Visualize Shotcut String
                            ui.add_space(30.0);
                            ui.label(format!("Start Streaming Shortcut : "));
                            if self.insert_shortcut_start {
                                if let Some(sc) = self.temp_shortcut{ 
                                    ui.label(format!("{:}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }else{
                                if let Some(sc) = self.start_shortcut{                                    
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }
                        });
                        
                        ui.spacing_mut().item_spacing.y = button_width/4.0;

                        ui.horizontal(|ui|{
                            //Visualize shortcut buttons
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_pause{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            ui.add_space(30.0);
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_blank && !self.insert_shortcut_start && !self.insert_shortcut_stop{
                                self.insert_shortcut_pause=!self.insert_shortcut_pause;
                                if !self.insert_shortcut_pause {

                                    self.pause_shortcut=self.temp_shortcut.clone();
                                    self.temp_shortcut=None;
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.pause_shortcut=None;
                            }
                            
                            //Visualize Shotcut String
                            ui.add_space(30.0);
                            ui.label(format!("Pause Streaming Shortcut : "));
                            if self.insert_shortcut_pause {
                                if let Some(sc) = self.temp_shortcut{ 
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }else{
                                if let Some(sc) = self.pause_shortcut{ 
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }
                        });

                        ui.spacing_mut().item_spacing.y = button_width/4.0;
                        
                        ui.horizontal(|ui|{
                            //Visualize shortcut buttons
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_blank{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            ui.add_space(30.0);
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_start && !self.insert_shortcut_pause && !self.insert_shortcut_stop{
                                self.insert_shortcut_blank=!self.insert_shortcut_blank;
                                if !self.insert_shortcut_blank {

                                    self.blank_shortcut=self.temp_shortcut.clone();
                                    self.temp_shortcut=None;
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.blank_shortcut=None;
                            }
                            
                            //Visualize Shotcut String
                            ui.add_space(30.0);
                            ui.label(format!("Blank Streaming Shortcut : "));
                            if self.insert_shortcut_blank {
                                if let Some(sc) = self.temp_shortcut{
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }else{
                                if let Some(sc) = self.blank_shortcut{ 
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }
                        });

                        ui.spacing_mut().item_spacing.y = button_width/4.0;
                        
                        ui.horizontal(|ui|{
                            //Visualize shortcut buttons
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_stop{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            ui.add_space(30.0);
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_blank && !self.insert_shortcut_pause && !self.insert_shortcut_start{
                                self.insert_shortcut_stop=!self.insert_shortcut_stop;
                                if !self.insert_shortcut_stop {

                                    self.stop_shortcut=self.temp_shortcut.clone();
                                    self.temp_shortcut=None;
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.stop_shortcut=None;
                            }
                            
                            //Visualize Shotcut String
                            ui.add_space(30.0);
                            ui.label(format!("Stop Streaming Shortcut : "));
                            if self.insert_shortcut_stop {
                                if let Some(sc) = self.temp_shortcut{ 
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }else{
                                if let Some(sc) = self.stop_shortcut{ 
                                    ui.label(format!("{}+{:?}",self.get_mod_simbol(sc.modifiers),sc.logical_key));
                                }
                            }
                        });
                        ui.spacing_mut().item_spacing.y = button_width/8.0;
                        
                    });        
                    

                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::BottomUp), |ui|{
                        if self.insert_shortcut_start || self.insert_shortcut_pause || self.insert_shortcut_blank || self.insert_shortcut_stop{
                            ui.label("Press the shortcut");
                        }
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui|{
                        let back_button =egui::ImageButton::new((back_img.id(),egui::vec2(35.0,35.0))).rounding(30.0);
                        if ui.add(back_button).clicked() {
                            match self.my_enum{
                                CastRecEnum::Caster => {
                                    self.current_page= Pages::CASTER;
                                },
                                CastRecEnum::Receiver => {
                                    self.current_page= Pages::RECEIVER;
                                },
                            }
                            self.state.set_screen_state(StreamingState::STOP);
                        }
                    });
                    
                    ui.input(|i|{
                        for event in &i.raw.events {
                            if let egui::Event::Key { key, pressed, modifiers, .. } = event{
                                if self.insert_shortcut_start || self.insert_shortcut_pause || self.insert_shortcut_blank || self.insert_shortcut_stop{
                                    if *pressed{
                                        
                                        self.temp_shortcut=Some(egui::KeyboardShortcut::new(modifiers.clone(), key.clone()));
                                 
                                    }
                            }
                            }
                            
                        }
                        
                    });    
                                    
                    //ctx.request_repaint();
                },
                
            }
        });

    }
}
