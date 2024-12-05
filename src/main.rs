#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod enums;
mod screen;
use crate::enums::StreamingState;
use eframe::egui::Rounding;
use eframe::egui::{
    self, Color32, ColorImage, KeyboardShortcut, ModifierNames, Modifiers, PointerButton, Pos2,
    Rect,
};
use local_ip_address::local_ip;
use screen::capture::capture::*;
use screen::net::net::*;
use screen::screen::loop_logic;
use screen::screen::ScreenState;
use std::sync::Arc;

#[derive(PartialEq, Debug, Default)]
enum CastRecEnum {
    #[default]
    Caster,
    Receiver,
}

#[derive(Default)]
enum Pages {
    #[default]
    HOME,
    CASTER,
    SETTING,
    RECEIVER,
}

#[derive(Default, PartialEq)]
enum Drawing {
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
    style
        .text_styles
        .get_mut(&egui::TextStyle::Body)
        .unwrap()
        .size = 15.0; // Cambia la dimensione del font a 24
    ctx.set_style(style);

    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../Hack-Regular.ttf")),
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
    pause_shortcut: Option<KeyboardShortcut>,
    blank_shortcut: Option<KeyboardShortcut>,
    stop_shortcut: Option<KeyboardShortcut>,
    insert_shortcut_start: bool,
    insert_shortcut_pause: bool,
    insert_shortcut_blank: bool,
    insert_shortcut_stop: bool,
    my_enum: CastRecEnum,
    server_address: String,
    state: Arc<ScreenState>,
    flag_thread: bool,
    x: String,
    y: String,
    f: String,
    line_annotations: Vec<(egui::Pos2, egui::Pos2)>,
    circle_annotations: Vec<(egui::Pos2, egui::Pos2)>,
    text_annotation: Vec<(egui::Pos2, String)>,
    is_drawing: bool,
    drawings: Drawing,
    monitor_number: u8,
    img_rect: Option<Rect>,
    annotation_color: Color32,
}

