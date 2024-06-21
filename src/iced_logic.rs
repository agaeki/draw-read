use image::imageops;
use image::ImageBuffer;
use image::SubImage;
use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use rten::Model;
use std::cmp;
use std::fs;
use tts::Features;
use tts::Tts;

#[derive(Clone, Debug, Copy)]
pub struct UPoint {
    pub x: u32,
    pub y: u32,
}

pub fn init_engine() -> OcrEngine {
    println!("Opening detection data");
    // Use the `download-models.sh` script to download the models.

    let detection_model_data = fs::read("text-detection.rten").unwrap();
    let rec_model_data = fs::read("text-recognition.rten").unwrap();

    let detection_model = Model::load(&detection_model_data).unwrap();
    let recognition_model = Model::load(&rec_model_data).unwrap();

    println!("Initialising OCR engine");

    OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap()
}

pub fn init_tts() -> Tts {
    println!("Initialising reader");
    let inner_tts = Tts::default().expect("Failed to start Text-to-Speech");
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

    inner_tts
}

pub fn get_cropped_image_source<'a>(
    screenshot: Vec<u8>,
    screenshot_size: (u32, u32),
    first_corner: UPoint,
    second_corner: UPoint,
) -> (Vec<u8>, u32, u32) {
    let (width, height) = screenshot_size;

    let rect_start = get_top_left(first_corner, second_corner);
    let rect_end = get_bottom_right(first_corner, second_corner);

    let (new_width, new_height) = (
        rect_end.x as u32 - rect_start.x,
        rect_end.y as u32 - rect_start.y,
    );
    let raw_img = ImageBuffer::from_raw(width, height, screenshot).unwrap();
    let cropped_buf: SubImage<&ImageBuffer<image::Rgba<u8>, Vec<u8>>> =
        imageops::crop_imm(&raw_img, rect_start.x, rect_start.y, new_width, new_height);

    cropped_buf.to_image().save("cropped_buf.png").unwrap();

    (cropped_buf.to_image().into_raw(), new_width, new_height)
}

pub fn get_top_left(point1: UPoint, point2: UPoint) -> UPoint {
    UPoint {
        x: cmp::min(point1.x, point2.x),
        y: cmp::min(point1.y, point2.y),
    }
}

pub fn get_bottom_right(point1: UPoint, point2: UPoint) -> UPoint {
    UPoint {
        x: cmp::max(point1.x, point2.x),
        y: cmp::max(point1.y, point2.y),
    }
}

static RECT_COLOUR: [u8; 4] = [0, 255, 0, 255];

pub fn draw_rectangle(buffer: &mut [u8], size: (u32, u32), start: UPoint, end: UPoint) {
    // Work out the indices of the edges of the rectangle in the 1D vec of image data, *4 because a pixel is RGBA
    for i in start.x..end.x {
        // for j_min
        let left_index = (((size.0 * start.y) + i) * 4) as usize;
        // for j_max
        let right_index = (((size.0 * end.y) + i) * 4) as usize;

        buffer[left_index..left_index + 4].clone_from_slice(&RECT_COLOUR);
        buffer[right_index..right_index + 4].clone_from_slice(&RECT_COLOUR);
    }

    for j in start.y..end.y {
        // for i_min
        let top_index = (((size.0 * j) + start.x) * 4) as usize;
        // for i_max
        let bottom_index = (((size.0 * j) + end.x) * 4) as usize;

        buffer[top_index..top_index + 4].clone_from_slice(&RECT_COLOUR);
        buffer[bottom_index..bottom_index + 4].clone_from_slice(&RECT_COLOUR);
    }
}
