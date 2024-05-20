extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::app_logic::App;
use crate::app_logic::StrToRead;
use crate::nwg::NativeUi;
use tts::*;

use ocrs::ImageSource;
use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use std::error::Error as StdError;
use std::fs;

use mouse_position::mouse_position::Mouse;
use rten::Model;

mod app_logic;
mod basic_ui;
mod timer;

fn main() -> Result<(), Error> {
    println!("Creating UI");
    nwg::init().expect("Failed to init Native Windows GUI");

    let _built_gui = App::build_ui(App::default()).unwrap();

    nwg::dispatch_thread_events();

    Ok(())
}

fn read(data: &String, tts: &mut Tts) {
    println!("Speaking '{:?}'", data);

    tts.speak(data, false).unwrap();
}

fn get_strings(engine: &OcrEngine) -> Result<Vec<StrToRead>, Box<dyn StdError>> {
    /*let screen = Screen::all().unwrap()[0];

    //println!("Screen: {screen:?}");
    let rgb_image = screen.capture().unwrap();

    let img_source = ImageSource::from_bytes(rgb_image.as_raw(), rgb_image.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;

    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input)?;

    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    // Recognise the words in each line and match the string to the line position
    Ok(line_rects
        .into_iter()
        .map(|line| StrToRead {
            position: line[0],
            str: engine.recognize_text(&ocr_input, &[line]).unwrap()[0]
                .clone()
                .map_or("".to_string(), |x| x.to_string()),
        })
        .filter(|str_to_read| str_to_read.str.len() > 5)
        .collect())*/
    Ok(vec![])
}

fn get_closest_rect(rects: &mut Vec<StrToRead>, position: Mouse) -> StrToRead {
    match position {
        Mouse::Position { x, y } => {
            println!("Mouse pos is {:?},{:?}", x, y);
            //println!("Rects before {:?}", rects);
            rects
                .into_iter()
                .min_by_key(|rect| {
                    let center = rect.position.center();
                    let vec: (i64, i64) = (
                        center.x.floor() as i64 - x as i64,
                        center.y.floor() as i64 - y as i64,
                    );
                    vec.0.pow(2) + vec.1.pow(2)
                })
                .unwrap()
                .clone()
        }
        _ => panic!("Mouse position error!"),
    }
}

fn update_and_get_new_position(
    engine: &mut Option<OcrEngine>,
    tts: &mut Option<Tts>,
    read_position: &mut (f32, f32),
    to_read: &mut String,
) -> (f32, f32) {
    if engine.is_none() {
        println!("Opening detection data");
        // Use the `download-models.sh` script to download the models.
        let detection_model_data =
            fs::read("F:/Projects/rust/ocrs/ocrs/examples/text-detection.rten").unwrap();
        let rec_model_data =
            fs::read("F:/Projects/rust/ocrs/ocrs/examples/text-recognition.rten").unwrap();

        let detection_model = Model::load(&detection_model_data).unwrap();
        let recognition_model = Model::load(&rec_model_data).unwrap();

        println!("Initialising OCR engine");
        *engine = Some(
            OcrEngine::new(OcrEngineParams {
                detection_model: Some(detection_model),
                recognition_model: Some(recognition_model),
                ..Default::default()
            })
            .unwrap(),
        );
    }

    if tts.is_none() {
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
        *tts = Some(inner_tts);
    }

    //println!("Getting strings");
    let mut strings = get_strings(engine.as_ref().unwrap()).unwrap();

    //println!("Finding closest string");
    let mouse_pos = Mouse::get_mouse_position();
    let closest_str = get_closest_rect(&mut strings, mouse_pos);
    println!(
        "Closest string is '{:?}' at '{:?}'",
        closest_str.str,
        closest_str.position.center()
    );

    let x_pos = if (closest_str.position.center().x - 60.) < 0.0 {
        closest_str.position.center().x
    } else {
        closest_str.position.center().x - 60.0
    };

    let y_pos = if (closest_str.position.center().x - 60.) < 0.0 {
        closest_str.position.center().y + 25.0
    } else {
        closest_str.position.center().y
    };

    *read_position = (x_pos, y_pos);
    *to_read = closest_str.str.clone();

    println!("Creating window at {:?}", read_position);
    (read_position.0, read_position.1)
}
