/// Main executable

#[macro_use]
extern crate log;

use quartz::gui::run_gui;
use quartz::tsdb::TsDb;
use std::thread;

use quartz::tsdb::server::run_server;

fn main() {
    simple_logger::init().unwrap();
    info!("BOOTING QUARTZ TOOL");

    // Create datastore?
    let db = TsDb::new();

    // Start server

    let t = thread::spawn(move || {
        run_server().unwrap();
    });

    run_gui();

    t.join().unwrap();
}
