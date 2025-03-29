// mod disasm;
// mod logging;
#[macro_use]
mod macros;
mod romloader;
mod views;
mod window;

use clap::{App, Arg};
use cpal::traits::StreamTrait;
use egui::ViewportBuilder;
use log::LevelFilter;
use nes::run;
use romloader::RomLoader;
use std::sync::mpsc::channel;
use window::DebuggerWindow;

fn main() {
    let matches = App::new("NES Debugger")
        .version("0.0.1")
        .author("Sam Noedel")
        .about("Debugging environment for libnes")
        .arg(
            Arg::with_name("rom")
                .required(true)
                .long_help("Path to a .nes ROM file"),
        )
        .arg(
            Arg::with_name("record")
                .long("record")
                .required(false)
                .takes_value(false)
                .long_help("Begin recording at startup"),
        )
        .arg(
            Arg::with_name("arecord")
                .long("arecord")
                .required(false)
                .takes_value(false)
                .long_help("Begin recording audio at startup"),
        )
        .get_matches();

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let path = matches.value_of("rom").unwrap();
    let record = matches.is_present("record");
    let arecord = matches.is_present("arecord");

    let rom = RomLoader::load(path).unwrap();
    let (send_control, receive_control) = channel();
    let (send_frame, receive_frame) = channel();

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_min_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    let stream = run(rom, send_frame, receive_control);
    let _ = stream.play();

    eframe::run_native(
        "NES Debugger",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(DebuggerWindow::new(
                nes::EmulationState::Pause,
                send_control,
                receive_frame,
                record,
                arecord,
            )))
        }),
    )
    .unwrap();
}
