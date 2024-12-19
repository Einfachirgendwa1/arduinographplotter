use std::{
    cell::OnceCell,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use nannou::{prelude::*, App, Frame};

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
}

fn read_value() -> u32 {
    0
}
