use iced::widget::image::Handle;
use iced::widget::{button, column};
use iced::window::Id;
use iced::ContentFit;
use iced::Element;
use iced::Length;
use iced::Sandbox;
use iced::Size;
use image::codecs::bmp::BmpEncoder;
use image::ExtendedColorType;
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

impl Sandbox for IcedApp {
    fn view(&self) -> Element<'_, Message> {
        if let Some(screenshot_image) = &self.screenshot_image {
            column![iced::widget::image(Handle::from_pixels(
                self.screenshot_size.0,
                self.screenshot_size.1,
                screenshot_image.clone(),
            ))
            .content_fit(ContentFit::Contain)
            .width(Length::Fill)
            .height(Length::Fill)]
            .into()
        } else {
            let _ = iced::window::resize::<()>(Id::MAIN, Size::new(60., 32.));
            column![button("READ").on_press(Message::Read)].into()
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Read => {
                println!("Read clicked");
                let monitor = &Monitor::all().unwrap()[0];

                let rgb_image = monitor.capture_image().unwrap();

                // Store original screenshot so that we can draw a resizing rectangle on a clone without losing pixels
                self.screenshot_buffer = Vec::<u8>::with_capacity(rgb_image.len());
                let mut encoder = BmpEncoder::new(&mut self.screenshot_buffer);
                let _ = encoder.encode(
                    &rgb_image.clone().into_raw()[..],
                    rgb_image.width(),
                    rgb_image.height(),
                    ExtendedColorType::Rgba8,
                );

                self.screenshot_image = Some(self.screenshot_buffer.clone());
                let _ = iced::window::resize::<()>(
                    Id::MAIN,
                    Size::new(rgb_image.width() as f32, rgb_image.height() as f32),
                );
            }
            _ => panic!("{:?} Not implemented", message),
        }
    }
    type Message = Message;
    fn new() -> Self {
        Self::default()
    }
    fn title(&self) -> std::string::String {
        "".to_string()
    }
}
