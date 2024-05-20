use image::codecs::bmp::BmpEncoder;
use image::ExtendedColorType;
use std::cell::RefCell;
use xcap::image;
use xcap::Monitor;

use ocrs::OcrEngine;

use tts::Tts;

use rten_imageproc::RotatedRect;

static RECT_COLOUR: [u8; 4] = [0, 255, 0, 255];

#[derive(Default)]
pub struct App {
    pub to_read: RefCell<String>,
    pub read_position: RefCell<(f32, f32)>,
    pub engine: RefCell<Option<OcrEngine>>,
    pub tts: RefCell<Option<Tts>>,
    pub screenshot: Option<image::RgbaImage>,
    pub screenshot_buffer: Vec<u8>,

    pub window: nwg::Window,
    pub read_button: nwg::Button,
    #[allow(deprecated)]
    pub timer: nwg::Timer,
    pub screenshot_window: nwg::Window,
    pub screenshot_frame: nwg::ImageFrame,
    pub screenshot_image: nwg::Bitmap,
    pub rect_start: Option<(i32, i32)>,
}

#[derive(Clone, Debug)]
pub struct StrToRead {
    pub position: RotatedRect,
    pub str: String,
}

impl App {
    pub fn refresh_screenshot(&mut self) {
        let monitor = &Monitor::all().unwrap()[0];

        //println!("Screen: {screen:?}");
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

        nwg::Bitmap::builder()
            .source_bin(Some(&self.screenshot_buffer[..]))
            .build(&mut self.screenshot_image)
            .unwrap();

        self.screenshot_frame
            .set_bitmap(Some(&self.screenshot_image));

        self.screenshot_frame
            .set_size(rgb_image.width(), rgb_image.height());
        self.screenshot_window.maximize();
    }

    pub fn draw_rect(&mut self) {
        if let Some(rect_start) = self.rect_start {
            let mouse_pos = nwg::GlobalCursor::local_position(self.screenshot_frame.handle, None);

            let mut new_buffer = self.screenshot_buffer.clone();
            let (width, height) = self.screenshot_frame.size();

            let img_rect_start = (rect_start.0 as u32, height - rect_start.1 as u32);
            let img_rect_end = (mouse_pos.0 as u32, height - mouse_pos.1 as u32);

            // Work out the indices of the edges of the rectangle in the 1D vec of image data, *4 because a pixel is RGBA
            for i in img_rect_start.0..img_rect_end.0 {
                // for j_min
                let left_index = (((width * img_rect_start.1) + i as u32) * 4) as usize;
                // for j_max
                let right_index = (((width * img_rect_end.1) + i as u32) * 4) as usize;

                new_buffer[left_index..left_index + 4].clone_from_slice(&RECT_COLOUR);
                new_buffer[right_index..right_index + 4].clone_from_slice(&RECT_COLOUR);
            }

            for j in img_rect_end.1..img_rect_start.1 {
                // for i_min
                let top_index = (((width * j as u32) + img_rect_start.0) * 4) as usize;
                // for i_max
                let bottom_index = (((width * j as u32) + img_rect_end.0) * 4) as usize;

                new_buffer[top_index..top_index + 4].clone_from_slice(&RECT_COLOUR);
                new_buffer[bottom_index..bottom_index + 4].clone_from_slice(&RECT_COLOUR);
            }

            nwg::Bitmap::builder()
                .source_bin(Some(&new_buffer))
                .build(&mut self.screenshot_image)
                .unwrap();
            self.screenshot_frame
                .set_bitmap(Some(&self.screenshot_image));
        }
    }

    pub fn start_draw_rect(&mut self, position: (i32, i32)) {
        self.rect_start = Some(position);
    }

    pub fn end_draw_rect(&mut self) {
        self.rect_start = None;
    }
}
