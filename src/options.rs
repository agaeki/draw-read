// This file is part of draw-read
// Copyright (C) 2024 agaeki

use crate::iced_logic::UPoint;
use directories::ProjectDirs;
use serde::*;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum VoicePitch {
    Soprano,
    Mezzo,
    #[default]
    Alto,
    Tenor,
    Baritone,
    Bass,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum VoiceRate {
    Slowest,
    Slow,
    #[default]
    Default,
    Fast,
    Fastest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub detection_file: PathBuf,
    pub recognition_file: PathBuf,

    pub rect_colour: [u8; 4],

    pub volume: u8,
    pub pitch: VoicePitch,
    pub rate: VoiceRate,
    pub voice: String,

    pub position: UPoint,

    pub drag_draw: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let mut default_settings = Self {
            detection_file: "text-detection.rten".into(),
            recognition_file: "text-recognition.rten".into(),
            rect_colour: [0, 255, 0, 255],
            volume: 255,
            pitch: VoicePitch::default(),
            rate: VoiceRate::default(),
            voice: String::default(),
            position: UPoint::default(),
            drag_draw: true,
        };

        if let Some(proj_dirs) = ProjectDirs::from("net", "agaeki", "draw-read") {
            let settings_path = proj_dirs.config_dir().join("settings.json");
            match settings_path.try_exists() {
                Ok(true) => {
                    // If the file is found, read the settings from it
                    println!("Found settings in {:?}", settings_path);
                    default_settings =
                        serde_json::from_reader(BufReader::new(File::open(settings_path).unwrap()))
                            .unwrap();
                }
                Ok(false) => {
                    // If not, create the file with the default settings
                    println!("Creating settings file at {:?}", settings_path);
                    let _ = std::fs::create_dir_all(proj_dirs.config_dir());
                    serde_json::to_writer_pretty(
                        File::create(settings_path).unwrap(),
                        &default_settings,
                    )
                    .unwrap();
                }
                Err(e) => {
                    eprintln!("Error finding path {:?}: {:?}", settings_path, e);
                }
            };
        }
        default_settings
    }
}
