use rdev::{Event, EventType, Key, listen};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ENGLISH: &str = "com.apple.keylayout.UnicodeHexInput";

fn main() {
    println!("Input Method Switch started. Monitoring Ctrl+[ keypresses...");
    println!("Press Ctrl+C to exit.");

    // Track control key state
    let ctrl_pressed = Arc::new(Mutex::new(false));
    // Track last key press time to handle key combinations
    let last_key_press_time = Arc::new(Mutex::new(Instant::now()));
    // Define a threshold for considering keys as pressed together (e.g. 200ms)
    let threshold = Duration::from_millis(200);

    // Start listening for events
    if let Err(error) = listen(move |event| {
        callback(
            event,
            Arc::clone(&ctrl_pressed),
            Arc::clone(&last_key_press_time),
            threshold,
        )
    }) {
        println!("Error: {:?}", error);
    }
}

fn callback(
    event: Event,
    ctrl_pressed: Arc<Mutex<bool>>,
    last_key_press_time: Arc<Mutex<Instant>>,
    threshold: Duration,
) -> () {
    // if !matches!(event.event_type, EventType::MouseMove { .. })
    //     && !matches!(event.event_type, EventType::Wheel { .. })
    // {
    //     println!("{:?}", event.event_type);
    // }
    match event.event_type {
        EventType::KeyPress(Key::ControlLeft)
        | EventType::KeyPress(Key::ControlRight)
        | EventType::KeyPress(Key::Unknown(62)) => {
            // Control key pressed
            let mut ctrl = ctrl_pressed.lock().unwrap();
            *ctrl = true;
            *last_key_press_time.lock().unwrap() = Instant::now();
        }
        EventType::KeyRelease(Key::ControlLeft)
        | EventType::KeyRelease(Key::ControlRight)
        | EventType::KeyRelease(Key::Unknown(62)) => {
            // Control key released
            let mut ctrl = ctrl_pressed.lock().unwrap();
            *ctrl = false;
        }
        EventType::KeyPress(Key::LeftBracket) => {
            // Left bracket pressed
            let ctrl = *ctrl_pressed.lock().unwrap();
            let now = Instant::now();
            let last_press = *last_key_press_time.lock().unwrap();

            // Check if control is pressed or was pressed very recently
            if ctrl || now.duration_since(last_press) < threshold {
                // Switch to English input method
                switch_to_english_input_method();
            }
        }
        _ => {}
    }
}

fn switch_to_english_input_method() {
    // Call im-select and set to English
    // You might need to adjust this identifier based on your system configuration
    match Command::new("im-select").arg(ENGLISH).output() {
        Ok(_) => {
            println!("Switched to English input method");
        }
        Err(e) => {
            eprintln!("Failed to switch input method: {:?}", e);
            eprintln!("Make sure 'im-select' is installed and in your PATH");
        }
    }
}
