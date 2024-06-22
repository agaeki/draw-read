// This file is part of draw-read
// Copyright (C) 2024 agaeki

use crate::iced_logic::UPoint;
use serde::*;
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
