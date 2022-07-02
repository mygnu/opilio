use std::{
    io::{Read, Write},
    time::Duration,
};

use anyhow::{anyhow, Ok, Result};
use log::info;
use postcard::to_vec;
use serialport::{SerialPort, SerialPortType};
use shared::{
    Command, Config, Error, FanId, OverWireCmd, Stats, STATS_DATA_SIZE,
};

pub struct OpilioSerial {
    port: Box<dyn SerialPort>,
}

impl OpilioSerial {
    pub fn new(vid: u16, pid: u16) -> Result<Self> {
        let port_name = serialport::available_ports()?
            .into_iter()
            .find(|info| {
                if let SerialPortType::UsbPort(port) = &info.port_type {
                    port.vid == vid && port.pid == pid
                } else {
                    false
                }
            })
            .map(|p| p.port_name)
            .ok_or(anyhow!(
                "Can't find port with vid {:#06x} pid {:#06x}",
                vid,
                pid
            ))?;

        let port = serialport::new(port_name, 115_200)
            .timeout(Duration::from_secs(1))
            .open()?;

        Ok(Self { port })
    }

    pub fn get_stats(&mut self) -> Result<Stats> {
        let cmd = OverWireCmd::new(Command::GetStats).to_vec()?;
        self.port.write(&cmd)?;

        let mut serial_buf = vec![0; STATS_DATA_SIZE];
        self.port.read(serial_buf.as_mut_slice())?;

        info!("data over serial: {:?}", serial_buf);

        let data = Stats::from_bytes(&serial_buf)?;
        info!("Received {:?}", data);
        Ok(data)
    }

    pub fn get_config(&mut self, fan_id: FanId) -> Result<Config> {
        let cmd = OverWireCmd::new(Command::GetConfig)
            .data(to_vec(&fan_id).map_err(Error::from)?)
            .to_vec()?;
        self.port.write(&cmd)?;

        let mut serial_buf = vec![0; 64];

        self.port.read(serial_buf.as_mut_slice())?;

        let data = Config::from_bytes(&serial_buf)?;
        Ok(data)
    }

    // pub fn get_temp(&mut self) -> Result<f32> {
    //     let cmd = OverWireCmd::new(Command::GetStats).to_vec()?;
    //     self.port.write(&cmd)?;

    //     let mut serial_buf = vec![0; 8];

    //     self.port.read(serial_buf.as_mut_slice())?;
    //     info!("buf {:?}", serial_buf);

    //     let temp: Temp = from_bytes(&serial_buf).map_err(Error::from)?;
    //     info!("temp {:?}", temp);
    //     Ok(temp.value)
    // }
}
