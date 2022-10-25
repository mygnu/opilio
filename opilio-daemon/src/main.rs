use std::{thread, time::Duration};

fn main() {
    loop {
        match opilio_tui::OpilioSerial::new() {
            Ok(_) => println!("Ping Success"),
            Err(e) => eprintln!("{e}"),
        };
        thread::sleep(Duration::from_secs(60))
    }
}
