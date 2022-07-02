use std::{
    io::{Read, Write},
    time::Duration,
};

use anyhow::{anyhow, Ok, Result};
use log::info;
use postcard::to_vec;
use serialport::{ClearBuffer, SerialPort, SerialPortType};
use shared::{
    Command, Config, Error, FanId, OverWireCmd, Stats, CONFIG_SIZE,
    STATS_DATA_SIZE,
};

pub struct OpilioSerial {
    vid: u16,
    pid: u16,
    port: Box<dyn SerialPort>,
}

impl OpilioSerial {
    pub fn new(vid: u16, pid: u16) -> Result<Self> {
        let port = open_port(vid, pid)?;
        Ok(Self { vid, pid, port })
    }

    pub fn get_stats(&mut self) -> Result<Stats> {
        self.clear_buffers()?;

        let cmd = OverWireCmd::new(Command::GetStats).to_vec()?;
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; STATS_DATA_SIZE];
        self.port.read(buffer.as_mut_slice())?;

        info!("data over serial: {:?}", buffer);

        let data = Stats::from_bytes(&buffer)?;
        info!("Received {:?}", data);
        Ok(data)
    }

    pub fn get_config(&mut self, fan_id: FanId) -> Result<Config> {
        self.clear_buffers()?;
        let cmd = OverWireCmd::new(Command::GetConfig)
            .data(to_vec(&fan_id).map_err(Error::from)?)
            .to_vec()?;
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; CONFIG_SIZE];

        self.port.read(buffer.as_mut_slice())?;

        let data = Config::from_bytes(&buffer)?;
        Ok(data)
    }

    fn clear_buffers(&mut self) -> Result<()> {
        if let Err(e) = self.port.clear(ClearBuffer::All) {
            log::error!("Error clearing buffers: {:?}: {}", e.kind(), e);
            self.port = open_port(self.vid, self.pid).map_err(|e| {
                anyhow!("Could not open port after error: {}", e)
            })?;
        };
        Ok(())
    }
}

fn open_port(vid: u16, pid: u16) -> Result<Box<dyn SerialPort>> {
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
        .ok_or_else(|| {
            anyhow!("Can't find port with vid {:#06x} pid {:#06x}", vid, pid)
        })?;
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_secs(1))
        .open()?;
    Ok(port)
}
