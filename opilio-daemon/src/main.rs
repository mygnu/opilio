use anyhow::Result;
use std::{thread, time::Duration};

fn main() {
    loop {
        let sleep_for = get_sleep_time().unwrap_or_else(|_| {
            eprintln!("try again in 2 secs");
            Duration::from_secs(2)
        });
        thread::sleep(sleep_for);
    }
}

fn get_sleep_time() -> Result<Duration> {
    let mut serial = opilio_tui::OpilioSerial::new()?;
    let config = serial.get_config()?;

    let sleep_for = if config.general.sleep_after <= 5 {
        config.general.sleep_after as u64
    } else {
        config.general.sleep_after as u64 - 5
    };
    println!("sleep_for (config) {}s", sleep_for);

    Ok(Duration::from_secs(sleep_for))
}
