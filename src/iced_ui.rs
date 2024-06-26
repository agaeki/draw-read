// This file is part of draw-read
// Copyright (C) 2024 agaeki

use crate::iced_logic;
use crate::iced_logic::draw_rectangle;
use crate::iced_logic::get_bottom_right;
use crate::iced_logic::get_top_left;
use crate::iced_logic::UPoint;
use crate::options;
use crate::options::Settings;
use crate::options::VoicePitch;
use crate::options::VoiceRate;
use iced::alignment::Horizontal;
use iced::event;
use iced::executor;
use iced::widget;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container::Appearance;
use iced::widget::horizontal_rule;
use iced::widget::image::Handle;
use iced::widget::mouse_area;
use iced::widget::row;
use iced::widget::text;
use iced::widget::vertical_rule;
use iced::window::Id;
use iced::Application;
use iced::Color;
use iced::Command;
use iced::ContentFit;
use iced::Element;
use iced::Point;
use iced::Size;
use iced::Subscription;
use iced::Theme;
use mouse_position::mouse_position::Mouse;
use ocrs::ImageSource;
use ocrs::OcrEngine;
use rfd::FileDialog;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;
use tts::Tts;
use xcap::Monitor;

pub const WINDOW_SIZE: Size = Size::new(100., 32.);
pub const WINDOW_SIZE_SETTINGS: Size = Size::new(200., 400.);

#[derive(Clone)]
pub enum Message {
    Read,
    Stop,
    StartRect,
    EndRect,
    MouseMoved(UPoint),
    Settings,
    SettingsCancel,
    SettingsApply,
    SettingChanged(Arc<dyn Fn(&mut Settings) + Send + Sync>),
    SettingError(String),
    DragWindow,
    ReleaseWindow,
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Message").finish()
    }
}

pub struct IcedApp {
    pub engine: OcrEngine,
    pub tts: Tts,
    pub screenshot_buffer: Vec<u8>,
    pub screenshot_size: (u32, u32),
    pub rect_start: Option<UPoint>,
    pub rect_end: Option<UPoint>,

    pub screenshot_image: Option<Vec<u8>>,
    pub settings_open: bool,

    pub settings: options::Settings,
}

