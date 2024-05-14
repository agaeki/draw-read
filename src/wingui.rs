use crate::NativeGui;
use ocrs::OcrEngine;
use rten_imageproc::RotatedRect;

use nwd::NwgUi;
#[derive(Default, NwgUi)]
pub struct WinGui {
    #[nwg_control(size: (300, 115), position: (0, 0), title: "", flags: "	VISIBLE")]
    #[nwg_events( OnMouseMove: [Self::mouse_moved] )]
    window: nwg::Window,

    #[nwg_control(text: "Read", size: (60,20), position: (0, 0))]
    #[nwg_events( OnButtonClick: [Self::button_clicked] )]
    hello_button: nwg::Button,

    data: App,
}

impl WinGui {
    fn button_clicked(&self) {}

    fn mouse_moved(&self) {}
}

impl NativeGui for WinGui {
    fn init() -> Box<(dyn NativeGui + 'static)> {
        todo!()
    }
    fn set_position(&self, _: f32, _: f32) {
        todo!()
    }
}

#[derive(Default)]
pub struct App {
    pub to_read: String,
    pub read_position: (f32, f32),
    pub engine: Option<OcrEngine>,
    pub previous_strings: Vec<StrToRead>,
}

#[derive(Clone, Debug)]
pub struct StrToRead {
    pub position: RotatedRect,
    pub str: String,
}
