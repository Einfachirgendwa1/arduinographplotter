use std::{
    cell::OnceCell,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use colored::Colorize;
use log::{set_logger, set_max_level, Level, LevelFilter, Log};
use nannou::{color::encoding::Srgb, prelude::*, App, Frame};

const COLOR: rgb::Rgb<Srgb, u8> = WHITE;

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
    y: i32,
}

impl PointInTime {
    fn new(value: i32) -> Self {
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

fn point2<A, B>(a: A, b: B) -> Point2
where
    A: AsPrimitive<f32>,
    B: AsPrimitive<f32>,
{
    pt2(a.as_(), b.as_())
}

fn view(app: &App, data: &Arc<Mutex<Model>>, frame: Frame) {
    let lock = data.lock().unwrap();
    frame.clear(BLACK);
    let window = app.window_rect();
    let draw = app.draw();

    let point_count = lock.points.len() as i32;
    if point_count == 0 {
        return;
    }

    let width = window.w() as i32;
    let point_width = width * 2 / point_count;
    let offset_down = window.h() as i32 / 2 - 100;

    let points =
        lock.points.iter().enumerate().map(|(index, point)| {
            point2((index as i32 * point_width) - width, point.y - offset_down)
        });

    draw.polyline().weight(3.).color(COLOR).points(points);

    draw.text("X-Achse")
        .color(COLOR)
        .xy(window.mid_bottom() + pt2(0., 50.))
        .font_size(32);

    draw.to_frame(app, &frame).unwrap();
}

fn read_value() -> i32 {
    sleep(Duration::from_millis(100));
    random_range(0, 1000)
}
