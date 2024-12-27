use std::{
    cell::OnceCell,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use colored::Colorize;
use log::{error, set_logger, set_max_level, Level, LevelFilter, Log};
use nannou::{color::encoding::Srgb, prelude::*, App, Frame};
use serialport::SerialPort;

const COLOR: rgb::Rgb<Srgb, u8> = GRAY;

const MAX_POINT_AMOUNT: usize = 100;

#[derive(Parser)]
struct Cli {
    #[cfg(unix)]
    /// Path to the arduino (/dev/...). Defaults to /dev/ttyACM0
    arduino: Option<String>,

    #[cfg(windows)]
    /// Port of the arduino (COM<some number>)
    arduino: String,
}

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
    let Cli { arduino } = Cli::parse();

    #[cfg(unix)]
    let arduino = arduino.unwrap_or("/dev/ttyACM0".to_string());

    let port = serialport::new(arduino.clone(), 9600)
        .timeout(Duration::from_secs(10))
        .open()
        .expect(&format!("Konnte den Port {arduino} nicht öffnen"));

    let mut reader = BufReader::new(port);

    app.new_window()
        .title("Arduino Messwerte")
        .view(view)
        .build()
        .unwrap();

    let model = Arc::new(Mutex::new(Model { points: Vec::new() }));
    let clone = model.clone();

    thread::spawn(move || loop {
        let point = PointInTime::new(read_value(&mut reader));
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

    let mut diff = (max_height - min_height) as f32;

    if diff == 0. {
        diff = 2.;
    }

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

fn read_value(reader: &mut BufReader<Box<dyn SerialPort>>) -> i32 {
    let mut buf = String::new();
    if let Err(err) = reader.read_line(&mut buf) {
        error!("Fehler beim Lesen vom arduino: {err}");
        return read_value(reader);
    }
    let buf = buf.trim();

    if buf.is_empty() {
        return read_value(reader);
    }

    match buf.parse() {
        Ok(value) => value,
        Err(err) => {
            error!(
                "Invaliden input vom arduino empfangen, ignoriere.\nInput: {buf}, Fehler: {err}"
            );
            read_value(reader)
        }
    }
}
