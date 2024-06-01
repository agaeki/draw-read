use druid::im::Vector;
use druid::widget::{Button, Flex};
use druid::Data;
use druid::Lens;
use druid::Widget;
use ocrs::OcrEngine;
use std::rc::Rc;
use tts::Tts;

#[derive(Clone, Data, Lens)]
struct AppState {
    pub screenshot_buffer: Vector<u8>,
    pub screenshot_image: Vector<u8>,

    pub rect_start: Option<(u32, u32)>,

    #[data(ignore)]
    tts: Tts,
    #[data(ignore)]
    engine: Rc<OcrEngine>,
}

fn build_ui() -> impl Widget<AppState> {
    let read_button = Button::new("Read").on_click(|_ctx, _data, _env| println!("Read"));
    let options_button = Button::new("Options").on_click(|_ctx, _data, _env| println!("Options"));

    Flex::column()
        .with_child(read_button)
        .with_child(options_button)
}
