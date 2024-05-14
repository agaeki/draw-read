extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::nwg::NativeUi;
use crate::nwg::Timer;
use rten_imageproc::RotatedRect;
use std::rc::Rc;

use ocrs::ImageSource;
use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use std::error::Error;
use std::fs;

use mouse_position::mouse_position::Mouse;
use rten::Model;
use screenshots::Screen;

use std::cell::RefCell;
use std::ops::Deref;

mod timer;

pub struct BasicAppUi {
    inner: Rc<App>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<BasicAppUi> for App {
    fn build_ui(mut data: App) -> Result<BasicAppUi, nwg::NwgError> {
        use nwg::Event as E;

        // Controls
        nwg::Window::builder()
            .flags(nwg::WindowFlags::VISIBLE)
            .size((60, 25))
            .position((300, 300))
            .title("")
            .build(&mut data.window)?;

        nwg::Button::builder()
            .text("Read")
            .parent(&data.window)
            .build(&mut data.read_button)?;

        #[allow(deprecated)]
        let _ = nwg::Timer::builder()
            .parent(&data.window)
            .interval(1000)
            .stopped(false)
            .build(&mut data.timer);

        // Wrap-up
        let ui = BasicAppUi {
            inner: Rc::new(data),
            default_handler: Default::default(),
        };

        // Events
        let evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(ui) = evt_ui.upgrade() {
                match evt {
                    E::OnButtonClick => {
                        if &handle == &ui.read_button {
                            read(&ui.to_read.borrow());
                        }
                    }
                    E::OnWindowClose => {
                        if &handle == &ui.window {
                            nwg::stop_thread_dispatch();
                        }
                    }
                    E::OnTimerTick => {
                        //println!("Timer ticked for {:?} {:?}", handle, ui.window.handle);
                        let new_position = update_and_get_new_position(
                            &mut ui.engine.borrow_mut(),
                            &mut ui.read_position.borrow_mut(),
                            &mut ui.to_read.borrow_mut(),
                        );

                        ui.window
                            .set_position(new_position.0 as i32, new_position.1 as i32);
                    }
                    _ => {}
                }
            }
        };

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.window.handle,
            handle_events,
        ));

        return Ok(ui);
    }
}

impl Drop for BasicAppUi {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for BasicAppUi {
    type Target = App;

    fn deref(&self) -> &App {
        &self.inner
    }
}

#[derive(Default)]
pub struct App {
    pub to_read: RefCell<String>,
    pub read_position: RefCell<(f32, f32)>,
    pub engine: RefCell<Option<OcrEngine>>,
    pub previous_strings: Vec<StrToRead>,

    window: nwg::Window,
    read_button: nwg::Button,
    #[allow(deprecated)]
    timer: nwg::Timer,
}

#[derive(Clone, Debug)]
pub struct StrToRead {
    pub position: RotatedRect,
    pub str: String,
}

fn main() {
    println!("Creating UI");
    nwg::init().expect("Failed to init Native Windows GUI");

    let _built_gui = App::build_ui(App::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}

fn read(data: &String) {
    println!("Reading '{:?}'", data)
}

fn get_strings(engine: &OcrEngine) -> Result<Vec<StrToRead>, Box<dyn Error>> {
    let screen = Screen::all().unwrap()[0];

    //println!("Screen: {screen:?}");
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

    Ok(line_rects
        .into_iter()
        .flatten()
        .zip(line_texts.into_iter().flatten())
        // Filter likely spurious detections. With future model improvements
        // this should become unnecessary.
        .filter(|l| l.1.to_string().len() > 5)
        .map(|(rect, line)| StrToRead {
            position: rect,
            str: line.to_string(),
        })
        .collect::<Vec<_>>())
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

    //println!("Getting strings");
    let mut strings = get_strings(engine.as_ref().unwrap()).unwrap();
    for string in strings.iter() {
        println!(
            "String is {:?} at {:?} {:?} {:?} {:?}",
            string.str,
            string.position.center(),
            string.position.up_axis(),
            string.position.width(),
            string.position.height()
        );
    }

    //println!("Finding closest string");
    let mouse_pos = Mouse::get_mouse_position();
    let closest_str = get_closest_rect(&mut strings, mouse_pos);
    println!(
        "Closest string is '{:?}' at '{:?}'",
        closest_str.str,
        closest_str.position.center()
    );
    *read_position = (
        closest_str.position.center().x - 50.,
        closest_str.position.center().y,
    );
    *to_read = closest_str.str.clone();

    //println!("Creating window at {:?}", read_position);
    (read_position.0, read_position.1)
}

trait NativeGui {
    fn init() -> Box<dyn NativeGui>
    where
        Self: Sized;

    fn set_position(&self, x: f32, y: f32);
}
