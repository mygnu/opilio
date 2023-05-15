#![no_std]

use error::Error;
use fixed::types::extra::U4;
use heapless::Vec;
pub use otw::OTW;
use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};

pub type Fixed = fixed::FixedU32<U4>;

pub const MAX_DUTY_PERCENT: f32 = 100.0;
pub const MIN_DUTY_PERCENT: f32 = 10.0; // 10% usually when a pwm fan starts to spin
pub const MIN_TEMP: f32 = 15.0;
pub const MAX_TEMP: f32 = 50.0;

pub const CONFIG_SIZE: usize = 18;
pub const STATS_DATA_SIZE: usize = 20;
pub const MAX_SERIAL_DATA_SIZE: usize = 256;
pub const DEFAULT_SLEEP_AFTER: u32 = 60 * 5; // five minutes
pub const SWITCH_TEMP_BUFFER: f32 = 1.0;

// requested from https:://pid.codes
// https://github.com/pidcodes/pidcodes.github.com/pull/751
pub const VID: u16 = 0x1209;
pub const PID: u16 = 0x2442;

pub mod error;
pub mod otw;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Msg {
    Ping = 1,
    Pong = 2,
    GetConfig = 3,
    SaveConfig = 4,
    GetStats = 5,
    Stats = 6,
    Config = 7,
    Result = 8,
    UploadConfig = 9,
    Reload = 10,
}

