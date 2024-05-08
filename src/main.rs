
use xilem::view::button;
use xilem::Xilem;
use xilem::MasonryView;
use std::fs;
use std::error::Error;use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use ocrs::ImageSource;

use screenshots::Screen;

use rten::Model;

struct AppData {
    to_read: String,
    engine: OcrEngine
}

struct StrToRead {
    position: u8,
    str: String
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating UI");

    println!("Opening detection data");
    // Use the `download-models.sh` script to download the models.
    let detection_model_data = fs::read("F:/Projects/rust/ocrs/ocrs/examples/text-detection.rten")?;
    let rec_model_data = fs::read("F:/Projects/rust/ocrs/ocrs/examples/text-recognition.rten")?;

    let detection_model = Model::load(&detection_model_data)?;
    let recognition_model = Model::load(&rec_model_data)?;

println!("Initialising OCR engine");
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    let data = AppData {
        to_read: "Hello I'm reading the screen".to_string(),
        engine: engine
    };

    let app = Xilem::new(data, app_logic);
    app.run_windowed("First Example".into()).unwrap();

    Ok(())
}

fn read(data: String) {
}

fn app_logic(data: &mut AppData) -> impl MasonryView<AppData> {
    button("READ", |data: &mut AppData| read(data.to_read.clone()))
}

fn get_strings(engine: &OcrEngine) -> Result<Vec<StrToRead>, Box<dyn Error>> {

    let screens = Screen::all().unwrap();

    for screen in screens {
        println!("Screen: {screen:?}");
        let mut rgb_image = screen.capture().unwrap();
        let img_source = ImageSource::from_bytes(rgb_image.as_raw(), rgb_image.dimensions())?;
        let ocr_input = engine.prepare_input(img_source)?;
        
    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input)?;

    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    // Recognize the characters in each line.
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    for line in line_texts
        .iter()
        .flatten()
        // Filter likely spurious detections. With future model improvements
        // this should become unnecessary.
        .filter(|l| l.to_string().len() > 1)
    {
        println!("{}", line);
    }
}
    Ok(vec!())

}