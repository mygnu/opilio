pub use opilio_lib::{PID, VID};
use std::{
    io::{Read, Write},
    time::Duration,
};

use anyhow::{anyhow, bail, Ok, Result};
use log::info;
use opilio_lib::{
    Cmd, Config, Data, DataRef, FanSetting, Stats, MAX_SERIAL_DATA_SIZE, OTW,
};
use serialport::{ClearBuffer, SerialPort, SerialPortType};

pub struct OpilioSerial {
    vid: u16,
    pid: u16,
    version: String,
    port: Box<dyn SerialPort>,
}

impl std::fmt::Debug for OpilioSerial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpilioSerial")
            .field("vid", &self.vid)
            .field("pid", &self.pid)
            .field("version", &self.version)
            .finish()
    }
}

impl OpilioSerial {
    pub fn new() -> Result<Self> {
        let vid = VID;
        let pid = PID;
        let (port, version) = open_port(vid, pid)?;
        Ok(Self {
            vid,
            pid,
            port,
            version,
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn get_stats(&mut self) -> Result<Stats> {
        self.clear_buffers()?;

        let cmd = OTW::serialised_vec(Cmd::GetStats, DataRef::Empty)?;
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }

        let response = OTW::from_bytes(&buffer)?;
        info!("Received {:?}", response);
        match response.data {
            Data::Stats(s) => Ok(s),
            _ => bail!("Failed to get data"),
        }
    }

    pub fn upload_config(&mut self, config: Config) -> Result<()> {
        self.clear_buffers()?;
        let cmd =
            OTW::serialised_vec(Cmd::UploadAll, DataRef::Config(&config))?;

        log::info!("sending all bytes {:?}", cmd);
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }

        Ok(())
    }

    pub fn save_config(&mut self) -> Result<()> {
        self.clear_buffers()?;
        let cmd = OTW::serialised_vec(Cmd::SaveConfig, DataRef::Empty)?;
        log::info!("saving config {:?}", cmd);
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }
        Ok(())
    }

    pub fn upload_setting(&mut self, setting: &FanSetting) -> Result<()> {
        self.clear_buffers()?;
        let cmd =
            OTW::serialised_vec(Cmd::UploadSetting, DataRef::Setting(setting))?;
        log::info!("sending setting bytes {:?}", cmd);
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }
        let response = OTW::from_bytes(&buffer)?;
        log::info!("{:?}", response);
        Ok(())
    }

    pub fn get_config(&mut self) -> Result<Config> {
        self.clear_buffers()?;
        let cmd = OTW::serialised_vec(Cmd::GetConfig, DataRef::Empty)?;
        self.port.write_all(&cmd)?;

        let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

        if self.port.read(buffer.as_mut_slice())? == 0 {
            bail!("Failed to read any bytes from the port")
        }
        let response = OTW::from_bytes(&buffer)?;
        match response.data {
            Data::Config(s) => Ok(s),
            _ => bail!("Failed to get data"),
        }
    }

    fn clear_buffers(&mut self) -> Result<()> {
        if let Err(e) = self.port.clear(ClearBuffer::All) {
            log::error!("Error clearing buffers: {:?}: {}", e.kind(), e);
            self.port = open_port(self.vid, self.pid)
                .map_err(|e| anyhow!("Could not open port after error: {}", e))?
                .0;
        };
        Ok(())
    }
}

fn open_port(vid: u16, pid: u16) -> Result<(Box<dyn SerialPort>, String)> {
    let mut version = "0.0.0".into();
    let port_name = serialport::available_ports()?
        .into_iter()
        .find(|info| {
            if let SerialPortType::UsbPort(port) = &info.port_type {
                println!("serial number {:?}", port.serial_number);
                if port.vid == vid && port.pid == pid {
                    version = port
                        .serial_number
                        .clone()
                        .unwrap_or_else(|| "0.0.0".to_string());
                    true
                } else {
                    false
                }
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
    Ok((port, version))
}
