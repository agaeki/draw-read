use image::codecs::bmp::BmpEncoder;
use image::imageops;
use image::ExtendedColorType;
use image::ImageBuffer;
use ocrs::ImageSource;
use ocrs::OcrEngineParams;
use rten::Model;
use std::cell::RefCell;
use std::fs;
use tts::Features;
use xcap::image;
use xcap::Monitor;

use ocrs::OcrEngine;

use tts::Tts;

static RECT_COLOUR: [u8; 4] = [0, 255, 0, 255];

#[derive(Default)]
pub struct App {
    pub engine: RefCell<Option<OcrEngine>>,
    pub tts: RefCell<Option<Tts>>,
    pub screenshot_buffer: Vec<u8>,

    pub window: nwg::Window,
    pub read_button: nwg::Button,
    #[allow(deprecated)]
    pub timer: nwg::Timer,
    pub screenshot_window: nwg::Window,
    pub screenshot_frame: nwg::ImageFrame,
    pub screenshot_image: nwg::Bitmap,
    pub rect_start: Option<(u32, u32)>,
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

            let img_rect_start = (rect_start.0, height - rect_start.1);
            let img_rect_end = (mouse_pos.0 as u32, height - mouse_pos.1 as u32);

            nwg::Bitmap::builder()
                .source_bin(Some(&new_buffer))
                .build(&mut self.screenshot_image)
                .unwrap();
            self.screenshot_frame
                .set_bitmap(Some(&self.screenshot_image));
        }
    }

    pub fn start_draw_rect(&mut self, position: (u32, u32)) {
        self.rect_start = Some(position);
    }

    pub fn end_draw_rect(&mut self) {
        let (width, height) = self.screenshot_frame.size();
        let img_buf: ImageBuffer<image::Rgba<u8>, Vec<u8>> = imageops::flip_vertical(
            &ImageBuffer::from_raw(width, height, self.screenshot_buffer.clone()).unwrap(),
        );

        let rect_start = self.rect_start.unwrap();
        let rect_end = nwg::GlobalCursor::local_position(self.screenshot_frame.handle, None);

        println!("{:?} -> {:?}", rect_start, rect_end);
        if rect_start.0 >= rect_end.0 as u32 || rect_start.1 >= rect_end.1 as u32 {
            println!("Invalid bounds received, reading nothing");
            return;
        }
        let (new_width, new_height) = (
            rect_end.0 as u32 - rect_start.0,
            rect_end.1 as u32 - rect_start.1,
        );
        let cropped_buf =
            imageops::crop_imm(&img_buf, rect_start.0, rect_start.1, new_width, new_height);

        cropped_buf.to_image().save("cropped_buf.png").unwrap();

        let binding = cropped_buf.to_image().into_raw();
        let img_source = ImageSource::from_bytes(&binding[..], (new_width, new_height)).unwrap();

        let engine_binding = self.engine.borrow();
        let engine = engine_binding.as_ref().unwrap();
        let ocr_input = engine.prepare_input(img_source).unwrap();

        // Get oriented bounding boxes of text words in input image.
        let word_rects = engine.detect_words(&ocr_input).unwrap();

        // Group words into lines. Each line is represented by a list of word
        // bounding boxes.
        let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

        let words = engine
            .recognize_text(&ocr_input, &line_rects[..])
            .unwrap()
            .into_iter()
            .map(|x| x.map_or("".to_string(), |x| x.to_string()))
            .collect::<Vec<_>>()
            .join(" ");

        println!("Speaking {words}");
        let mut tts_binding = self.tts.borrow_mut();
        tts_binding.as_mut().unwrap().speak(words, false).unwrap();

        self.rect_start = None;
    }

    pub fn init(&mut self) {
        println!("Opening detection data");
        // Use the `download-models.sh` script to download the models.
        let detection_model_data = fs::read("text-detection.rten").unwrap();
        let rec_model_data = fs::read("text-recognition.rten").unwrap();

        let detection_model = Model::load(&detection_model_data).unwrap();
        let recognition_model = Model::load(&rec_model_data).unwrap();

        println!("Initialising OCR engine");
        self.engine.replace(Some(
            OcrEngine::new(OcrEngineParams {
                detection_model: Some(detection_model),
                recognition_model: Some(recognition_model),
                ..Default::default()
            })
            .unwrap(),
        ));

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
        self.tts.replace(Some(inner_tts));
    }
}
