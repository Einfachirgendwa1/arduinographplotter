use std::time::Duration;

struct PointInTime {
    x: Duration,
    y: u32
}

fn main() {
    let mut points = Vec::new();

    loop {
        points.push(read_point());
        graph_something(&points);
    }
}

fn read_point() -> PointInTime {
    todo!()
}

fn graph_something(points: &[PointInTime]) {
    todo!()
}