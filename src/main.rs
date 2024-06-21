#![feature(let_chains)]
use iced::advanced::Application;
use iced::Error;
use iced::Size;
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::iced_ui::IcedApp;
use iced::Settings;
mod app_logic;
mod basic_ui;
mod iced_logic;
mod iced_ui;
mod timer;

fn main() -> Result<(), Error> {
    println!("Creating UI");

    let mut settings: iced::Settings<()> = Settings::default();
    settings.window.size = Size::new(65., 32.);
    settings.window.resizable = false;
    settings.window.decorations = false;

    IcedApp::run(settings)?;

    Ok(())
}
