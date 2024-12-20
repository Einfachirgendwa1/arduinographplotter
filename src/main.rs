use std::{
    cell::OnceCell,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use colored::Colorize;
use log::{set_logger, set_max_level, Level, LevelFilter, Log};
use nannou::{prelude::*, App, Frame};

pub struct Logger {}

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{}",
                match record.level() {
                    Level::Error => format!("[ERROR] {}", record.args()).red(),
                    Level::Warn => format!("[WARN ] {}", record.args()).yellow(),
                    Level::Info => format!("[INFO ] {}", record.args()).cyan(),
                    Level::Debug => format!("[DEBUG] {}", record.args()).green(),
                    Level::Trace => format!("[TRACE] {}", record.args()).black(),
                }
            );
        }
    }

    fn flush(&self) {}
}

static START: Mutex<OnceCell<Instant>> = Mutex::new(OnceCell::new());

struct PointInTime {
    x: Duration,
    y: u32,
}

impl PointInTime {
    fn new(value: u32) -> Self {
        Self {
            x: Instant::now().duration_since(*START.lock().unwrap().get_or_init(Instant::now)),
            y: value,
        }
    }
}

fn main() {
    set_logger(&Logger {}).unwrap();

    #[cfg(debug_assertions)]
    set_max_level(LevelFilter::Debug);

    #[cfg(not(debug_assertions))]
    set_max_level(LevelFilter::Warn);

    nannou::app(model).run();
}

fn model(app: &App) -> Arc<Mutex<Vec<PointInTime>>> {
    app.new_window()
        .title("Arduino Messwerte")
        .view(view)
        .build()
        .unwrap();

    let model = Arc::new(Mutex::new(Vec::new()));
    let clone = model.clone();

    thread::spawn(move || loop {
        clone.lock().unwrap().push(PointInTime::new(read_value()));
    });

    model
}

fn view(app: &App, data: &Arc<Mutex<Vec<PointInTime>>>, frame: Frame) {
    frame.clear(BLACK);
    // let window = app.window_rect();
    //
    // app.draw()
    //     .line()
    //     .start(window.bottom_left())
    //     .end(window.top_right())
    //     .color(BLUEVIOLET)
    //     .weight(4.);
}

fn read_value() -> u32 {
    0
}
