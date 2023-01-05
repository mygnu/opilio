use anyhow::Result;
use std::{thread, time::Duration};

fn main() {
    loop {
        let sleep_for = get_sleep_time().unwrap_or_else(|_| {
            eprintln!("try again in 30 secs");
            Duration::from_secs(30)
        });
        thread::sleep(sleep_for);
    }
}

fn get_sleep_time() -> Result<Duration> {
    let mut serial = opilio_tui::OpilioSerial::new()?;
    println!("{:?}", serial);
    let config = serial.get_config()?;
    let sleep_for;

    if let Some(sleep_after) = config.general.sleep_after {
        sleep_for = if sleep_after <= 5 {
            sleep_after as u64
        } else {
            sleep_after as u64 - 5
        };
    } else {
        sleep_for = 600;
    }

    println!("sleep_for (config) {}s", sleep_for);

    Ok(Duration::from_secs(sleep_for))
}
