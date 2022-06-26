use std::{
    io::{Read, Write},
    time::Duration,
};

use anyhow::{anyhow, Ok, Result};
use postcard::to_vec;
use serialport::{SerialPort, SerialPortType};
use shared::{Command, Config, Error, FanId, RpmData, SerialData};

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

    pub fn get_rpm(&mut self) -> Result<RpmData> {
        let cmd = SerialData::new(Command::GetRpm).to_vec()?;
        self.port.write(&cmd)?;

        let mut serial_buf = vec![0; 64];
        self.port.read(serial_buf.as_mut_slice())?;

        let data = RpmData::from_bytes(&serial_buf)?;
        Ok(data)
    }

    pub fn get_config(&mut self, fan_id: FanId) -> Result<Config> {
        let cmd = SerialData::new(Command::GetConfig)
            .data(to_vec(&fan_id).map_err(Error::from)?)
            .to_vec()?;
        self.port.write(&cmd)?;

        let mut serial_buf = vec![0; 64];

        self.port.read(serial_buf.as_mut_slice())?;

        let data = Config::from_bytes(&serial_buf)?;
        Ok(data)
    }
}