impl MyApp {
    fn handle_mouse_input(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers: _,
                } = event
                {
                    if *pressed && *button == PointerButton::Primary {
                        let rect = self.img_rect.unwrap();
                        let min = rect.min;
                        let max = rect.max;
                        let w = max.x - min.x;
                        let h = max.y - min.y;
                        self.is_drawing = true;
                        match self.drawings {
                            Drawing::NONE => {}
                            Drawing::LINE => {
                                let x1 = (pos.x - min.x) / w;
                                let y1 = (pos.y - min.y) / h;
                                self.line_annotations
                                    .push((Pos2 { x: x1, y: y1 }, Pos2 { x: x1, y: y1 }));
                            }
                            Drawing::CIRCLE => {
                                let x1 = (pos.x - min.x) / w;
                                let y1 = (pos.y - min.y) / h;
                                self.circle_annotations
                                    .push((Pos2 { x: x1, y: y1 }, Pos2 { x: x1, y: y1 }));
                            }
                            Drawing::TEXT => {}
                        }
                    } else if !*pressed {
                        self.is_drawing = false;
                    }
                };
                if let egui::Event::PointerMoved(pos) = event {
                    if self.is_drawing == true {
                        let rect = self.img_rect.unwrap();
                        let min = rect.min;
                        let max = rect.max;
                        let w = max.x - min.x;
                        let h = max.y - min.y;

                        match self.drawings {
                            Drawing::NONE => {}
                            Drawing::LINE => {
                                if let Some(last_ann) = self.line_annotations.last_mut() {
                                    let x2 = (pos.x - min.x) / w;
                                    let y2 = (pos.y - min.y) / h;
                                    last_ann.1 = Pos2 { x: x2, y: y2 };
                                }
                            }
                            Drawing::CIRCLE => {
                                if let Some(last_ann) = self.circle_annotations.last_mut() {
                                    let x2 = (pos.x - min.x) / w;
                                    let y2 = (pos.y - min.y) / h;
                                    last_ann.1 = Pos2 { x: x2, y: y2 };
                                }
                            }
                            Drawing::TEXT => {}
                        }
                    }
                }
            }
        });
    }

    fn handle_text_input(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            for event in &i.raw.events {
                let rect = self.img_rect.unwrap();
                let min = rect.min;
                let max = rect.max;
                let w = max.x - min.x;
                let h = max.y - min.y;

                if let egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers: _,
                } = event
                {
                    if *pressed && *button == PointerButton::Primary {
                        let x2 = (pos.x - min.x) / w;
                        let y2 = (pos.y - min.y) / h;
                        self.text_annotation
                            .push((Pos2 { x: x2, y: y2 }, String::new()));
                    }
                };
                if let egui::Event::Text(text) = event {
                    if let Some(last_ann) = self.text_annotation.last_mut() {
                        last_ann.1.push_str(&text);
                    }
                }
            }
        });
    }

    fn start_cast_function(&mut self) {
        if !self.flag_thread {
            let my_local_ip = local_ip().unwrap();
            self.state.set_ip_rec(my_local_ip.to_string() + ":7878");

            let server = Server::new(my_local_ip.to_string() + ":7878");
            let state_clone1 = self.state.clone();
            let _ = server.bind_to_ip(state_clone1);

            self.state.set_server(Some(server));

            self.state.set_screen_state(StreamingState::START);
            self.flag_thread = true;

            let state_clone = self.state.clone();
            std::thread::spawn(move || {
                let _ = loop_logic("caster".to_string(), state_clone);
            });
        } else {
            self.state.set_screen_state(StreamingState::START);
            self.state.cv.notify_all();
        }
    }

    fn start_rec_function(&mut self) {
        self.state.drop_client();

        let client = Client::new(self.server_address.clone());
        if let Ok(stream) = client.connect_to_ip() {
            self.state.set_client(Some((stream, client)));
            self.state.set_ip_send(self.server_address.clone());
            let state_clone = self.state.clone();
            self.current_page = Pages::RECEIVER;
            if !self.flag_thread {
                self.state.set_screen_state(StreamingState::START);
                self.flag_thread = true;

                std::thread::spawn(move || {
                    println!("loop logic");
                    let _ = loop_logic("receiver".to_string(), state_clone);
                });
            } else {
                self.state.set_screen_state(StreamingState::START);
                self.state.cv.notify_all();
            }
        } else {
            self.current_page = Pages::HOME;
        }
    }

    fn screenshot(&mut self) -> ColorImage {
        let st = self.state.clone();

        let img = st.get_frame();

        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels)
    }

    fn take_setting_icon(&self, path: &str) -> ColorImage {
        let img = image::open(path).expect("Image does not exist");

        let img_buf = img.into_rgba8();
        let (height, width) = img_buf.dimensions();
        let pixels = img_buf.into_raw();

        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels)
    }

    fn get_mod_simbol(&self, modifier: Modifiers) -> String {
        let modnames = ModifierNames::NAMES;
        let modnamesformat = modnames.format(&modifier, false);
        modnamesformat
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        setup_custom_fonts(&ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.get_sc_state() == StreamingState::STOP {
                self.annotation_color = Color32::from_rgb(255, 0, 0)
            }

            let mut shapes: Vec<egui::Shape> = Vec::new();

            let button_width = ui.available_width() / 5.0;
            let button_height = ui.available_height() / 8.0;
            if let Some(screenshot) = &self.screenshot {
                self.texture = Some(ui.ctx().load_texture(
                    "screenshot",
                    screenshot.clone(),
                    Default::default(),
                ));
            } else {
                self.texture = None;
            }
            let setting_img = Some(ui.ctx().load_texture(
                "settings",
                self.take_setting_icon("settings.png").clone(),
                Default::default(),
            ))
            .unwrap();

            let back_img = Some(ui.ctx().load_texture(
                "back",
                self.take_setting_icon("back.png").clone(),
                Default::default(),
            ))
            .unwrap();

            match self.current_page {
                Pages::HOME => {
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            ui.heading("ScreenCast Application");

                            ui.add_space(30.0);

                            ui.label("Seleziona Modalità Operativa:");

                            ui.add_space(30.0);
                        },
                    );

                    ui.horizontal(|ui| {
                        let cast_button = egui::Button::new("Caster")
                            .min_size(egui::vec2(ui.available_width() / 2.0, button_height))
                            .fill(if self.my_enum == CastRecEnum::Caster {
                                Color32::from_rgb(255, 70, 70)
                            } else {
                                Color32::LIGHT_RED
                            });
                        if ui.add(cast_button).clicked() {
                            self.my_enum = CastRecEnum::Caster;
                        };

                        let rec_button = egui::Button::new("Receiver")
                            .min_size(egui::vec2(ui.available_width(), button_height))
                            .fill(if self.my_enum == CastRecEnum::Receiver {
                                Color32::from_rgb(70, 70, 255)
                            } else {
                                Color32::LIGHT_BLUE
                            });
                        if ui.add(rec_button).clicked() {
                            self.my_enum = CastRecEnum::Receiver;
                        };
                    });

                    ui.add_space(40.0);

                    match self.my_enum {
                        CastRecEnum::Caster => {
                            ui.horizontal(|ui| {
                                ui.add_space(ui.available_width() / 4.0);
                                ui.label("Indice monitor:");
                                let monitor = get_monitors();
                                egui::ComboBox::from_label("")
                                    .selected_text(format!("{}", self.monitor_number))
                                    .show_ui(ui, |ui| {
                                        for num in 0..monitor.len() as u8 {
                                            ui.selectable_value(
                                                &mut self.monitor_number,
                                                num,
                                                format!("{num}"),
                                            );
                                        }
                                    });

                                self.state.set_n_monitor(self.monitor_number.clone());
                            });
                            ui.add_space(50.0);
                            ui.horizontal(|ui| {
                                ui.add_space(ui.available_width() / 3.0);
                                ui.horizontal(|ui| {
                                    let main_button = egui::Button::new("Condividi schermo")
                                        .min_size(egui::vec2(
                                            ui.available_width() / 2.0,
                                            button_height,
                                        ))
                                        .fill(Color32::from_rgb(255, 70, 70));
                                    if ui.add(main_button).clicked() {
                                        self.current_page = Pages::CASTER;
                                    };
                                });
                            });
                        }
                        CastRecEnum::Receiver => {
                            ui.horizontal(|ui| {
                                ui.add_space(ui.available_width() / 4.0);
                                ui.label("IP Server:");
                                ui.text_edit_singleline(&mut self.server_address);
                            });
                            ui.add_space(50.0);
                            ui.horizontal(|ui| {
                                ui.add_space(ui.available_width() / 3.0);
                                ui.horizontal(|ui| {
                                    let main_button = egui::Button::new("Visualizza straming")
                                        .min_size(egui::vec2(
                                            ui.available_width() / 2.0,
                                            button_height,
                                        ))
                                        .fill(Color32::from_rgb(70, 70, 255));
                                    if ui.add(main_button).clicked() {
                                        self.start_rec_function();
                                    };
                                });
                            });
                        }
                    }

                    let color = match self.my_enum {
                        CastRecEnum::Caster => egui::Color32::RED,
                        CastRecEnum::Receiver => egui::Color32::BLUE,
                    };
                    ui.add_space(10.0);

                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Modalità {:?} selezionata",
                                    self.my_enum
                                ))
                                .color(color),
                            );
                        },
                    );
                }
                Pages::RECEIVER => {
                    ui.horizontal(|ui| {
                        let stop_button = egui::Button::new("Stop Streaming")
                            .min_size(egui::vec2(button_width, button_height))
                            .fill(Color32::LIGHT_RED);
                        if ui.add(stop_button).clicked() {
                            self.state.set_screen_state(StreamingState::STOP);
                            self.screenshot = None;
                            self.flag_thread = false;
                            self.current_page = Pages::HOME;
                        }

                        let rec_button = egui::Button::new(match self.state.get_rec() {
                            Some(rec) => {
                                if rec {
                                    "Stop Record"
                                } else {
                                    "Record"
                                }
                            }
                            None => "Record",
                        })
                        .min_size(egui::vec2(button_width, button_height));
                        if ui.add(rec_button).clicked() {
                            if let Some(rec) = self.state.get_rec() {
                                if rec {
                                    self.state.set_rec(Some(false));
                                } else {
                                    self.state.set_rec(Some(true));
                                }
                            } else {
                                self.state.set_rec(Some(true));
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let setting_button = egui::ImageButton::new((
                                setting_img.id(),
                                egui::vec2(button_height / 1.7, button_height / 1.7),
                            ))
                            .rounding(30.0);
                            if ui.add(setting_button).clicked() {
                                self.current_page = Pages::SETTING;
                            }
                            let back_button = egui::ImageButton::new((
                                back_img.id(),
                                egui::vec2(button_height / 1.7, button_height / 1.7),
                            ))
                            .rounding(30.0);
                            if ui.add(back_button).clicked() {
                                self.state.set_screen_state(StreamingState::STOP);
                                self.screenshot = None;
                                self.flag_thread = false;
                                self.current_page = Pages::HOME;
                            }
                        });
                    });

                    if let Some(texture) = self.texture.as_ref() {
                        if let Some(rect) = self.img_rect {
                            let min = rect.min;
                            let max = rect.max;
                            let w = max.x - min.x;
                            let h = max.y - min.y;

                            if let Some(color) = self.state.get_color_ann() {
                                let ann_color = Color32::from_rgba_unmultiplied(
                                    color[0], color[1], color[2], color[3],
                                );

                                if let Some(ann) = self.state.get_line_ann() {
                                    for &(x1, y1, x2, y2) in &ann {
                                        let x1 = x1 * w + min.x;
                                        let y1 = y1 * h + min.y;
                                        let x2 = x2 * w + min.x;
                                        let y2 = y2 * h + min.y;
                                        shapes.push(egui::Shape::line_segment(
                                            [Pos2 { x: x1, y: y1 }, Pos2 { x: x2, y: y2 }],
                                            egui::Stroke::new(2.0, ann_color),
                                        ));
                                    }
                                }

                                if let Some(ann) = self.state.get_circle_ann() {
                                    for &(x1, y1, x2, y2) in &ann {
                                        let x1 = x1 * w + min.x;
                                        let y1 = y1 * h + min.y;
                                        let x2 = x2 * w + min.x;
                                        let y2 = y2 * h + min.y;

                                        shapes.push(egui::Shape::circle_stroke(
                                            Pos2 { x: x1, y: y1 },
                                            ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt(),
                                            egui::Stroke::new(2.0, ann_color),
                                        ));
                                    }
                                }

                                if let Some(ann) = self.state.get_text_ann() {
                                    for (x1, y1, text) in &ann {
                                        let x1 = x1 * w + min.x;
                                        let y1 = y1 * h + min.y;
                                        ui.fonts(|f| {
                                            let t = egui::Shape::text(
                                                f,
                                                Pos2 { x: x1, y: y1 },
                                                egui::Align2::CENTER_CENTER,
                                                text,
                                                egui::FontId::proportional(15.0),
                                                ann_color,
                                            );
                                            shapes.push(t);
                                        });
                                    }
                                }
                            }
                        }
                        self.img_rect = Some(ui.image((texture.id(), ui.available_size())).rect);

                        if let Some(rect) = self.img_rect {
                            let painter = ui.painter_at(rect);
                            painter.extend(shapes);
                        }
                    } else {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::BottomUp),
                            |ui| {
                                ui.spinner();
                            },
                        );
                    }

                    if self.state.get_sc_state() == StreamingState::STOP {
                        self.screenshot = None;
                        self.flag_thread = false;
                        self.current_page = Pages::HOME;
                    } else {
                        self.screenshot = Some(self.screenshot());
                    }

                    //HANDLE SHORTCUTS
                    ui.input(|i| {
                        for event in &i.raw.events {
                            if let egui::Event::Key {
                                key,
                                pressed,
                                modifiers,
                                ..
                            } = event
                            {
                                if *pressed {
                                    self.temp_shortcut = Some(egui::KeyboardShortcut::new(
                                        modifiers.clone(),
                                        key.clone(),
                                    ));
                                }
                            }
                        }
                        //check inserted shorcut
                        if let Some(sct) = self.temp_shortcut {
                            if let Some(sc) = self.stop_shortcut {
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::STOP);
                                    self.current_page = Pages::HOME;
                                    self.temp_shortcut = None;
                                }
                            }
                        }

                        self.temp_shortcut = None;
                    });

                    ctx.request_repaint();
                }
                Pages::CASTER => {
                    //HANDLE BUTTONS
                    ui.horizontal(|ui| {
                        let start_button = egui::Button::new(match self.state.get_sc_state() {
                            StreamingState::START => "Start Streaming",
                            StreamingState::PAUSE => "Resume Streming",
                            StreamingState::BLANK => "Resume Streaming",
                            StreamingState::STOP => "Start Streaming",
                        })
                        .min_size(egui::vec2(button_width, button_height))
                        .fill(
                            if self.state.get_sc_state() == StreamingState::START {
                                Color32::LIGHT_GREEN
                            } else {
                                Color32::GRAY
                            },
                        );
                        if ui.add(start_button).clicked() {
                            self.start_cast_function();
                        }

                        let pause_button = egui::Button::new("Pause Streaming")
                            .min_size(egui::vec2(button_width, button_height))
                            .fill(if self.state.get_sc_state() == StreamingState::PAUSE {
                                Color32::from_rgb(100, 100, 255)
                            } else {
                                Color32::GRAY
                            });
                        if ui.add(pause_button).clicked() {
                            self.state.set_screen_state(StreamingState::PAUSE);
                            self.state.cv.notify_all();
                        }

                        let blank_button = egui::Button::new("Blank Streaming")
                            .min_size(egui::vec2(button_width, button_height))
                            .fill(if self.state.get_sc_state() == StreamingState::BLANK {
                                Color32::WHITE
                            } else {
                                Color32::GRAY
                            });
                        if ui.add(blank_button).clicked() {
                            self.state.set_screen_state(StreamingState::BLANK);
                            self.state.cv.notify_all();
                        }

                        let stop_button = egui::Button::new("Stop Streaming")
                            .min_size(egui::vec2(button_width, button_height))
                            .rounding(Rounding::ZERO)
                            .fill(Color32::LIGHT_RED);
                        if ui.add(stop_button).clicked() {
                            self.state.set_screen_state(StreamingState::STOP);
                            self.state.set_kill_listener(true);
                            self.screenshot = None;
                            self.flag_thread = false;
                            self.current_page = Pages::HOME;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let setting_button = egui::ImageButton::new((
                                setting_img.id(),
                                egui::vec2(button_height / 1.7, button_height / 1.7),
                            ))
                            .rounding(30.0);
                            if ui.add(setting_button).clicked() {
                                self.current_page = Pages::SETTING;
                            }
                            let back_button = egui::ImageButton::new((
                                back_img.id(),
                                egui::vec2(button_height / 1.7, button_height / 1.7),
                            ))
                            .rounding(30.0);
                            if ui.add(back_button).clicked() {
                                self.state.set_screen_state(StreamingState::STOP);
                                self.screenshot = None;
                                self.flag_thread = false;
                                self.current_page = Pages::HOME;
                            }
                        });
                    });

                    //Stream Customization
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.label("Customize screen trasmetted");
                    });
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("X coordinate");
                            ui.add_space(90.0);
                            //self.x = self.state.get_x().to_string();
                            if ui.text_edit_singleline(&mut self.x).changed() {
                                if let Ok(n) = self.x.parse::<u32>() {
                                    if n < 1999 && n % 2 == 0 {
                                        self.state.set_x(n);
                                        self.x = n.to_string();
                                    } else {
                                        self.state.set_x(0);
                                        self.x = 0.to_string();
                                    }
                                } else if self.x == "".to_string() {
                                    self.x = 0.to_string();
                                    self.state.set_x(0);
                                } else {
                                    ui.label("Invalid value");
                                }
                            };
                        });

                        ui.horizontal(|ui| {
                            ui.label("Y coordinate");
                            ui.add_space(90.0);
                            //self.y = self.state.get_y().to_string();
                            if ui.text_edit_singleline(&mut self.y).changed() {
                                if let Ok(n) = self.y.parse::<u32>() {
                                    if n < 999 && n % 2 == 0 {
                                        self.state.set_y(n);
                                        self.y = n.to_string()
                                    } else {
                                        self.state.set_y(0);
                                        self.y = 0.to_string()
                                    }
                                } else if self.y == "".to_string() {
                                    self.y = 0.to_string();
                                    self.state.set_y(0);
                                } else {
                                    ui.label("Invalid value");
                                }
                            };
                        });

                        ui.horizontal(|ui| {
                            ui.label("Screen reduction (%)");
                            ui.add_space(20.0);
                            //self.f = self.state.get_f().to_string();
                            if ui.text_edit_singleline(&mut self.f).changed() {
                                if let Ok(n) = self.f.parse::<u32>() {
                                    if n <= 100 && n > 1 {
                                        self.state.set_f(n);
                                        self.f = n.to_string();
                                    } else {
                                        self.f = 0.to_string()
                                    }
                                } else if self.f == "".to_string() {
                                    self.f = 0.to_string();
                                    self.state.set_f(100);
                                } else {
                                    ui.label("Invalid value");
                                }
                            };
                        });
                    });
                    ui.add_space(10.0);
                    let my_local_ip = local_ip().unwrap();
                    ui.horizontal(|ui| {
                        ui.label("Your Server IP: ");
                        ui.label(my_local_ip.to_string() + ":7878");
                    });

                    if self.state.get_sc_state() == StreamingState::START {
                        self.screenshot = Some(self.screenshot());
                    }

                    if let Some(texture) = self.texture.as_ref() {
                        if let Some(rect) = self.img_rect {
                            let min = rect.min;
                            let max = rect.max;
                            let w = max.x - min.x;
                            let h = max.y - min.y;

                            self.state.set_line_ann(
                                self.line_annotations
                                    .iter()
                                    .map(|(p1, p2)| (p1.x, p1.y, p2.x, p2.y))
                                    .collect(),
                            );
                            self.state.set_circle_ann(
                                self.circle_annotations
                                    .iter()
                                    .map(|(p1, p2)| (p1.x, p1.y, p2.x, p2.y))
                                    .collect(),
                            );
                            self.state.set_text_ann(
                                self.text_annotation
                                    .iter()
                                    .map(|(p1, t)| (p1.x, p1.y, t.to_owned()))
                                    .collect(),
                            );

                            for &(start, end) in &self.line_annotations {
                                println!("start: {:?}, end: {:?}", start, end);
                                let x1 = start.x * w + min.x;
                                let y1 = start.y * h + min.y;
                                let x2 = end.x * w + min.x;
                                let y2 = end.y * h + min.y;
                                shapes.push(egui::Shape::line_segment(
                                    [Pos2 { x: x1, y: y1 }, Pos2 { x: x2, y: y2 }],
                                    egui::Stroke::new(2.0, self.annotation_color.clone()),
                                ));
                            }

                            for &(start, end) in &self.circle_annotations {
                                let x1 = start.x * w + min.x;
                                let y1 = start.y * h + min.y;
                                let x2 = end.x * w + min.x;
                                let y2 = end.y * h + min.y;

                                shapes.push(egui::Shape::circle_stroke(
                                    Pos2 { x: x1, y: y1 },
                                    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt(),
                                    egui::Stroke::new(2.0, self.annotation_color),
                                ));
                            }

                            for (start, text) in &mut self.text_annotation {
                                let x1 = start.x * w + min.x;
                                let y1 = start.y * h + min.y;
                                ui.fonts(|f| {
                                    let t = egui::Shape::text(
                                        f,
                                        Pos2 { x: x1, y: y1 },
                                        egui::Align2::CENTER_CENTER,
                                        text,
                                        egui::FontId::proportional(15.0),
                                        self.annotation_color,
                                    );
                                    shapes.push(t);
                                });
                            }
                        }

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            self.img_rect = Some(
                                ui.image((
                                    texture.id(),
                                    egui::vec2(
                                        ui.available_width() * 4.0 / 5.0,
                                        ui.available_height(),
                                    ),
                                ))
                                .rect,
                            );

                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Annotations");
                                });

                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.color_edit_button_srgba(&mut self.annotation_color);
                                });
                                // Set color ann
                                self.state
                                    .set_color_ann(self.annotation_color.to_srgba_unmultiplied());

                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    let draw_line_button = egui::Button::new("Line")
                                        .min_size(ui.available_size())
                                        .fill(if self.drawings == Drawing::LINE {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        });
                                    if ui.add(draw_line_button).clicked() {
                                        if self.drawings == Drawing::LINE {
                                            self.drawings = Drawing::NONE;
                                        } else {
                                            self.drawings = Drawing::LINE;
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    let draw_circle_button = egui::Button::new("Circle")
                                        .min_size(ui.available_size())
                                        .fill(if self.drawings == Drawing::CIRCLE {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        });
                                    if ui.add(draw_circle_button).clicked() {
                                        if self.drawings == Drawing::CIRCLE {
                                            self.drawings = Drawing::NONE;
                                        } else {
                                            self.drawings = Drawing::CIRCLE;
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    let draw_text_button = egui::Button::new("Text")
                                        .min_size(ui.available_size())
                                        .fill(if self.drawings == Drawing::TEXT {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        });
                                    if ui.add(draw_text_button).clicked() {
                                        if self.drawings == Drawing::TEXT {
                                            self.drawings = Drawing::NONE;
                                        } else {
                                            self.drawings = Drawing::TEXT;
                                        }
                                    }
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    let clear_button = egui::Button::new("Clear All")
                                        .min_size(ui.available_size())
                                        .fill(Color32::LIGHT_RED);
                                    if ui.add(clear_button).clicked() {
                                        self.line_annotations.clear();
                                        self.circle_annotations.clear();
                                        self.text_annotation.clear();
                                        shapes.clear();
                                    }
                                });
                            });

                            if let Some(rect) = self.img_rect {
                                let painter = ui.painter_at(rect);

                                if self.drawings != Drawing::NONE {
                                    painter.extend(shapes);
                                }
                            }
                        });
                    } else {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::BottomUp),
                            |ui| {
                                ui.spinner();
                            },
                        );
                    }

                    match self.drawings {
                        Drawing::NONE => {}
                        Drawing::LINE => {
                            self.handle_mouse_input(ui);
                        }
                        Drawing::CIRCLE => {
                            self.handle_mouse_input(ui);
                        }
                        Drawing::TEXT => {
                            self.handle_text_input(ui);
                        }
                    }

                    //HANDLE SHORTCUTS
                    ui.input(|i| {
                        for event in &i.raw.events {
                            if let egui::Event::Key {
                                key,
                                pressed,
                                modifiers,
                                ..
                            } = event
                            {
                                if *pressed {
                                    self.temp_shortcut = Some(egui::KeyboardShortcut::new(
                                        modifiers.clone(),
                                        key.clone(),
                                    ));
                                }
                            }
                        }
                        //check inserted shorcut
                        if let Some(sct) = self.temp_shortcut {
                            if let Some(sc) = self.start_shortcut {
                                if sct == sc {
                                    self.start_cast_function();
                                    self.temp_shortcut = None;
                                }
                            }
                            if let Some(sc) = self.pause_shortcut {
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::PAUSE);
                                    self.temp_shortcut = None;
                                }
                            }
                            if let Some(sc) = self.blank_shortcut {
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::BLANK);
                                    self.temp_shortcut = None;
                                }
                            }
                            if let Some(sc) = self.stop_shortcut {
                                if sct == sc {
                                    self.state.set_screen_state(StreamingState::STOP);
                                    self.current_page = Pages::HOME;
                                    self.temp_shortcut = None;
                                }
                            }
                        }

                        self.temp_shortcut = None;
                    });
                    ctx.request_repaint();
                }
                Pages::SETTING => {
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.y = button_width / 4.0;
                            ui.heading("Shotcut Settings");

                            ui.horizontal(|ui| {
                                //Visualize shortcut buttons
                                let add_close_butt =
                                    egui::Button::new(if self.insert_shortcut_start {
                                        "End"
                                    } else {
                                        "Add"
                                    })
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(
                                        if self.insert_shortcut_start {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        },
                                    );
                                let clear_butt = egui::Button::new("Clear")
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(if self.insert_shortcut_start {
                                        Color32::LIGHT_RED
                                    } else {
                                        Color32::GRAY
                                    });
                                ui.add_space(30.0);
                                if ui.add(add_close_butt).clicked()
                                    && !self.insert_shortcut_blank
                                    && !self.insert_shortcut_pause
                                    && !self.insert_shortcut_stop
                                {
                                    self.insert_shortcut_start = !self.insert_shortcut_start;
                                    if !self.insert_shortcut_start {
                                        self.start_shortcut = self.temp_shortcut.clone();
                                        self.temp_shortcut = None;
                                    };
                                }
                                if ui.add(clear_butt).clicked() {
                                    self.start_shortcut = None;
                                }

                                //Visualize Shotcut String
                                ui.add_space(30.0);
                                ui.label(format!("Start Streaming Shortcut : "));
                                if self.insert_shortcut_start {
                                    if let Some(sc) = self.temp_shortcut {
                                        ui.label(format!(
                                            "{:}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                } else {
                                    if let Some(sc) = self.start_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                }
                            });

                            ui.spacing_mut().item_spacing.y = button_width / 4.0;

                            ui.horizontal(|ui| {
                                //Visualize shortcut buttons
                                let add_close_butt =
                                    egui::Button::new(if self.insert_shortcut_pause {
                                        "End"
                                    } else {
                                        "Add"
                                    })
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(
                                        if self.insert_shortcut_pause {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        },
                                    );
                                let clear_butt = egui::Button::new("Clear")
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(if self.insert_shortcut_pause {
                                        Color32::LIGHT_RED
                                    } else {
                                        Color32::GRAY
                                    });
                                ui.add_space(30.0);
                                if ui.add(add_close_butt).clicked()
                                    && !self.insert_shortcut_blank
                                    && !self.insert_shortcut_start
                                    && !self.insert_shortcut_stop
                                {
                                    self.insert_shortcut_pause = !self.insert_shortcut_pause;
                                    if !self.insert_shortcut_pause {
                                        self.pause_shortcut = self.temp_shortcut.clone();
                                        self.temp_shortcut = None;
                                    };
                                }
                                if ui.add(clear_butt).clicked() {
                                    self.pause_shortcut = None;
                                }

                                //Visualize Shotcut String
                                ui.add_space(30.0);
                                ui.label(format!("Pause Streaming Shortcut : "));
                                if self.insert_shortcut_pause {
                                    if let Some(sc) = self.temp_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                } else {
                                    if let Some(sc) = self.pause_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                }
                            });

                            ui.spacing_mut().item_spacing.y = button_width / 4.0;

                            ui.horizontal(|ui| {
                                //Visualize shortcut buttons
                                let add_close_butt =
                                    egui::Button::new(if self.insert_shortcut_blank {
                                        "End"
                                    } else {
                                        "Add"
                                    })
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(
                                        if self.insert_shortcut_blank {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        },
                                    );
                                let clear_butt = egui::Button::new("Clear")
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(if self.insert_shortcut_blank {
                                        Color32::LIGHT_RED
                                    } else {
                                        Color32::GRAY
                                    });
                                ui.add_space(30.0);
                                if ui.add(add_close_butt).clicked()
                                    && !self.insert_shortcut_start
                                    && !self.insert_shortcut_pause
                                    && !self.insert_shortcut_stop
                                {
                                    self.insert_shortcut_blank = !self.insert_shortcut_blank;
                                    if !self.insert_shortcut_blank {
                                        self.blank_shortcut = self.temp_shortcut.clone();
                                        self.temp_shortcut = None;
                                    };
                                }
                                if ui.add(clear_butt).clicked() {
                                    self.blank_shortcut = None;
                                }

                                //Visualize Shotcut String
                                ui.add_space(30.0);
                                ui.label(format!("Blank Streaming Shortcut : "));
                                if self.insert_shortcut_blank {
                                    if let Some(sc) = self.temp_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                } else {
                                    if let Some(sc) = self.blank_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                }
                            });

                            ui.spacing_mut().item_spacing.y = button_width / 4.0;

                            ui.horizontal(|ui| {
                                //Visualize shortcut buttons
                                let add_close_butt =
                                    egui::Button::new(if self.insert_shortcut_stop {
                                        "End"
                                    } else {
                                        "Add"
                                    })
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(
                                        if self.insert_shortcut_stop {
                                            Color32::LIGHT_GREEN
                                        } else {
                                            Color32::GRAY
                                        },
                                    );
                                let clear_butt = egui::Button::new("Clear")
                                    .min_size(egui::vec2(button_width / 2.0, button_height / 2.0))
                                    .fill(if self.insert_shortcut_stop {
                                        Color32::LIGHT_RED
                                    } else {
                                        Color32::GRAY
                                    });
                                ui.add_space(30.0);
                                if ui.add(add_close_butt).clicked()
                                    && !self.insert_shortcut_blank
                                    && !self.insert_shortcut_pause
                                    && !self.insert_shortcut_start
                                {
                                    self.insert_shortcut_stop = !self.insert_shortcut_stop;
                                    if !self.insert_shortcut_stop {
                                        self.stop_shortcut = self.temp_shortcut.clone();
                                        self.temp_shortcut = None;
                                    };
                                }
                                if ui.add(clear_butt).clicked() {
                                    self.stop_shortcut = None;
                                }

                                //Visualize Shotcut String
                                ui.add_space(30.0);
                                ui.label(format!("Stop Streaming Shortcut : "));
                                if self.insert_shortcut_stop {
                                    if let Some(sc) = self.temp_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                } else {
                                    if let Some(sc) = self.stop_shortcut {
                                        ui.label(format!(
                                            "{}+{:?}",
                                            self.get_mod_simbol(sc.modifiers),
                                            sc.logical_key
                                        ));
                                    }
                                }
                            });
                            ui.spacing_mut().item_spacing.y = button_width / 8.0;
                        },
                    );

                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::BottomUp),
                        |ui| {
                            if self.insert_shortcut_start
                                || self.insert_shortcut_pause
                                || self.insert_shortcut_blank
                                || self.insert_shortcut_stop
                            {
                                ui.label("Press the shortcut");
                            }
                        },
                    );

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        let back_button =
                            egui::ImageButton::new((back_img.id(), egui::vec2(35.0, 35.0)))
                                .rounding(30.0);
                        if ui.add(back_button).clicked() {
                            match self.my_enum {
                                CastRecEnum::Caster => {
                                    self.current_page = Pages::CASTER;
                                }
                                CastRecEnum::Receiver => {
                                    self.current_page = Pages::RECEIVER;
                                }
                            }
                            self.state.set_screen_state(StreamingState::STOP);
                        }
                    });

                    ui.input(|i| {
                        for event in &i.raw.events {
                            if let egui::Event::Key {
                                key,
                                pressed,
                                modifiers,
                                ..
                            } = event
                            {
                                if self.insert_shortcut_start
                                    || self.insert_shortcut_pause
                                    || self.insert_shortcut_blank
                                    || self.insert_shortcut_stop
                                {
                                    if *pressed {
                                        self.temp_shortcut = Some(egui::KeyboardShortcut::new(
                                            modifiers.clone(),
                                            key.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    });

                    //ctx.request_repaint();
                }
            }
        });
    }
}
