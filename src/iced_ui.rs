use iced::advanced::Application;
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
use iced::Theme;
use mouse_position::mouse_position::Mouse;
use ocrs::OcrEngine;
use std::cell::RefCell;
use tts::Tts;
use xcap::Monitor;

static RECT_COLOUR: [u8; 4] = [0, 255, 0, 255];

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Read,
    StartRect,
    EndRect,
    Tick,
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
    type Renderer = Renderer;
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn view(&self) -> Element<'_, Message> {
        if let Some(screenshot_image) = &self.screenshot_image {
            let mut buffer = screenshot_image.clone();
            if let Some(rect_start) = self.rect_start {
                if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
                    draw_rectangle(
                        &mut buffer,
                        self.screenshot_size,
                        rect_start,
                        (x as u32, y as u32),
                    );
                }
            }

            clickable(
                iced::widget::image(Handle::from_rgba(
                    self.screenshot_size.0,
                    self.screenshot_size.1,
                    screenshot_image.clone(),
                ))
                .content_fit(ContentFit::None),
            )
            .on_mouse_down(Message::StartRect)
            .on_mouse_down(Message::EndRect)
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
                    self.rect_start = Some((x as u32, y as u32));
                }
                Command::none()
            }
            Message::EndRect => Command::none(),
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

fn draw_rectangle(buffer: &mut [u8], size: (u32, u32), start: (u32, u32), end: (u32, u32)) {
    // Work out the indices of the edges of the rectangle in the 1D vec of image data, *4 because a pixel is RGBA
    for i in start.0..end.0 {
        // for j_min
        let left_index = (((size.0 * start.1) + i) * 4) as usize;
        // for j_max
        let right_index = (((size.0 * end.1) + i) * 4) as usize;

        buffer[left_index..left_index + 4].clone_from_slice(&RECT_COLOUR);
        buffer[right_index..right_index + 4].clone_from_slice(&RECT_COLOUR);
    }

    for j in end.1..start.1 {
        // for i_min
        let top_index = (((size.0 * j) + start.0) * 4) as usize;
        // for i_max
        let bottom_index = (((size.0 * j) + end.0) * 4) as usize;

        buffer[top_index..top_index + 4].clone_from_slice(&RECT_COLOUR);
        buffer[bottom_index..bottom_index + 4].clone_from_slice(&RECT_COLOUR);
    }
}
