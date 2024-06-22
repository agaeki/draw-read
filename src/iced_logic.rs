// This file is part of draw-read
// Copyright (C) 2024 agaeki

use crate::options;
use crate::options::VoicePitch;
use crate::options::VoiceRate;
use image::imageops;
use image::ImageBuffer;
use image::SubImage;
use ocrs::OcrEngine;
use ocrs::OcrEngineParams;
use rten::Model;
use serde::*;
use std::cmp;
use std::fs;
use tts::Features;
use tts::Tts;

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct UPoint {
    pub x: u32,
    pub y: u32,
}

impl Into<iced::Point> for UPoint {
    fn into(self) -> iced::Point {
        iced::Point {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

pub fn init_engine(settings: &options::Settings) -> OcrEngine {
    println!("Opening detection data");
    // Use the `download-models.sh` script to download the models.

    let detection_model_data = fs::read(&settings.detection_file).unwrap();
    let rec_model_data = fs::read(&settings.recognition_file).unwrap();

    let detection_model = Model::load(detection_model_data).unwrap();
    let recognition_model = Model::load(rec_model_data).unwrap();

    println!("Initialising OCR engine");

    OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap()
}

pub fn init_tts(settings: &options::Settings) -> Tts {
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

    let pitch_coefficient = (inner_tts.max_pitch() - inner_tts.min_pitch()) / 5.;
    let pitch_multiplier = match &settings.pitch {
        VoicePitch::Soprano => 1.,
        VoicePitch::Mezzo => 2.,
        VoicePitch::Alto => 3.,
        VoicePitch::Tenor => 4.,
        VoicePitch::Baritone => 5.,
        VoicePitch::Bass => 6.,
    };
    println!(
        "Setting pitch to {:?}",
        inner_tts.min_pitch() + pitch_multiplier * pitch_coefficient
    );
    println!(
        "Pitch goes from {:?} through {:?} to {:?}",
        inner_tts.min_pitch(),
        inner_tts.normal_pitch(),
        inner_tts.max_pitch()
    );
    inner_tts
        .set_pitch(inner_tts.min_pitch() + pitch_multiplier * pitch_coefficient)
        .unwrap();

    let rate_coefficient = (inner_tts.max_rate() - inner_tts.min_rate()) / 5.;
    let rate_multiplier = match &settings.rate {
        // Windows rate goes non-linearly from 0.5 to 6.0 with 1 as the default. Delineations are approximate.
        VoiceRate::Slowest => 0.4545,
        VoiceRate::Slow => 0.6,
        VoiceRate::Default => 0.909,
        VoiceRate::Fast => 3.18,
        VoiceRate::Fastest => 5.45,
    };
    println!("Setting rate to {:?}", rate_multiplier * rate_coefficient);
    println!(
        "Rate goes from {:?} through {:?} to {:?}",
        inner_tts.min_rate(),
        inner_tts.normal_rate(),
        inner_tts.max_rate()
    );
    inner_tts
        .set_rate(rate_multiplier * rate_coefficient)
        .unwrap();

    let volume_coefficient = (inner_tts.max_volume() - inner_tts.min_volume()) / 255.;
    println!(
        "Setting volume to {:?}",
        settings.volume as f32 * volume_coefficient
    );
    println!(
        "Volume goes from {:?} through {:?} to {:?}",
        inner_tts.min_volume(),
        inner_tts.normal_volume(),
        inner_tts.max_volume()
    );
    inner_tts
        .set_volume(settings.volume as f32 * volume_coefficient)
        .unwrap();

    let chosen_voice = inner_tts
        .voices()
        .unwrap()
        .into_iter()
        .find(|v| v.id() == settings.voice)
        .unwrap_or(inner_tts.voices().unwrap()[0].clone());
    println!("Setting voice to {:?}", &chosen_voice);
    inner_tts.set_voice(&chosen_voice).unwrap();

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

pub fn draw_rectangle(
    buffer: &mut [u8],
    size: (u32, u32),
    start: UPoint,
    end: UPoint,
    colour: &[u8; 4],
) {
    // Work out the indices of the edges of the rectangle in the 1D vec of image data, *4 because a pixel is RGBA
    for i in start.x..end.x {
        // for j_min
        let left_index = (((size.0 * start.y) + i) * 4) as usize;
        // for j_max
        let right_index = (((size.0 * end.y) + i) * 4) as usize;

        buffer[left_index..left_index + 4].clone_from_slice(colour);
        buffer[right_index..right_index + 4].clone_from_slice(colour);
    }

    for j in start.y..end.y {
        // for i_min
        let top_index = (((size.0 * j) + start.x) * 4) as usize;
        // for i_max
        let bottom_index = (((size.0 * j) + end.x) * 4) as usize;

        buffer[top_index..top_index + 4].clone_from_slice(colour);
        buffer[bottom_index..bottom_index + 4].clone_from_slice(colour);
    }
}
