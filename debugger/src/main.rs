// mod disasm;
// mod logging;
#[macro_use]
mod macros;
mod romloader;
mod views;
mod window;

use clap::{Arg, App};
use log::LevelFilter;
use nes::{Emulation, FrameBuffer};
use romloader::RomLoader;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use window::DebuggerWindow;

fn main() {
    let matches = App::new("NES Debugger")
        .version("0.0.1")
        .author("Sam Noedel")
        .about("Debugging environment for libnes")
        .arg(Arg::with_name("rom")
             .required(true)
             .long_help("Path to a .nes ROM file")
        )
        .arg(Arg::with_name("record")
             .long("record")
             .required(false)
             .takes_value(false)
             .long_help("Begin recording at startup")
        )
        .arg(Arg::with_name("arecord")
             .long("arecord")
             .required(false)
             .takes_value(false)
             .long_help("Begin recording audio at startup")
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

    let frame_buffer = FrameBuffer::new();
    let frame_buffer = Arc::new(Mutex::new(frame_buffer));
    let (send_control, receive_control) = channel();
    let (send_frame, receive_frame) = channel();
    let emulation = Emulation::new(frame_buffer.clone(), receive_control, send_frame);

    thread::spawn(move || {
        DebuggerWindow::new(send_control, receive_frame, frame_buffer, record, arecord).run()
    });

    emulation.run(rom);
}