impl Application for IcedApp {
    //type Renderer = Renderer;
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn view(&self) -> Element<'_, Message> {
        if let Some(screenshot_image) = &self.screenshot_image {
            mouse_area(
                iced::widget::image(Handle::from_pixels(
                    self.screenshot_size.0,
                    self.screenshot_size.1,
                    screenshot_image.clone(),
                ))
                .content_fit(ContentFit::None),
            )
            .on_press(Message::StartRect)
            .on_release(Message::EndRect)
            .into()
        } else if let Ok(false) = self.tts.is_speaking() {
            column([
                row([
                    button(widget::image(Handle::from_memory(include_bytes!(
                        "gear_image.png"
                    ))))
                    .on_press(Message::Settings)
                    .into(),
                    button("READ").on_press(Message::Read).into(),
                ])
                .into(),
                settings_widget(&self),
            ])
            .into()
        } else if let Ok(true) = self.tts.is_speaking() {
            column([
                row([
                    button(widget::image(Handle::from_memory(include_bytes!(
                        "gear_image.png"
                    ))))
                    .on_press(Message::Settings)
                    .into(),
                    button("STOP").on_press(Message::Stop).into(),
                ])
                .into(),
                settings_widget(&self),
            ])
            .into()
        } else {
            eprintln!("ERROR: {:?}", self.tts.is_speaking());
            text("ERROR").into()
        }
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::Read => {
                println!("Read clicked");
                if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
                    let monitor = &Monitor::from_point(x, y).unwrap();
                    let rgb_image = monitor.capture_image().unwrap();

                    let ret = Command::batch([
                        iced::window::resize(
                            Id::MAIN,
                            Size::new(rgb_image.width() as f32, rgb_image.height() as f32),
                        ),
                        iced::window::move_to(Id::MAIN, Point::new(0., 0.)),
                    ]);

                    self.screenshot_size = (rgb_image.width(), rgb_image.height());

                    // Store original screenshot so that we can draw a resizing rectangle on a clone without losing pixels
                    self.screenshot_buffer = rgb_image.into_raw();

                    self.screenshot_image = Some(self.screenshot_buffer.clone());

                    ret
                } else {
                    Command::none()
                }
            }
            Message::StartRect => {
                if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
                    println!("Start rect at {x} {y}");
                    self.rect_start = Some(UPoint {
                        x: x as u32,
                        y: y as u32,
                    });
                }
                Command::none()
            }
            Message::EndRect => {
                if let Some(rect_start) = self.rect_start
                    && let Some(rect_end) = self.rect_end
                {
                    self.speak_from_rect(
                        self.screenshot_buffer.clone(),
                        self.screenshot_size,
                        rect_start,
                        rect_end,
                    );
                }
                self.rect_start = None;
                self.screenshot_image = None;
                Command::batch([
                    iced::window::resize(Id::MAIN, WINDOW_SIZE),
                    iced::window::move_to(Id::MAIN, self.settings.position.into()),
                ])
            }
            Message::MouseMoved(pos) => {
                self.rect_end = Some(pos);
                if let Some(rect_start) = self.rect_start
                    && let Some(rect_end) = self.rect_end
                    && let Some(screenshot_image) = &mut self.screenshot_image
                {
                    screenshot_image.copy_from_slice(&self.screenshot_buffer[..]);
                    draw_rectangle(
                        &mut self.screenshot_image.as_mut().unwrap(),
                        self.screenshot_size,
                        get_top_left(rect_start, rect_end),
                        get_bottom_right(rect_start, rect_end),
                        &self.settings.rect_colour,
                    );
                }
                Command::none()
            }
            Message::Stop => {
                if let Err(e) = self.tts.stop() {
                    eprintln!("Error stopping speaking: {:?}", e);
                }
                Command::none()
            }
            Message::Settings => {
                self.settings_open = !self.settings_open;
                iced::window::resize(
                    Id::MAIN,
                    if self.settings_open {
                        WINDOW_SIZE_SETTINGS
                    } else {
                        WINDOW_SIZE
                    },
                )
            }
            Message::SettingChanged(set_function) => {
                set_function(&mut self.settings);
                Command::none()
            }
            Message::DragWindow | Message::ReleaseWindow => todo!(),
            Message::SettingsCancel => {
                self.settings = Settings::default();
                self.update(Message::Settings)
            }
            Message::SettingError(e) => {
                eprintln!("Error from settings: {:?}", e);
                Command::none()
            }
            Message::SettingsApply => {
                let _ = self.settings.save_to_file();
                self.engine = iced_logic::init_engine(&self.settings);
                self.tts = iced_logic::init_tts(&self.settings);
                self.update(Message::Settings)
            }
        }
    }
    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        (
            Self::default(),
            Command::batch([
                iced::window::resize(Id::MAIN, WINDOW_SIZE),
                iced::window::move_to(Id::MAIN, Self::default().settings.position.into()),
            ]),
        )
    }
    fn title(&self) -> std::string::String {
        "".to_string()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|evt, _| {
            if let iced::Event::Mouse(iced::mouse::Event::CursorMoved {
                position: Point { x, y },
            }) = evt
            {
                Some(Message::MouseMoved(UPoint {
                    x: x as u32,
                    y: y as u32,
                }))
            } else {
                None
            }
        })
    }
}

impl Default for IcedApp {
    fn default() -> Self {
        let settings = Settings::default();
        Self {
            engine: iced_logic::init_engine(&settings),
            tts: iced_logic::init_tts(&settings),
            screenshot_buffer: vec![],
            screenshot_size: (0, 0),
            rect_start: None,
            rect_end: None,

            screenshot_image: None,

            settings: settings,
            settings_open: false,
        }
    }
}

