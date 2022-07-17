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
pub const STATS_DATA_SIZE: usize = 20;
pub const MAX_SERIAL_DATA_SIZE: usize = 256;
pub const DEFAULT_SLEEP_AFTER: u32 = 60 * 5; // five minutes

// requested from https:://pid.codes
// https://github.com/pidcodes/pidcodes.github.com/pull/751
pub const VID: u16 = 0x1209;
pub const PID: u16 = 0x2442;

pub mod error;
pub mod otw;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, Format, Serialize, Deserialize, PartialEq)]
pub enum Cmd {
    UploadSetting = 1,
    GetConfig = 2,
    SaveConfig = 3,
    GetStats = 4,
    Stats = 5,
    Config = 6,
    Result = 8,
    UploadGeneral = 9,
}

#[derive(Format, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Data {
    SettingId(Id),
    Setting(FanSetting),
    Config(Config),
    Stats(Stats),
    Result(Response),
    General(GeneralConfig),
    Empty,
}

#[derive(Format, Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Id {
    P1 = 1,
    F1 = 2,
    F2 = 3,
    F3 = 4,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct Stats {
    pub pump1_rpm: f32,
    pub fan1_rpm: f32,
    pub fan2_rpm: f32,
    pub fan3_rpm: f32,
    pub liquid_temp: f32,
    pub ambient_temp: f32,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct FanSetting {
    pub id: Id,
    pub enabled: bool,
    pub curve: [(f32, f32); 4],
}

pub type TempDuty = (f32, f32);

impl FanSetting {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            enabled: true,
            curve: [(15.0, 0.0), (20.0, 30.0), (25.0, 50.0), (40.0, 100.0)],
        }
    }

    pub fn get_duty(&self, temp: f32, max_duty_value: u16) -> u16 {
        if !self.enabled || temp < self.curve[0].0 {
            return 0;
        }

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

    pub fn is_valid(&self) -> bool {
        let mut previous = self.curve[0];
        for current in &self.curve[1..] {
            if current.0 < previous.0 || current.1 < previous.1 {
                return false;
            }
            previous = *current;
        }
        return true;
    }
}

#[derive(Format, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Error(Error),
}

#[derive(Format, Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct GeneralConfig {
    pub sleep_after: u32,
}

#[derive(Format, Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Config {
    pub general: GeneralConfig,
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
            settings,
            general: GeneralConfig {
                sleep_after: DEFAULT_SLEEP_AFTER,
            },
        }
    }
}

impl Config {
    pub fn is_valid(&self) -> bool {
        self.settings.iter().all(|c| c.is_valid())
    }

    pub fn set(&mut self, config: FanSetting) {
        for c in self.settings.iter_mut() {
            if c.id == config.id {
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
