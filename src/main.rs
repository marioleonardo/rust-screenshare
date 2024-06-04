#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod screen;
use std::default;
use std::sync::Arc;
use std::{env, os::windows::thread};

use eframe::egui::mutex::Mutex;
use screen::screen::loop_logic;
use screen::screen::screen_state;
use std::mem::needs_drop;

use image::{ImageBuffer, Rgba};
use eframe::egui::{self, ColorImage, Key, KeyboardShortcut, ModifierNames, Modifiers, TextBuffer};
//use screen::screen;


#[derive(PartialEq, Debug, Default)]
enum CastRecEnum { 
    #[default]
    Caster, 
    Receiver 
}

#[derive(Default)]
enum StreamingState{
    START,
    PAUSE,
    BLANK,
    #[default]
    STOP,
}
#[derive(Default)]
enum Pages{
    #[default]
    HOME,
    CASTER,
    SETTING,
    RECEIVER,

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


fn main() -> eframe::Result<()> {

    let args: Vec<String> = env::args().collect();
    let state=(screen_state::default());
    std::thread::spawn(move ||{
        
        loop_logic(args,state);

    });

    
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
        let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Streaming Application",
        options,
        Box::new(|_cc| Box::new(MyApp::new(state))),
    )

    
    
}

#[derive(Default)]
struct MyApp {
    current_page: Pages,
    stream_screenshots: StreamingState,
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
    state: screen_state,
}

impl MyApp{

    fn new(state:screen_state)->Self{
        let mut a = Self::default();
        a.state=state;
        a
    }
    
    fn screenshot(&mut self)->ColorImage{
        let img = self.state.get_frame();
        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels)
    }

    fn blanked_screen(&self)->ColorImage{
        let pixel = Rgba([255u8,255u8,255u8,255u8]);
        let img = ImageBuffer::from_pixel(1920, 1080, pixel);

        let (height,width) = img.dimensions();
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
                    ui.heading("ScreenCast Application");

            ui.horizontal(|ui| {
                ui.label("Seleziona Modalità Operativa:");
                ui.selectable_value(&mut self.my_enum, CastRecEnum::Caster, "Caster");
                ui.selectable_value(&mut self.my_enum, CastRecEnum::Receiver, "Receiver");
            });
            if self.my_enum == CastRecEnum::Receiver {
                ui.horizontal(|ui| {
                    ui.label("Indirizzo del Server:");
                    self.server_address="0.0.0.0".to_string();
                    ui.text_edit_singleline(&mut self.server_address);
                    
                });
                if ui.button("Visualizza trasmissione").clicked(){
                    self.current_page = Pages::RECEIVER;
                };
            }
            if self.my_enum == CastRecEnum::Caster {
                if ui.button("Condividi schermo").clicked(){
                    self.current_page = Pages::CASTER;
                };
            }

            // Change color based on selection
            let color = match self.my_enum {
                CastRecEnum::Caster => egui::Color32::RED,
                CastRecEnum::Receiver => egui::Color32::GREEN,
            };
            ui.label(egui::RichText::new(format!("{:?} è selezionato", self.my_enum)).color(color));

                },
                Pages::CASTER => {

                    ui.horizontal(|ui| {
                        
                        let start_button= egui::Button::new(match self.stream_screenshots{
                            StreamingState::START => "Start Streaming",
                            StreamingState::PAUSE => "Resume Streming",
                            StreamingState::BLANK => "Resume Streaming",
                            StreamingState::STOP => "Start Streaming",
                        }).min_size(egui::vec2(button_width, button_height));
                        if ui.add(start_button).clicked() {
                            self.stream_screenshots = StreamingState::START;
                        }
                        
                        let pause_button= egui::Button::new("Pause Streaming").min_size(egui::vec2(button_width, button_height));
                        if ui.add(pause_button).clicked() {
                            self.stream_screenshots = StreamingState::PAUSE;
                        }
        
                        let blank_button = egui::Button::new("Blank Streaming").min_size(egui::vec2(button_width,button_height));
                        if ui.add(blank_button).clicked() {
                            self.stream_screenshots = StreamingState::BLANK;
                        }
        
                        let stop_button = egui::Button::new("Stop Streaming").min_size(egui::vec2(button_width,button_height));
                        if ui.add(stop_button).clicked() {
                            self.stream_screenshots = StreamingState::STOP;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui|{
                            let setting_button =egui::ImageButton::new((setting_img.id(),egui::vec2(button_height/1.7,button_height/1.7))).rounding(30.0);
                        if ui.add(setting_button).clicked() {
                            self.current_page= Pages::SETTING;
                        }
                        let back_button =egui::ImageButton::new((back_img.id(),egui::vec2(button_height/1.7,button_height/1.7))).rounding(30.0);
                        if ui.add(back_button).clicked() {
                            self.current_page= Pages::HOME;
                        }
                        });
                        
        
                    });

                    //visualize screen
                    if let Some(texture) = self.texture.as_ref() {
                        ui.image((texture.id(), ui.available_size()));
                    }  else {
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::BottomUp), |ui|{
                            ui.spinner();
                        });
                    }
                    
                    //visualize options
                    match self.stream_screenshots{
                        //take new screenshot
                        StreamingState::START => self.screenshot= Some(self.screenshot()),
                        //keep same screenshot
                        StreamingState::PAUSE => self.screenshot= self.screenshot.clone(),
                        //take blanked screen
                        StreamingState::BLANK => self.screenshot=Some(self.blanked_screen()),
                        //remove screen
                        StreamingState::STOP => self.screenshot=None,
                    }

                    
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
                                    self.stream_screenshots = StreamingState::START;
                                    self.temp_shortcut=None;
                                }
                            }
                            if let Some(sc) = self.pause_shortcut{
                                if sct == sc {
                                    self.stream_screenshots = StreamingState::PAUSE;
                                    self.temp_shortcut=None;
                                }
                            }
                            if let Some(sc) = self.blank_shortcut{
                                if sct == sc {
                                    self.stream_screenshots = StreamingState::BLANK;
                                    self.temp_shortcut=None;
                                }
                            }
                            if let Some(sc) = self.stop_shortcut{
                                if sct == sc {
                                    self.stream_screenshots = StreamingState::STOP;
                                    self.temp_shortcut=None;
                                }
                            }
                        }
                        
                        self.temp_shortcut=None;
                             
                    });
                    //ctx.request_repaint();
        
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
                            self.current_page= Pages::CASTER;
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
                Pages::RECEIVER=>{

                }
            }
        });

    }
}
