use std::{
    cell::OnceCell,
    sync::{Arc, Mutex},
    thread::{self, sleep},
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

struct Model {
    points: Vec<PointInTime>,
}

fn main() {
    set_logger(&Logger {}).unwrap();

    #[cfg(debug_assertions)]
    set_max_level(LevelFilter::Debug);

    #[cfg(not(debug_assertions))]
    set_max_level(LevelFilter::Warn);

    nannou::app(model).run();
}

fn model(app: &App) -> Arc<Mutex<Model>> {
    app.new_window()
        .title("Arduino Messwerte")
        .view(view)
        .build()
        .unwrap();

    let model = Arc::new(Mutex::new(Model { points: Vec::new() }));
    let clone = model.clone();

    thread::spawn(move || loop {
        let point = PointInTime::new(read_value());
        clone.lock().unwrap().points.push(point);
    });

    model
}

fn draw_line(
    draw: &Draw,
    index: f32,
    point_width: f32,
    point: &PointInTime,
    points: &[PointInTime],
) -> Option<()> {
    let next_index = index + 1.;
    let this_point = pt2(index * point_width, point.y as f32);
    let next_point = pt2(
        next_index as f32 * point_width,
        points.get(next_index as usize)?.y as f32,
    );

    draw.line()
        .start(this_point)
        .end(next_point)
        .color(BLUEVIOLET)
        .weight(4.);

    Some(())
}

fn view(app: &App, data: &Arc<Mutex<Model>>, frame: Frame) {
    let lock = data.lock().unwrap();
    frame.clear(BLACK);
    let window = app.window_rect();
    let draw = app.draw();
    let width = window.w();

    let points = lock.points.len() as f32;
    let point_width = width / points;

    for (index, point) in lock.points.iter().enumerate().map(|(i, x)| (i as f32, x)) {
        draw_line(&draw, index, point_width, point, &lock.points);
    }

    draw.text("X-Achse")
        .color(BLUEVIOLET)
        .xy(window.mid_bottom() + pt2(0., 50.))
        .font_size(32);

    draw.to_frame(app, &frame).unwrap();
}

fn read_value() -> u32 {
    sleep(Duration::from_millis(100));
    0
}
