extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::app_logic::App;
use crate::nwg::NativeUi;
use tts::*;

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
