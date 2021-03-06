#![no_std]

use defmt::Format;
use error::Error;
use heapless::Vec;
pub use otw::OTW;
use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};

pub const MAX_DUTY_PERCENT: f32 = 100.0;
pub const MIN_DUTY_PERCENT: f32 = 10.0; // 10% usually when a pwm fan starts to spin
pub const MIN_TEMP: f32 = 15.0;
pub const MAX_TEMP: f32 = 50.0;

pub const CONFIG_SIZE: usize = 18;
pub const CONFIGS_SIZE: usize = 96; // 76 bytes currently but can expand
pub const STATS_DATA_SIZE: usize = 20;
pub const MAX_SERIAL_DATA_SIZE: usize = 128;
pub const DEFAULT_SLEEP_AFTER_MS: u64 = 60 * 5 * 1000; // five minutes

// requested from https:://pid.codes
// https://github.com/pidcodes/pidcodes.github.com/pull/751
pub const VID: u16 = 0x1209;
pub const PID: u16 = 0x2442;

pub mod error;
pub mod otw;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, Format, Serialize, Deserialize, PartialEq)]
pub enum Cmd {
    SetConfig = 1,
    GetConfig = 2,
    SaveConfig = 3,
    GetStats = 4,
    Stats = 5,
    Config = 6,
    Result = 7,
    SetStandby = 8,
}

#[derive(Format, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Data {
    ConfigId(ConfId),
    Config(Config),
    Stats(Stats),
    Result(Response),
    U64(u64),
    Empty,
}

#[derive(Format, Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ConfId {
    P1 = 1,
    F1 = 2,
    F2 = 3,
    F3 = 4,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct Stats {
    pub rpm1: f32,
    pub rpm2: f32,
    pub rpm3: f32,
    pub rpm4: f32,
    pub liquid_temp: f32,
    pub ambient_temp: f32,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct FanSetting {
    pub id: ConfId,
    pub enabled: bool,
    pub min_duty: f32,
    pub max_duty: f32,
    pub min_temp: f32,
    pub max_temp: f32,
}

impl FanSetting {
    pub fn new(id: ConfId) -> Self {
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

    pub fn get_duty(&self, temp: f32, max_duty_value: u16) -> u16 {
        if !self.enabled {
            return 0;
        }

        let duty_percent = if temp <= self.min_temp {
            0.0 // stop the fan if tem is really low
        } else if temp >= self.max_temp {
            self.max_duty
        } else {
            (self.max_duty - self.min_duty) * (temp - self.min_temp)
                / (self.max_temp - self.min_temp)
                + self.min_duty
        };
        max_duty_value / 100 * duty_percent as u16
    }
}

#[derive(Format, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Error(Error),
}

#[derive(Format, Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Config {
    pub sleep_after_ms: u64,
    pub data: Vec<FanSetting, 4>,
}

impl Default for Config {
    fn default() -> Self {
        let mut data: Vec<_, 4> = Vec::new();
        data.push(FanSetting::new(ConfId::P1)).ok();
        data.push(FanSetting::new(ConfId::F1)).ok();
        data.push(FanSetting::new(ConfId::F2)).ok();
        data.push(FanSetting::new(ConfId::F3)).ok();

        Self {
            data,
            sleep_after_ms: DEFAULT_SLEEP_AFTER_MS,
        }
    }
}

impl AsRef<Vec<FanSetting, 4>> for Config {
    fn as_ref(&self) -> &Vec<FanSetting, 4> {
        &self.data
    }
}

impl Config {
    pub fn is_valid(&self) -> bool {
        self.as_ref().iter().all(|c| c.is_valid())
    }

    pub fn set(&mut self, config: FanSetting) {
        for c in self.data.iter_mut() {
            if c.id == config.id {
                defmt::debug!("setting new config {:?}", config);
                *c = config;
                break;
            }
        }
    }

    pub fn get(&self, fan_id: ConfId) -> Option<&FanSetting> {
        self.data.iter().find(|&&c| c.id == fan_id)
    }

    pub fn to_vec(&self) -> Result<Vec<u8, CONFIGS_SIZE>> {
        to_vec(&self).map_err(Error::from)
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        from_bytes(slice).map_err(Error::from)
    }
}
