use iced::advanced::Application;
use iced::event;
use iced::executor;
use iced::widget::button;
use iced::widget::clickable;
use iced::widget::image::Handle;
use iced::window::Id;
use iced::Command;
use iced::ContentFit;
use iced::Element;
use iced::Point;
use iced::Renderer;
use iced::Size;
use iced::Subscription;
use iced::Theme;
use image::imageops;
use image::ImageBuffer;
use mouse_position::mouse_position::Mouse;
use ocrs::ImageSource;
use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use rten::Model;
use std::fs;
use tts::Features;
use tts::Tts;
use xcap::Monitor;

static RECT_COLOUR: [u8; 4] = [0, 255, 0, 255];

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Read,
    StartRect,
    EndRect,
    MouseMoved(UPoint),
}

#[derive(Clone, Debug, Copy)]
pub struct UPoint {
    x: u32,
    y: u32,
}

pub struct IcedApp {
    pub engine: OcrEngine,
    pub tts: Tts,
    pub screenshot_buffer: Vec<u8>,
    pub screenshot_size: (u32, u32),
    pub rect_start: Option<UPoint>,
    pub rect_end: Option<UPoint>,

    pub screenshot_image: Option<Vec<u8>>,
}

impl Application for IcedApp {
    type Renderer = Renderer;
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn view(&self) -> Element<'_, Message> {
        if let Some(screenshot_image) = &self.screenshot_image {
            clickable(
                iced::widget::image(Handle::from_rgba(
                    self.screenshot_size.0,
                    self.screenshot_size.1,
                    screenshot_image.clone(),
                ))
                .content_fit(ContentFit::None),
            )
            .on_mouse_down(Message::StartRect)
            .on_mouse_up(Message::EndRect)
            .into()
        } else {
            //let _ = iced::window::resize::<()>(Id::MAIN, Size::new(60., 32.));
            button("READ").on_press(Message::Read).into()
        }
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::Read => {
                println!("Read clicked");
                let monitor = &Monitor::all().unwrap()[0];
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
                self.speak_from_rect();
                self.rect_start = None;
                self.screenshot_image = None;
                iced::window::resize(Id::MAIN, Size::new(65., 32.))
            }
            Message::MouseMoved(pos) => {
                if let Some(rect_start) = self.rect_start {
                    if let Some(screenshot_image) = &mut self.screenshot_image {
                        screenshot_image.copy_from_slice(&self.screenshot_buffer[..]);
                        draw_rectangle(
                            &mut self.screenshot_image.as_mut().unwrap(),
                            self.screenshot_size,
                            rect_start,
                            pos,
                        );

                        self.rect_end = Some(pos);
                    }
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
        event::listen_with(|evt, _, _| {
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

fn draw_rectangle(buffer: &mut [u8], size: (u32, u32), start: UPoint, end: UPoint) {
    // Work out the indices of the edges of the rectangle in the 1D vec of image data, *4 because a pixel is RGBA
    for i in start.x..end.x {
        // for j_min
        let left_index = (((size.0 * start.y) + i) * 4) as usize;
        // for j_max
        let right_index = (((size.0 * end.y) + i) * 4) as usize;

        buffer[left_index..left_index + 4].clone_from_slice(&RECT_COLOUR);
        buffer[right_index..right_index + 4].clone_from_slice(&RECT_COLOUR);
    }

    for j in start.y..end.y {
        // for i_min
        let top_index = (((size.0 * j) + start.x) * 4) as usize;
        // for i_max
        let bottom_index = (((size.0 * j) + end.x) * 4) as usize;

        buffer[top_index..top_index + 4].clone_from_slice(&RECT_COLOUR);
        buffer[bottom_index..bottom_index + 4].clone_from_slice(&RECT_COLOUR);
    }
}

impl Default for IcedApp {
    fn default() -> Self {
        Self {
            engine: IcedApp::init_engine(),
            tts: IcedApp::init_tts(),
            screenshot_buffer: vec![],
            screenshot_size: (0, 0),
            rect_start: None,
            rect_end: None,

            screenshot_image: None,
        }
    }
}

impl IcedApp {
    fn init_engine() -> OcrEngine {
        println!("Opening detection data");
        // Use the `download-models.sh` script to download the models.

        let detection_model_data = fs::read("text-detection.rten").unwrap();
        let rec_model_data = fs::read("text-recognition.rten").unwrap();

        let detection_model = Model::load(&detection_model_data).unwrap();
        let recognition_model = Model::load(&rec_model_data).unwrap();

        println!("Initialising OCR engine");

        OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })
        .unwrap()
    }

    fn init_tts() -> Tts {
        println!("Initialising reader");
        let mut inner_tts = Tts::default().expect("Failed to start Text-to-Speech");
        if Tts::screen_reader_available() {
            println!("A screen reader is available on this platform.");
        } else {
            println!("No screen reader is available on this platform.");
        }
        let Features {
            utterance_callbacks,
            ..
        } = inner_tts.supported_features();
        if utterance_callbacks {
            inner_tts
                .on_utterance_begin(Some(Box::new(|utterance| {
                    println!("Started speaking {:?}", utterance)
                })))
                .unwrap();
            inner_tts
                .on_utterance_end(Some(Box::new(|utterance| {
                    println!("Finished speaking {:?}", utterance)
                })))
                .unwrap();
            inner_tts
                .on_utterance_stop(Some(Box::new(|utterance| {
                    println!("Stopped speaking {:?}", utterance)
                })))
                .unwrap();
        }
        let Features { is_speaking, .. } = inner_tts.supported_features();
        if is_speaking {
            println!("Are we speaking? {}", inner_tts.is_speaking().unwrap());
        }
        inner_tts.speak("Hello, world.", false).unwrap();

        inner_tts
    }

    fn speak_from_rect(&mut self) {
        let (width, height) = self.screenshot_size;
        let img_buf: ImageBuffer<image::Rgba<u8>, Vec<u8>> = imageops::flip_vertical(
            &ImageBuffer::from_raw(width, height, self.screenshot_buffer.clone()).unwrap(),
        );

        if let Some(rect_start) = self.rect_start
            && let Some(rect_end) = self.rect_end
        {
            println!("{:?} -> {:?}", rect_start, rect_end);
            if rect_start.x >= rect_end.x as u32 || rect_start.y >= rect_end.y as u32 {
                println!("Invalid bounds received, reading nothing");
                return;
            }
            let (new_width, new_height) = (
                rect_end.x as u32 - rect_start.x,
                rect_end.y as u32 - rect_start.y,
            );
            let cropped_buf =
                imageops::crop_imm(&img_buf, rect_start.x, rect_start.y, new_width, new_height);

            cropped_buf.to_image().save("cropped_buf.png").unwrap();

            let binding = cropped_buf.to_image().into_raw();
            let img_source =
                ImageSource::from_bytes(&binding[..], (new_width, new_height)).unwrap();

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

            self.rect_start = None;
        }
    }
}
