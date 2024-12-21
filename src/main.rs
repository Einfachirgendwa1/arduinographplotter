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
const MAX_POINT_AMOUNT: usize = 100;

struct PaddingRect {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

const PADDING_RECT: PaddingRect = PaddingRect {
    top: 50.,
    bottom: 50.,
    left: 50.,
    right: 50.,
};

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
    set_max_level(LevelFilter::Error);

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

    let mut points = &lock.points[..];

    match points.len() {
        0 => return,
        x if x > MAX_POINT_AMOUNT => {
            let lowest = x as usize - MAX_POINT_AMOUNT;
            points = &lock.points[lowest..(x - 1) as usize];
        }
        _ => {}
    }

    let top = window.top() - PADDING_RECT.top;
    let bottom = window.bottom() + PADDING_RECT.bottom;
    let right = window.right() - PADDING_RECT.right;
    let left = window.left() + PADDING_RECT.left;

    let width = right - left;
    let height = top - bottom;

    let max_height = points.iter().map(|point| point.y).max().unwrap();
    let min_height = points.iter().map(|point| point.y).min().unwrap();

    let diff = (max_height - min_height) as f32;

    let point_height = height / diff;
    let point_width = width / MAX_POINT_AMOUNT as f32;

    // X-Achse
    draw.line()
        .start(point2(left, bottom))
        .end(point2(right, bottom))
        .color(COLOR);

    // Y-Achse
    draw.line()
        .start(point2(left, bottom))
        .end(point2(left, top))
        .color(COLOR);

    // Graph
    draw.polyline()
        .weight(3.)
        .color(COLOR)
        .points(points.iter().enumerate().map(|(index, point)| {
            let x = left + index as f32 * point_width;
            let y = bottom + (point.y - min_height) as f32 * point_height;
            (x, y)
        }));

    draw.text("X-Achse")
        .color(COLOR)
        .xy(point2(0, bottom - 20.))
        .font_size(20);

    draw.to_frame(app, &frame).unwrap();
}

fn read_value() -> i32 {
    sleep(Duration::from_millis(40));
    random_range(0, 1000)
}
