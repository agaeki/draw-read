// This file is part of draw-read
// Copyright (C) 2024 agaeki

use crate::iced_logic::ScreenPoint;
use directories::ProjectDirs;
use num_traits::cast::FromPrimitive;
use serde::*;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy, PartialOrd, PartialEq)]
pub enum VoicePitch {
    Soprano,
    Mezzo,
    #[default]
    Alto,
    Tenor,
    Baritone,
    Bass,
}

impl From<u8> for VoicePitch {
    fn from(num: u8) -> Self {
        match num {
            1 => VoicePitch::Soprano,
            2 => VoicePitch::Mezzo,
            3 => VoicePitch::Alto,
            4 => VoicePitch::Tenor,
            5 => VoicePitch::Baritone,
            _ => VoicePitch::Bass,
        }
    }
}

impl Into<f32> for VoicePitch {
    fn into(self) -> f32 {
        match self {
            VoicePitch::Soprano => 1.,
            VoicePitch::Mezzo => 2.,
            VoicePitch::Alto => 3.,
            VoicePitch::Tenor => 4.,
            VoicePitch::Baritone => 5.,
            VoicePitch::Bass => 6.,
        }
    }
}
impl Into<f64> for VoicePitch {
    fn into(self) -> f64 {
        Into::<f32>::into(self) as f64
    }
}

impl FromPrimitive for VoicePitch {
    fn from_i64(num: i64) -> std::option::Option<Self> {
        match num {
            1 => Some(VoicePitch::Soprano),
            2 => Some(VoicePitch::Mezzo),
            3 => Some(VoicePitch::Alto),
            4 => Some(VoicePitch::Tenor),
            5 => Some(VoicePitch::Baritone),
            _ => Some(VoicePitch::Bass),
        }
    }
    fn from_u64(num: u64) -> std::option::Option<Self> {
        match num {
            1 => Some(VoicePitch::Soprano),
            2 => Some(VoicePitch::Mezzo),
            3 => Some(VoicePitch::Alto),
            4 => Some(VoicePitch::Tenor),
            5 => Some(VoicePitch::Baritone),
            _ => Some(VoicePitch::Bass),
        }
    }
}

impl Display for VoicePitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&format!("{:?}", self).to_owned())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy, PartialOrd, PartialEq)]
pub enum VoiceRate {
    Slowest,
    Slow,
    #[default]
    Default,
    Fast,
    Fastest,
    TooFast,
}

impl From<u8> for VoiceRate {
    fn from(num: u8) -> Self {
        match num {
            1 => VoiceRate::Slowest,
            2 => VoiceRate::Slow,
            3 => VoiceRate::Default,
            4 => VoiceRate::Fast,
            5 => VoiceRate::Fastest,
            _ => VoiceRate::TooFast,
        }
    }
}

impl Into<f32> for VoiceRate {
    fn into(self) -> f32 {
        match self {
            VoiceRate::Slowest => 1.,
            VoiceRate::Slow => 2.,
            VoiceRate::Default => 3.,
            VoiceRate::Fast => 4.,
            VoiceRate::Fastest => 5.,
            VoiceRate::TooFast => 6.,
        }
    }
}
impl Into<f64> for VoiceRate {
    fn into(self) -> f64 {
        Into::<f32>::into(self) as f64
    }
}

impl FromPrimitive for VoiceRate {
    fn from_i64(num: i64) -> std::option::Option<Self> {
        match num {
            1 => Some(VoiceRate::Slowest),
            2 => Some(VoiceRate::Slow),
            3 => Some(VoiceRate::Default),
            4 => Some(VoiceRate::Fast),
            5 => Some(VoiceRate::Fastest),
            _ => Some(VoiceRate::TooFast),
        }
    }
    fn from_u64(num: u64) -> std::option::Option<Self> {
        match num {
            1 => Some(VoiceRate::Slowest),
            2 => Some(VoiceRate::Slow),
            3 => Some(VoiceRate::Default),
            4 => Some(VoiceRate::Fast),
            5 => Some(VoiceRate::Fastest),
            _ => Some(VoiceRate::TooFast),
        }
    }
}

impl Display for VoiceRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&format!("{:?}", self).to_owned())
    }
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

    pub position: ScreenPoint,

    pub drag_draw: bool,
}

impl Settings {
    pub fn save_to_file(&self) -> Result<(), serde_json::Error> {
        let settings_path = Settings::get_file_path();
        serde_json::to_writer_pretty(File::create(settings_path).unwrap(), self)
    }

    fn get_file_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("net", "agaeki", "draw-read") {
            let settings_path = proj_dirs.config_dir().join("settings.json");
            match settings_path.try_exists() {
                Ok(false) => {
                    let _ = std::fs::create_dir_all(proj_dirs.config_dir());
                    File::create(settings_path.clone()).unwrap();
                }
                Err(e) => {
                    eprintln!("Error finding path {:?}: {:?}", settings_path, e);
                }
                _ => (),
            }
            return settings_path;
        }
        panic!("Error finding project dir for settings file!");
    }
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
            position: ScreenPoint::default(),
            drag_draw: true,
        };

        let settings_path = Settings::get_file_path();
        let settings_string = std::fs::read_to_string(settings_path.clone()).unwrap();

        match settings_string.len() {
            0 => {
                let _ = serde_json::to_writer_pretty(
                    File::create(settings_path).unwrap(),
                    &default_settings,
                );
            }
            _ => {
                default_settings =
                    serde_json::from_reader(BufReader::new(File::open(settings_path).unwrap()))
                        .unwrap();
            }
        }

        default_settings
    }
}
