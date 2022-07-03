#![no_std]

use defmt::Format;
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};

pub const MAX_DUTY_PERCENT: f32 = 100.0;
pub const MIN_DUTY_PERCENT: f32 = 10.0; // 10% usually when a pwm fan starts to spin
pub const MIN_TEMP: f32 = 15.0;
pub const MAX_TEMP: f32 = 40.0;

pub const CONFIG_SIZE: usize = 18;
pub const STATS_DATA_SIZE: usize = 20;
pub const MAX_SERIAL_DATA_SIZE: usize = 64;
pub const CMD_SERIAL_SIZE: usize = 1;
pub const OVER_WIRE_DATA_SIZE: usize = MAX_SERIAL_DATA_SIZE - CMD_SERIAL_SIZE;

pub const VID: u16 = 0x1209;
pub const PID: u16 = 0x2442;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, Format, Serialize, Deserialize, PartialEq)]
pub enum Command {
    SetConfig = 1,
    GetConfig = 2,
    SaveConfig = 3,
    GetStats = 4,
    Stats = 5,
    Config = 6,
    Result = 7,
}

#[derive(Format, Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum OtwData {
    FanId(FanId),
    Config(Config),
    Stats(Stats),
    Result(Response),
    Empty,
}

#[derive(Format, Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum FanId {
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct Stats {
    pub rpm1: f32,
    pub rpm2: f32,
    pub rpm3: f32,
    pub rpm4: f32,
    pub temp1: f32,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct Config {
    pub id: FanId,
    pub enabled: bool,
    pub min_duty: f32,
    pub max_duty: f32,
    pub min_temp: f32,
    pub max_temp: f32,
}

impl Config {
    pub fn new(id: FanId) -> Self {
        Self {
            id,
            enabled: true,
            min_duty: MIN_DUTY_PERCENT,
            max_duty: MAX_DUTY_PERCENT,
            min_temp: MIN_TEMP,
            max_temp: MAX_TEMP,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.min_duty >= MIN_DUTY_PERCENT
            && self.max_duty <= MAX_DUTY_PERCENT
            && self.min_temp >= MIN_TEMP
            && self.max_temp <= MAX_TEMP
    }
}

#[derive(Format, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Error(Error),
}

#[derive(Debug, Format, Serialize, PartialEq)]
pub struct OverTheWire {
    command: Command,
    data: OtwData,
}

impl OverTheWire {
    pub fn new(command: Command, data: OtwData) -> Result<Self> {
        if match command {
            Command::GetStats | Command::SaveConfig => {
                matches!(data, OtwData::Empty)
            }
            Command::Config | Command::SetConfig => {
                matches!(data, OtwData::Config(_))
            }
            Command::GetConfig => matches!(data, OtwData::FanId(_)),
            Command::Result => matches!(data, OtwData::Result(_)),
            Command::Stats => matches!(data, OtwData::Stats(_)),
        } {
            Ok(Self { command, data })
        } else {
            Err(Error::InvalidCmdDataPair)
        }
    }

    pub fn data(self) -> OtwData {
        self.data
    }
    pub fn command(&self) -> Command {
        self.command
    }

    pub fn to_vec(&self) -> Result<Vec<u8, MAX_SERIAL_DATA_SIZE>> {
        to_vec(&self).map_err(Error::from)
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        let command = from_bytes(slice)?;

        let data = match command {
            Command::Config | Command::SetConfig => {
                OtwData::Config(from_bytes(&slice[2..])?)
            }
            Command::Stats => OtwData::Stats(from_bytes(&slice[2..])?),
            Command::GetConfig => OtwData::FanId(from_bytes(&slice[2..])?),
            Command::Result => OtwData::Result(from_bytes(&slice[2..])?),
            Command::GetStats | Command::SaveConfig => OtwData::Empty,
        };
        Ok(Self { command, data })
    }
}

#[derive(Format, Deserialize, Serialize)]
pub struct Configs {
    pub data: Vec<Config, 4>,
    pub persistent: bool,
}

impl Default for Configs {
    fn default() -> Self {
        let mut data: Vec<_, 4> = Vec::new();
        data.push(Config::new(FanId::F1)).ok();
        data.push(Config::new(FanId::F2)).ok();
        data.push(Config::new(FanId::F3)).ok();
        data.push(Config::new(FanId::F4)).ok();

        Self {
            data,
            persistent: true,
        }
    }
}

impl AsRef<Vec<Config, 4>> for Configs {
    fn as_ref(&self) -> &Vec<Config, 4> {
        &self.data
    }
}

impl Configs {
    pub fn is_valid(&self) -> bool {
        self.as_ref().iter().all(|c| c.is_valid())
    }

    pub fn set(&mut self, config: Config) {
        for c in self.data.iter_mut() {
            if c.id == config.id {
                defmt::debug!("setting new config {:?}", config);
                *c = config;
                break;
            }
        }
    }
    pub fn get(&self, fan_id: FanId) -> Option<&Config> {
        self.data.iter().find(|&&c| c.id == fan_id)
    }
}

#[derive(Debug, Copy, Clone, Format, Serialize, Deserialize, PartialEq)]
pub enum Error {
    Deserialize,
    FlashErase,
    FlashRead,
    FlashWrite,
    SerialRead,
    SerialWrite,
    Serialize,
    InvalidCmdDataPair,
    Unknown,
}

impl From<postcard::Error> for Error {
    fn from(e: postcard::Error) -> Self {
        use postcard::Error::*;
        match e {
            DeserializeBadBool
            | DeserializeBadChar
            | DeserializeBadEncoding
            | DeserializeBadEnum
            | DeserializeBadOption
            | DeserializeBadUtf8
            | DeserializeBadVarint
            | SerdeDeCustom => Self::Deserialize,
            SerdeSerCustom
            | SerializeBufferFull
            | SerializeSeqLengthUnknown => Self::Serialize,
            _ => Self::Unknown,
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    extern crate std;
    use std::{error::Error, fmt::Display};

    impl Display for super::Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Error for super::Error {}
}
