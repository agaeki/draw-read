
use eframe::egui::{self};

use std::fs;
use std::error::Error;
use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use ocrs::ImageSource;

use screenshots::Screen;
use mouse_position::mouse_position::{Mouse};
use rten::Model;
use rten_imageproc::RotatedRect;

#[derive(Clone, Debug)]
struct StrToRead {
    position: RotatedRect,
    str: String
}

#[derive(Default)]
struct App {
    to_read: String,
    read_position: (f32,f32),
    engine: Option<OcrEngine>,
    previous_strings: Vec<StrToRead>
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating UI");

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    Ok(eframe::run_native(
        "screen_reader_1",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )?)
}

fn read(data: &String) {
    println!("Reading '{:?}'", data)
}

fn get_strings(engine: &OcrEngine) -> Result<Vec<StrToRead>, Box<dyn Error>> {

    let screen = Screen::all().unwrap()[1];

    println!("Screen: {screen:?}");
    let rgb_image = screen.capture().unwrap();
    let img_source = ImageSource::from_bytes(rgb_image.as_raw(), rgb_image.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;
    
    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input)?;

    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    // Recognize the characters in each line.
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    Ok(line_rects.into_iter().flatten().zip(line_texts.into_iter().flatten())
        // Filter likely spurious detections. With future model improvements
        // this should become unnecessary.
        .filter(|l| l.1.to_string().len() > 1)
        .map(|(rect, line)| StrToRead{
            position: rect,
            str: line.to_string()
        })
        .collect::<Vec<_>>())

}

fn get_closest_rect(rects: &mut Vec<StrToRead>, position: Mouse) -> StrToRead {
    match position{
        Mouse::Position{x,y} => {rects.sort_by(|rect1, rect2| {
                let corner1 = rect1.position.corners()[0];
                let vec1: (i32, i32) = (corner1.x as i32 - x, corner1.y as i32 - y);
                let mag1 = vec1.0.pow(2) + vec1.1.pow(2);

                let corner2 = rect2.position.corners()[0];
                let vec2: (i32, i32) = (corner2.x as i32 - x, corner2.y as i32 - y);
                let mag2 = vec2.0.pow(2) + vec2.1.pow(2);

                mag1.cmp( &mag2)
            });
        rects[0].clone()
    },
        _ => panic!("Mouse position error!")
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.engine.is_none() {

        println!("Opening detection data");
        // Use the `download-models.sh` script to download the models.
        let detection_model_data = fs::read("F:/Projects/rust/ocrs/ocrs/examples/text-detection.rten").unwrap();
        let rec_model_data = fs::read("F:/Projects/rust/ocrs/ocrs/examples/text-recognition.rten").unwrap();

        let detection_model = Model::load(&detection_model_data).unwrap();
        let recognition_model = Model::load(&rec_model_data).unwrap();

    println!("Initialising OCR engine");
        self.engine = Some(OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        }).unwrap());
        }

        println!("Getting strings");
        let mut strings = get_strings(&self.engine.as_ref().unwrap()).unwrap();

        println!("Finding closest string");
        let mouse_pos = Mouse::get_mouse_position();
        let closest_str = get_closest_rect(&mut strings, mouse_pos);
        self.read_position = (closest_str.position.corners()[0].x - 50., closest_str.position.corners()[0].y);
        self.to_read = closest_str.str.clone();

        println!("Creating window at {:?}", self.read_position);
        egui::Window::new("screen_reader_1")
            .movable(false)
            .fixed_pos(self.read_position)
            .fixed_size((500.,100.))
            .title_bar(false)
            .show(ctx, |ui| {
                if ui.button("read").clicked() {
                    read(&self.to_read);
                }

                ctx.request_repaint();
            });
    }
}
