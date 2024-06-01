use iced::executor;
use iced::widget::button;
use iced::widget::image::Handle;
use iced::window::Id;
use iced::Application;
use iced::Command;
use iced::ContentFit;
use iced::Element;
use iced::Point;
use iced::Sandbox;
use iced::Size;
use iced::Theme;
use ocrs::OcrEngine;
use std::cell::RefCell;
use tts::Tts;
use xcap::Monitor;

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Read,
    StartRect,
    EndRect,
}

#[derive(Default)]
pub struct IcedApp {
    pub engine: RefCell<Option<OcrEngine>>,
    pub tts: RefCell<Option<Tts>>,
    pub screenshot_buffer: Vec<u8>,
    pub screenshot_size: (u32, u32),
    pub rect_start: Option<(u32, u32)>,

    pub screenshot_image: Option<Vec<u8>>,
}

impl Application for IcedApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn view(&self) -> Element<'_, Message> {
        if let Some(screenshot_image) = &self.screenshot_image {
            iced::widget::image(Handle::from_pixels(
                self.screenshot_size.0,
                self.screenshot_size.1,
                screenshot_image.clone(),
            ))
            .content_fit(ContentFit::None)
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
                println!("Size is {:?} {:?}", monitor.width(), monitor.height());
                let rgb_image = monitor.capture_image().unwrap();

                println!("Size is {:?} {:?}", rgb_image.width(), rgb_image.height());
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
            _ => panic!("{:?} Not implemented", message),
        }
    }
    fn new(flags: Self::Flags) -> (Self, iced::Command<Message>) {
        (
            Self::default(),
            iced::window::resize(Id::MAIN, Size::new(65., 32.)),
        )
    }
    fn title(&self) -> std::string::String {
        "".to_string()
    }
}
