use crate::iced_logic;
use crate::iced_logic::draw_rectangle;
use crate::iced_logic::get_bottom_right;
use crate::iced_logic::get_top_left;
use crate::iced_logic::UPoint;
use crate::options;
use crate::options::Settings;
use iced::event;
use iced::executor;
use iced::widget::button;
use iced::widget::image::Handle;
use iced::widget::mouse_area;
use iced::widget::text;
use iced::window::Id;
use iced::Application;
use iced::Command;
use iced::ContentFit;
use iced::Element;
use iced::Point;
use iced::Renderer;
use iced::Size;
use iced::Subscription;
use iced::Theme;
use mouse_position::mouse_position::Mouse;
use ocrs::ImageSource;
use ocrs::OcrEngine;
use tts::Tts;
use xcap::Monitor;

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Read,
    Stop,
    StartRect,
    EndRect,
    MouseMoved(UPoint),
}

pub struct IcedApp {
    pub engine: OcrEngine,
    pub tts: Tts,
    pub screenshot_buffer: Vec<u8>,
    pub screenshot_size: (u32, u32),
    pub rect_start: Option<UPoint>,
    pub rect_end: Option<UPoint>,

    pub screenshot_image: Option<Vec<u8>>,

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
            button("READ").on_press(Message::Read).into()
        } else if let Ok(true) = self.tts.is_speaking() {
            button("STOP").on_press(Message::Stop).into()
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
                iced::window::resize(Id::MAIN, Size::new(65., 32.))
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
        }
    }
    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        (
            Self::default(),
            iced::window::resize(Id::MAIN, Size::new(65., 32.)),
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
        Self {
            engine: iced_logic::init_engine(),
            tts: iced_logic::init_tts(),
            screenshot_buffer: vec![],
            screenshot_size: (0, 0),
            rect_start: None,
            rect_end: None,

            screenshot_image: None,

            settings: Settings::default(),
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
