use std::{
    io::{Read, Write},
    time::Duration,
};

use anyhow::{anyhow, bail, Ok, Result};
use log::info;
use opilio_lib::{
    Cmd, Config, Data, FanSetting, Stats, MAX_SERIAL_DATA_SIZE, OTW,
};
use serialport::{ClearBuffer, SerialPort, SerialPortType};

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

        let cmd = OTW::new(Cmd::GetStats, Data::Empty)?.to_vec()?;
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }

        // info!("data over serial: {:?}", buffer);

        let data = OTW::from_bytes(&buffer)?;
        info!("Received {:?}", data);
        match data.data() {
            Data::Stats(s) => Ok(s),
            _ => bail!("Failed to get data"),
        }
    }

    pub fn upload_config(&mut self, config: Config) -> Result<()> {
        for setting in config.settings.iter() {
            self.upload_setting(setting)?;
        }
        Ok(())
    }

    pub fn upload_setting(&mut self, setting: &FanSetting) -> Result<()> {
        self.clear_buffers()?;
        let cmd = OTW::new(Cmd::UploadSetting, Data::Setting(setting.clone()))?
            .to_vec()?;
        log::info!("sending config bytes {:?}", cmd);
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }
        let data = OTW::from_bytes(&buffer)?;
        log::info!("{:?}", data);
        Ok(())
    }

    pub fn get_config(&mut self) -> Result<Config> {
        self.clear_buffers()?;
        let cmd = OTW::new(Cmd::GetConfig, Data::Empty)?.to_vec()?;
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }
        let data = OTW::from_bytes(&buffer)?;
        match data.data() {
            Data::Config(s) => Ok(s),
            _ => bail!("Failed to get data"),
        }
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
            anyhow!("Port with vid {:#06x} pid {:#06x}, not found", vid, pid)
        })?;
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_secs(1))
        .open()?;
    Ok(port)
}
