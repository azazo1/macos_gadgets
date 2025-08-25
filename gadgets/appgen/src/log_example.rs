use chrono::Local;
use std::fs;
use std::io::Write;

fn main() {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("~/tmp/a.log")
        .unwrap();
    let now = Local::now();
    writeln!(f, "Current time: {}", now.format("%Y-%m-%d %H:%M:%S")).unwrap();
}