impl IcedApp {
    fn speak_from_rect(
        &mut self,
        screenshot: Vec<u8>,
        screenshot_size: (u32, u32),
        first_corner: UPoint,
        second_corner: UPoint,
    ) {
        let (img_source_bytes, new_width, new_height) = iced_logic::get_cropped_image_source(
            screenshot,
            screenshot_size,
            first_corner,
            second_corner,
        );
        let img_source =
            ImageSource::from_bytes(&img_source_bytes[..], (new_width, new_height)).unwrap();

        let ocr_input = self.engine.prepare_input(img_source).unwrap();

        // Get oriented bounding boxes of text words in input image.
        let word_rects = self.engine.detect_words(&ocr_input).unwrap();

        // Group words into lines. Each line is represented by a list of word
        // bounding boxes.
        let line_rects = self.engine.find_text_lines(&ocr_input, &word_rects);

        let words = self
            .engine
            .recognize_text(&ocr_input, &line_rects[..])
            .unwrap()
            .into_iter()
            .map(|x| x.map_or("".to_string(), |x| x.to_string()))
            .collect::<Vec<_>>()
            .join(" ");

        println!("Speaking {words}");
        self.tts.speak(words, false).unwrap();
    }
}

fn settings_widget(app: &IcedApp) -> Element<'_, Message> {
    if app.settings_open {
        let rect_colour = app.settings.rect_colour.clone();
        column([
            // top rule
            horizontal_rule(2).into(),
            row([
                widget::button("CANCEL")
                    .on_press(Message::SettingsCancel)
                    .into(),
                widget::button("APPLY")
                    .on_press(Message::SettingsApply)
                    .into(),
            ])
            .into(),
            // Rate slider
            row([
                widget::text(app.settings.rate)
                    .width(70)
                    .horizontal_alignment(Horizontal::Right)
                    .into(),
                vertical_rule(2).into(),
                iced::widget::slider(
                    VoiceRate::Slowest..=VoiceRate::TooFast,
                    app.settings.rate,
                    |new_value| {
                        if new_value != app.settings.rate {
                            println!("Setting s.rate to {:?}", new_value);
                            Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                                s.rate = new_value
                            }))
                        } else {
                            Message::SettingChanged(Arc::new(move |_: &mut Settings| {}))
                        }
                    },
                )
                .into(),
            ])
            .into(),
            // Pitch slider
            row([
                widget::text(app.settings.pitch)
                    .width(70)
                    .horizontal_alignment(Horizontal::Right)
                    .into(),
                vertical_rule(2).into(),
                iced::widget::slider(
                    VoicePitch::Soprano..=VoicePitch::Bass,
                    app.settings.pitch,
                    |new_value| {
                        if new_value != app.settings.pitch {
                            println!("Setting s.pitch to {:?}", new_value);
                            Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                                s.pitch = new_value
                            }))
                        } else {
                            Message::SettingChanged(Arc::new(move |_: &mut Settings| {}))
                        }
                    },
                )
                .into(),
            ])
            .into(),
            // Volume slider
            row([
                widget::text(app.settings.volume)
                    .width(70)
                    .horizontal_alignment(Horizontal::Right)
                    .into(),
                vertical_rule(2).into(),
                iced::widget::slider(0..=255, app.settings.volume, |new_value| {
                    if new_value != app.settings.volume {
                        println!("Setting s.volume to {:?}", new_value);
                        Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                            s.volume = new_value
                        }))
                    } else {
                        Message::SettingChanged(Arc::new(move |_: &mut Settings| {}))
                    }
                })
                .into(),
            ])
            .into(),
            // Rect colour picker
            row([
                widget::container(
                    widget::text("Line Colour")
                        .width(70)
                        .horizontal_alignment(Horizontal::Right),
                )
                .style(move |_theme: &Theme| Appearance {
                    background: Some(iced::Background::Color(Color::from_rgb8(
                        rect_colour[0],
                        rect_colour[1],
                        rect_colour[2],
                    ))),
                    border: iced::Border::with_radius(1),
                    shadow: iced::Shadow::default(),
                    text_color: Some(
                        Color::from_rgb8(rect_colour[0], rect_colour[1], rect_colour[2]).inverse(),
                    ),
                })
                .into(),
                vertical_rule(2).into(),
                widget::text_input("Red", &app.settings.rect_colour[0].to_string())
                    .on_input(|new_value| {
                        if let Ok(new_u) = new_value.parse::<u8>() {
                            println!("Setting s.rect_colour[0] to {:?}", new_u);
                            Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                                s.rect_colour[0] = new_u
                            }))
                        } else {
                            Message::SettingError(
                                "Colour components must be a number 0..255".to_string(),
                            )
                        }
                    })
                    .into(),
                widget::text_input("Green", &app.settings.rect_colour[1].to_string())
                    .on_input(|new_value| {
                        if let Ok(new_u) = new_value.parse::<u8>() {
                            println!("Setting s.rect_colour[1] to {:?}", new_u);
                            Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                                s.rect_colour[1] = new_u
                            }))
                        } else {
                            Message::SettingError(
                                "Colour components must be a number 0..255".to_string(),
                            )
                        }
                    })
                    .into(),
                widget::text_input("Blue", &app.settings.rect_colour[2].to_string())
                    .on_input(|new_value| {
                        if let Ok(new_u) = new_value.parse::<u8>() {
                            println!("Setting s.rect_colour[2] to {:?}", new_u);
                            Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                                s.rect_colour[2] = new_u
                            }))
                        } else {
                            Message::SettingError(
                                "Colour components must be a number 0..255".to_string(),
                            )
                        }
                    })
                    .into(),
            ])
            .into(),
            // Voice picker
            row([iced::widget::pick_list(
                app.tts
                    .voices()
                    .unwrap()
                    .into_iter()
                    .map(|v| PickableVoice(v))
                    .collect::<Vec<_>>(),
                Some(PickableVoice(
                    app.tts
                        .voices()
                        .unwrap()
                        .into_iter()
                        .find(|v| v.id() == app.settings.voice)
                        .unwrap_or(app.tts.voices().unwrap()[0].clone()),
                )),
                |new_value| {
                    if new_value.0.id() != app.settings.voice {
                        println!("Setting s.voice to {:?}", new_value);
                        Message::SettingChanged(Arc::new(move |s: &mut Settings| {
                            s.voice = new_value.0.id()
                        }))
                    } else {
                        Message::SettingChanged(Arc::new(move |_: &mut Settings| {}))
                    }
                },
            )
            .width(200)
            .into()])
            .into(),
            // Detection model picker
            row([iced::widget::button(
                iced::widget::text_input(
                    "OCR Detection model",
                    &app.settings.detection_file.to_string_lossy(),
                )
                .width(200),
            )
            .on_press(Message::SettingChanged(Arc::new(
                move |s: &mut Settings| {
                    if let Some(file) = FileDialog::new().add_filter("text", &["rten"]).pick_file()
                    {
                        s.detection_file = file;
                    }
                },
            )))
            .into()])
            .into(),
            // Recognition model picker
            row([iced::widget::button(
                iced::widget::text_input(
                    "OCR Recognition model",
                    &app.settings.recognition_file.to_string_lossy(),
                )
                .width(200),
            )
            .on_press(Message::SettingChanged(Arc::new(
                move |s: &mut Settings| {
                    if let Some(file) = FileDialog::new().add_filter("text", &["rten"]).pick_file()
                    {
                        s.recognition_file = file;
                    }
                },
            )))
            .into()])
            .into(),
            // Position picker
            row([iced::widget::mouse_area(iced::widget::text(
                "Click & Drag here to move window(TODO)",
            ))
            .on_press(Message::DragWindow)
            .on_release(Message::ReleaseWindow)
            .interaction(iced::mouse::Interaction::Grab)
            .into()])
            .into(),
        ])
        .into()
    } else {
        horizontal_rule(0).into()
    }
}

#[derive(PartialEq, Clone, Debug)]
struct PickableVoice(tts::Voice);
impl Display for PickableVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&self.0.name())
    }
}
