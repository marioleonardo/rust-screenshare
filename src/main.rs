#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod screen;

use image::{ImageBuffer, Rgba};
use eframe::{egui::{self, ColorImage, Key, Ui}, wgpu::hal::Label};
use screen::screen::capture_screenshot;


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
    CASTER,
    SETTING,
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
    stream_screenshots: StreamingState,
    texture: Option<egui::TextureHandle>,
    screenshot: Option<ColorImage>,
    start_shortcut:Vec<Key>,
    pause_shortcut:Vec<Key>,
    blank_shortcut:Vec<Key>,
    stop_shortcut:Vec<Key>,
    vec_keys:Vec<Key>,
    insert_shortcut_start:bool,
    insert_shortcut_pause:bool,
    insert_shortcut_blank:bool,
    insert_shortcut_stop:bool,
}

impl MyApp{
    
    fn screenshot(&self)->ColorImage{
        let img = capture_screenshot();
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


}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut style: egui::Style = (*ctx.style()).clone();
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 15.0; // Cambia la dimensione del font a 24
        ctx.set_style(style);

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
                            self.current_page= Pages::CASTER;
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
                    
        
                    ctx.request_repaint();
                },
                Pages::SETTING => {
                    
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui|{
                        
                        ui.spacing_mut().item_spacing.y = 40.0;
                        ui.heading("Shotcut Settings");
                        
                        ui.horizontal(|ui|{
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_start{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));

                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_blank && !self.insert_shortcut_pause && !self.insert_shortcut_stop{
                                self.insert_shortcut_start=!self.insert_shortcut_start;
                                if !self.insert_shortcut_start {
                                    self.start_shortcut=self.vec_keys.clone();
                                    self.vec_keys.clear();
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.start_shortcut.clear();
                            }
                            
                            let label:Vec<String>;
                            
                            if self.insert_shortcut_start{
                                label=self.vec_keys.iter().map(|k|{format!("{:?}",k)}).collect();
                            }else{
                                label=self.start_shortcut.iter().map(|k|{format!("{:?}",k)}).collect();
                            }
                            ui.add_space(30.0);
                            ui.label(format!("Start Streaming Shortcut : {:?}",label));
                        
                            
                        });
                        
                        ui.spacing_mut().item_spacing.y = 40.0;

                        ui.horizontal(|ui|{
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_pause{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_blank && !self.insert_shortcut_start && !self.insert_shortcut_stop {
                                self.insert_shortcut_pause=!self.insert_shortcut_pause;
                                if !self.insert_shortcut_pause {
                                    self.pause_shortcut=self.vec_keys.clone();
                                    self.vec_keys.clear();
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.pause_shortcut.clear();
                            }
                            
                            let label:Vec<String>;
                            
                            if self.insert_shortcut_pause{
                                label=self.vec_keys.iter().map(|k|{format!("{:?}",k)}).collect();
                            }else{
                                label=self.pause_shortcut.iter().map(|k|{format!("{:?}",k)}).collect();
                            }
                            ui.add_space(30.0);
                            ui.label(format!("Pause Streaming Shortcut : {:?}",label));
                       
                        });

                        ui.spacing_mut().item_spacing.y = 40.0;
                        
                        ui.horizontal(|ui|{
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_blank{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_pause && !self.insert_shortcut_start && !self.insert_shortcut_stop{
                                self.insert_shortcut_blank=!self.insert_shortcut_blank;
                                if !self.insert_shortcut_blank {
                                    self.blank_shortcut=self.vec_keys.clone();
                                    self.vec_keys.clear();
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.blank_shortcut.clear();
                            }
                            
                            let label:Vec<String>;
                            
                            if self.insert_shortcut_blank{
                                label=self.vec_keys.iter().map(|k|{format!("{:?}",k)}).collect();
                            }else{
                                label=self.blank_shortcut.iter().map(|k|{format!("{:?}",k)}).collect();
                            }
                            ui.add_space(30.0);
                            ui.label(format!("Blank Streaming Shortcut : {:?}",label));
                       
                        });

                        ui.spacing_mut().item_spacing.y = 40.0;
                        
                        ui.horizontal(|ui|{
                            let add_close_butt= egui::Button::new(if self.insert_shortcut_stop{"End"} else {"Add"}).min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            let clear_butt= egui::Button::new("Clear").min_size(egui::vec2(button_width/2.0,button_height/2.0));
                            
                            if ui.add(add_close_butt).clicked() && !self.insert_shortcut_blank && !self.insert_shortcut_pause && !self.insert_shortcut_start{
                                self.insert_shortcut_stop=!self.insert_shortcut_stop;
                                if !self.insert_shortcut_stop {
                                    self.stop_shortcut=self.vec_keys.clone();
                                    self.vec_keys.clear();
                                };
                            }
                            if ui.add(clear_butt).clicked(){
                                self.stop_shortcut.clear();
                            }
                            
                            let label:Vec<String>;
                            
                            if self.insert_shortcut_stop{
                                label=self.vec_keys.iter().map(|k|{format!("{:?}",k)}).collect();
                            }else{
                                label=self.stop_shortcut.iter().map(|k|{format!("{:?}",k)}).collect();
                            }
                            ui.add_space(30.0);
                            ui.label(format!("Blank Streaming Shortcut : {:?}",label));
                       
                        });
                        ui.spacing_mut().item_spacing.y = 20.0;
                        
                            
                    });

                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::BottomUp), |ui|{
                        if self.insert_shortcut_start || self.insert_shortcut_pause || self.insert_shortcut_blank || self.insert_shortcut_stop{
                            ui.label("Press the shortcut, max 3 key");
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
                            if let egui::Event::Key { key, pressed, .. } = event{
                                if self.insert_shortcut_start || self.insert_shortcut_pause || self.insert_shortcut_blank || self.insert_shortcut_stop{
                                    if *pressed{
                                        if self.vec_keys.len()<3{
                                            self.vec_keys.push(key.clone());
                                        }
                                        else{
                                            self.vec_keys.remove(0);
                                            self.vec_keys.push(key.clone());
                                        }    
                                    }
                            }
                            }
                        }
                        
                    });    
                                    
                    ctx.request_repaint();
                },
            }
        });

    }
}
