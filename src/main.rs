#![feature(let_chains)]
use iced::Application;
use iced::Error;
use iced::Size;

use crate::iced_ui::IcedApp;
use iced::Settings;
mod iced_logic;
mod iced_ui;
mod options;

fn main() -> Result<(), Error> {
    println!("Creating UI");

    let mut settings: iced::Settings<()> = Settings::default();
    settings.window.size = Size::new(65., 32.);
    settings.window.resizable = false;
    settings.window.decorations = false;

    IcedApp::run(settings)?;

    Ok(())
}
