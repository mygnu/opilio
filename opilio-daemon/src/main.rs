use std::{thread, time::Duration};

use anyhow::{anyhow, Result};
use opilio_lib::{serial::OpilioSerialDevice, PID, VID};

fn main() {
    loop {
        let sleep_for = get_sleep_time().unwrap_or_else(|_| {
            eprintln!(
                "Failed to connect to opilio device, will try again in 30 secs"
            );
            Duration::from_secs(30)
        });
        thread::sleep(sleep_for);
    }
}

fn get_sleep_time() -> Result<Duration> {
    let ports = OpilioSerialDevice::find_ports(VID, PID)?;
    let port = ports
        .first()
        .ok_or_else(|| anyhow!("No Opilio device found"))?;
    let mut serial = OpilioSerialDevice::new(&port.port_name)?;
    println!("{serial:?}");
    let sleep_after_seconds = serial.ping()?;
    // sleep for 90% of the time, so we can ping again.
    // 1000ms * 90% = 900ms
    let sleep_for = Duration::from_millis((sleep_after_seconds as u64) * 900);
    println!(
        "Sleep settings {sleep_after_seconds}s, Ping again in {sleep_for:?}"
    );

    Ok(sleep_for)
}
