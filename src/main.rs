use std::{
    cell::OnceCell,
    io::{BufRead, BufReader},
    marker::PhantomData,
    process::exit,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use clap::Parser;
use colored::Colorize;
use log::{error, set_logger, set_max_level, warn, Level, LevelFilter, Log};
use nannou::{color::encoding::Srgb, prelude::*, App, Frame};
use serialport::SerialPort;

type NannouColor = rgb::Rgb<Srgb, u8>;

const fn rgb(red: u8, green: u8, blue: u8) -> NannouColor {
    NannouColor {
        red,
        green,
        blue,
        standard: PhantomData,
    }
}

const GRAPH_COLOR: NannouColor = rgb(243, 139, 168);
const AXIS_COLOR: NannouColor = rgb(203, 166, 247);
const TEXT_COLOR: NannouColor = rgb(205, 214, 244);
const BACKGROUND_COLOR: NannouColor = rgb(30, 30, 46);

const X_LABEL_COUNT: u32 = 3;
const Y_LABEL_COUNT: u32 = 3;

const X_LABEL_Y: f32 = 10.;

const MAX_POINT_AMOUNT: usize = 100;

#[derive(Debug, Parser, Clone)]
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
static CLI: Mutex<OnceCell<Cli>> = Mutex::new(OnceCell::new());

struct PointInTime {
    x: Instant,
    y: i32,
}

impl PointInTime {
    fn new(value: i32) -> Self {
        Self {
            x: Instant::now(),
            y: value,
        }
    }
}

struct Model {
    points: Vec<PointInTime>,
    top_y: f32,
    btm_y: f32,
}

fn main() {
    CLI.lock().unwrap().set(Cli::parse()).unwrap();

    set_logger(&Logger {}).unwrap();

    #[cfg(debug_assertions)]
    set_max_level(LevelFilter::Debug);

    START.lock().unwrap().set(Instant::now()).unwrap();

    #[cfg(not(debug_assertions))]
    set_max_level(LevelFilter::Error);

    nannou::app(model).run();
}

fn model(app: &App) -> Arc<Mutex<Model>> {
    let lock = CLI.lock().unwrap();
    let Cli { arduino } = lock.get().unwrap().clone();

    #[cfg(unix)]
    let arduino = arduino.unwrap_or("/dev/ttyACM0".to_string());

    let serialport = serialport::new(arduino.clone(), 9600)
        .timeout(Duration::from_secs(10))
        .open();

    let mut read_fn = match serialport {
        Ok(port) => get_value_from_arduino(BufReader::new(port)),
        Err(err) => {
            error!("Konnte den Port {arduino} nicht öffnen: {err}");

            match cfg!(debug_assertions) {
                false => exit(1),
                true => {
                    warn!("Benutze zufällige Werte stattdessen.");
                    random_values()
                }
            }
        }
    };

    app.new_window()
        .title("Arduino Messwerte")
        .view(view)
        .build()
        .unwrap();

    let model = Arc::new(Mutex::new(Model {
        points: Vec::new(),
        btm_y: 0.,
        top_y: 0.,
    }));
    let clone = model.clone();

    thread::spawn(move || loop {
        let value = read_fn();
        clone.lock().unwrap().points.push(PointInTime::new(value));
    });

    model
}

fn get_value_from_arduino(
    mut reader: BufReader<Box<dyn SerialPort>>,
) -> Box<dyn FnMut() -> i32 + Send> {
    let fun = move || loop {
        let mut buf = String::new();
        if let Err(err) = reader.read_line(&mut buf) {
            error!("Fehler beim Lesen vom arduino: {err}");
            continue;
        }
        let buf = buf.trim();

        if buf.is_empty() {
            continue;
        }

        match buf.parse() {
            Ok(value) => break value,
            Err(err) => {
                error!(
                "Invaliden input vom arduino empfangen, ignoriere.\nInput: {buf}, Fehler: {err}"
            );
                continue;
            }
        }
    };
    Box::new(fun)
}

fn random_values() -> Box<dyn FnMut() -> i32 + Send> {
    let mut x = 0;
    let fun = move || {
        sleep(Duration::from_millis(20));
        x += random_range(-2, 3);
        x
    };
    Box::new(fun)
}

fn point2<A, B>(a: A, b: B) -> Point2
where
    A: AsPrimitive<f32>,
    B: AsPrimitive<f32>,
{
    pt2(a.as_(), b.as_())
}

fn step(base: f32, target: f32) -> f32 {
    base + (target - base) * 0.07
}

fn view(app: &App, data: &Arc<Mutex<Model>>, frame: Frame) {
    let mut lock = data.lock().unwrap();

    frame.clear(BACKGROUND_COLOR);

    let window = app.window_rect();
    let draw = app.draw();

    let mut points = &lock.points[..];

    match points.len() {
        0 => return,
        x if x > MAX_POINT_AMOUNT => {
            let lowest = x - MAX_POINT_AMOUNT;
            points = &lock.points[lowest..(x - 1)];
        }
        _ => {}
    }

    let top = window.top() - PADDING_RECT.top;
    let bottom = window.bottom() + PADDING_RECT.bottom;
    let right = window.right() - PADDING_RECT.right;
    let left = window.left() + PADDING_RECT.left;

    let width = right - left;
    let height = top - bottom;

    let target_top_y = points.iter().map(|point| point.y).max().unwrap() as f32 + 10.;
    let target_btm_y = points.iter().map(|point| point.y).min().unwrap() as f32 - 10.;

    let top_y = step(lock.top_y, target_top_y);
    let btm_y = step(lock.btm_y, target_btm_y);

    let mut diff = top_y - btm_y;

    if diff == 0. {
        diff = 2.;
    }

    let point_height = height / diff;
    let point_width = width / MAX_POINT_AMOUNT as f32;

    let start_time = points.first().unwrap().x;
    let end_time = points.last().unwrap().x;

    let duration = end_time.duration_since(start_time);
    let label_time_diff = duration / X_LABEL_COUNT;

    let label_pos_diff = width / (X_LABEL_COUNT - 1) as f32;

    let mut time = start_time.duration_since(*START.lock().unwrap().get().unwrap());

    let mut pos = left;
    for _ in 0..X_LABEL_COUNT {
        draw.text(&format!("{time:?}"))
            .xy(point2(pos, bottom + X_LABEL_Y))
            .color(TEXT_COLOR);

        time += label_time_diff;
        pos += label_pos_diff;
    }

    // Achsen
    draw.polyline()
        .weight(6.)
        .points([
            point2(left, top),
            point2(left, bottom),
            point2(right, bottom),
        ])
        .color(AXIS_COLOR);

    // Graph
    draw.polyline()
        .weight(3.)
        .color(GRAPH_COLOR)
        .points(points.iter().enumerate().map(|(index, point)| {
            let x = left + index as f32 * point_width;
            let y = bottom + (point.y as f32 - btm_y) * point_height;
            (x, y)
        }));

    draw.text("X-Achse")
        .color(TEXT_COLOR)
        .xy(point2(0, bottom - 20.))
        .font_size(20);

    draw.to_frame(app, &frame).unwrap();

    lock.top_y = top_y;
    lock.btm_y = btm_y;
}
