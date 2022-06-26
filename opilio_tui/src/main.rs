use anyhow::Result;
use serial_port::OpilioSerial;
use shared::{FanId, PID, SERIAL_DATA_SIZE, VID};

mod serial_port;

fn main() -> Result<()> {
    println!("{}", SERIAL_DATA_SIZE);
    let mut opilio = OpilioSerial::new(VID, PID)?;

    let config = opilio.get_config(FanId::F1)?;
    println!("{:?}", config);

    let rpm = opilio.get_rpm()?;
    println!("{:?}", rpm);
    Ok(())
}