#[derive(Serialize, Clone)]
pub enum DataRef<'a> {
    Config(&'a Config),
    Stats(&'a Stats),
    Result(&'a Response),
    Pong(&'a u32),
    Empty,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Data {
    Config(Config),
    Stats(Stats),
    Result(Response),
    Pong(u32),
    Empty,
}

#[derive(Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Id {
    P1 = 1,
    F1 = 2,
    F2 = 3,
    F3 = 4,
}

#[derive(Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Stats {
    pub pump1_rpm: f32,
    pub fan1_rpm: f32,
    pub fan2_rpm: f32,
    pub fan3_rpm: f32,
    pub liquid_temp: f32,
    pub ambient_temp: f32,
    pub liquid_out_temp: f32,
}

#[derive(Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FanSetting {
    pub id: Id,
    pub curve: [(f32, f32); 4],
}

pub type TempDuty = (f32, f32);

impl FanSetting {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            curve: if id == Id::P1 {
                [(25.0, 50.0), (30.0, 60.0), (35.0, 80.0), (40.0, 100.0)]
            } else {
                [(25.0, 0.0), (30.0, 30.0), (35.0, 50.0), (40.0, 100.0)]
            },
        }
    }

    pub fn get_duty(&self, temp: f32, max_duty_value: u16) -> u16 {
        let calculate =
            |(min_temp, min_duty): TempDuty, (max_temp, max_duty): TempDuty| {
                ((max_duty - min_duty) * (temp - min_temp)
                    / (max_temp - min_temp))
                    + min_duty
            };

        let duty_percent = if temp < self.curve[1].0 {
            calculate(self.curve[0], self.curve[1])
        } else if temp < self.curve[2].0 {
            calculate(self.curve[1], self.curve[2])
        } else if temp < self.curve[3].0 {
            calculate(self.curve[2], self.curve[3])
        }
        // return max duty
        else {
            self.curve[3].1
        };
        (max_duty_value as f32 / 100.0 * duty_percent) as u16
    }

    pub fn is_fan(&self) -> bool {
        !matches!(self.id, Id::P1)
    }

    pub fn is_valid(&self) -> bool {
        let mut previous = self.curve[0];
        for current in &self.curve[1..] {
            if current.0 <= previous.0 || current.1 <= previous.1 {
                return false;
            }
            previous = *current;
        }
        return true;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Response {
    Ok,
    Error(Error),
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SmartMode {
    pub trigger_above_ambient: f32,
    pub upper_temp: f32,
    pub pump_duty: f32,
}

impl Default for SmartMode {
    fn default() -> Self {
        Self {
            trigger_above_ambient: 5.0,
            upper_temp: 40.0,
            pump_duty: 95.0,
        }
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SwitchMode {
    #[default]
    On,
    Off,
}

impl SwitchMode {
    pub fn is_on(&self) -> bool {
        matches!(self, Self::On)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GeneralConfig {
    pub sleep_after: u32,
    pub led: SwitchMode,
    pub buzzer: SwitchMode,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            sleep_after: 3600,
            led: Default::default(),
            buzzer: Default::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    pub general: GeneralConfig,
    pub smart_mode: Option<SmartMode>,
    pub settings: Vec<FanSetting, 4>,
}

impl Default for Config {
    fn default() -> Self {
        let mut settings: Vec<_, 4> = Vec::new();
        settings.push(FanSetting::new(Id::P1)).ok();
        settings.push(FanSetting::new(Id::F1)).ok();
        settings.push(FanSetting::new(Id::F2)).ok();
        settings.push(FanSetting::new(Id::F3)).ok();

        Self {
            general: GeneralConfig::default(),
            smart_mode: Some(SmartMode::default()),
            settings,
        }
    }
}

impl Config {
    pub fn is_valid(&self) -> bool {
        if self.general.sleep_after < 5 {
            return false;
        }
        // running in smart mode only requires pump to be at a decent speed.
        if let Some(ref smart_mode) = self.smart_mode {
            return smart_mode.pump_duty >= 40.0;
        }
        self.settings.iter().all(|c| c.is_valid())
    }

    pub fn set(&mut self, config: FanSetting) {
        for c in self.settings.iter_mut() {
            if c.id == config.id {
                #[cfg(feature = "defmt")]
                defmt::debug!("setting new config {:?}", config);
                *c = config;
                break;
            }
        }
    }

    pub fn get(&self, fan_id: Id) -> Option<&FanSetting> {
        self.settings.iter().find(|&&c| c.id == fan_id)
    }

    pub fn to_vec(&self) -> Result<Vec<u8, MAX_SERIAL_DATA_SIZE>> {
        to_vec(&self).map_err(Error::from)
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        from_bytes(slice).map_err(Error::from)
    }
}

pub fn get_smart_duty(
    temp: f32,
    ambient_temp: f32,
    min_delta: f32,
    max_temp: f32,
    max_duty_value: u16,
    is_running: bool,
) -> u16 {
    let ambient_temp = if ambient_temp < -20.0 {
        // sane default if thermistor is unplugged
        22.0
    } else {
        ambient_temp
    };
    let trigger_temp = ambient_temp + min_delta;

    // if we are 1C below the minimum trigger delta turn off
    if is_running && temp <= trigger_temp - SWITCH_TEMP_BUFFER {
        return 0;
    }

    // if not running and temp delta isn't reached keep off
    if !is_running && temp <= trigger_temp {
        return 0;
    }

    // if we reached the max temp run at full speed.
    if temp >= max_temp {
        return max_duty_value;
    }

    let calculate = |(min_temp, min_duty): TempDuty,
                     (max_t, max_duty): TempDuty| {
        ((max_duty - min_duty) * (temp - min_temp) / (max_t - min_temp))
            + min_duty
    };

    let duty_percent = calculate((trigger_temp, 20.0), (max_temp, 100.0));

    (max_duty_value as f32 / 100.0 * duty_percent) as u16
}

#[cfg(feature = "std")]
pub mod serial {
    extern crate std;

    use std::{
        boxed::Box,
        dbg,
        io::{Read, Write},
        string::{String, ToString},
        time::Duration,
        vec,
        vec::Vec,
    };

    use anyhow::{anyhow, bail, Ok, Result};
    use log::info;
    use serialport::{ClearBuffer, DataBits, SerialPort, SerialPortType};

    use super::{Config, Data, DataRef, Msg, Stats, MAX_SERIAL_DATA_SIZE, OTW};

    pub struct OpilioSerialDevice {
        name: String,
        port: Box<dyn SerialPort>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PortWithSerialNumber {
        pub port_name: String,
        pub serial_number: Option<String>,
    }

    impl std::fmt::Display for PortWithSerialNumber {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.port_name)
        }
    }

    impl std::fmt::Debug for OpilioSerialDevice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("OpilioSerial")
                .field("name", &self.name)
                .finish()
        }
    }

    impl OpilioSerialDevice {
        pub fn new(port_name: &str) -> Result<Self> {
            let port = Self::open_port(port_name)?;
            Ok(Self {
                port,
                name: port_name.to_string(),
            })
        }

        pub fn find_ports(
            vid: u16,
            pid: u16,
        ) -> Result<Vec<PortWithSerialNumber>, anyhow::Error> {
            let ports: Vec<_> = serialport::available_ports()?
                .into_iter()
                .filter_map(|info| {
                    dbg!(&info);
                    if let SerialPortType::UsbPort(port) = info.port_type {
                        if port.vid == vid && port.pid == pid {
                            Some(PortWithSerialNumber {
                                port_name: info.port_name,
                                serial_number: port.serial_number,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            dbg!(&ports);
            Ok(ports)
        }

        pub fn version(&self) -> Result<String> {
            Ok("0.1.0".to_string())
        }

        pub fn ping(&mut self) -> Result<u32> {
            self.clear_buffers()?;
            let cmd = OTW::serialised_vec(Msg::Ping, DataRef::Empty)?;
            self.port.write_all(&cmd)?;

            let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

            if self.port.read(buffer.as_mut_slice())? == 0 {
                bail!("Failed to read any bytes from the port")
            }

            let response = OTW::from_bytes(&buffer)?;
            info!("Received {:?}", response);
            match response.data {
                Data::Pong(p) => Ok(p),
                _ => bail!("Failed to get data"),
            }
        }

        pub fn get_stats(&mut self) -> Result<Stats> {
            self.clear_buffers()?;

            let cmd = OTW::serialised_vec(Msg::GetStats, DataRef::Empty)?;
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
            let cmd = OTW::serialised_vec(
                Msg::UploadConfig,
                DataRef::Config(&config),
            )?;

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
            let cmd = OTW::serialised_vec(Msg::SaveConfig, DataRef::Empty)?;
            log::info!("saving config {:?}", cmd);
            self.port.write_all(&cmd)?;

            let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

            if self.port.read(buffer.as_mut_slice())? == 0 {
                bail!("Failed to read any bytes from the port")
            }
            Ok(())
        }

        pub fn get_config(&mut self) -> Result<Config> {
            self.clear_buffers()?;
            let cmd = OTW::serialised_vec(Msg::GetConfig, DataRef::Empty)?;
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

        pub fn reload(&mut self) -> Result<()> {
            self.clear_buffers()?;
            let cmd = OTW::serialised_vec(Msg::Reload, DataRef::Empty)?;
            self.port.write_all(&cmd)?;

            log::info!("resetting opilio {:?}", cmd);
            self.port.write_all(&cmd)?;

            let mut buffer = vec![0; MAX_SERIAL_DATA_SIZE];

            if self.port.read(buffer.as_mut_slice())? == 0 {
                bail!("Failed to read any bytes from the port")
            }
            Ok(())
        }

        fn clear_buffers(&mut self) -> Result<()> {
            if let Err(e) = self.port.clear(ClearBuffer::All) {
                log::error!("Error clearing buffers: {:?}: {}", e.kind(), e);
                self.port = Self::open_port(&self.name)?;
            };
            Ok(())
        }

        fn open_port(
            port_name: &str,
        ) -> Result<Box<dyn SerialPort>, anyhow::Error> {
            let port = serialport::new(port_name, 115_200)
                .timeout(Duration::from_millis(10))
                .data_bits(DataBits::Eight)
                .open()
                .map_err(|e| {
                    anyhow!("Failed to connect to {port_name}, ({e})")
                })?;
            Ok(port)
        }
    }
}
