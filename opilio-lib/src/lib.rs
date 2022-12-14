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
pub const SWITCH_TEMP_BUFFER: f32 = 1.0;

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
    UploadAll = 10,
}

#[derive(Serialize, Clone)]
pub enum DataRef<'a> {
    SettingId(&'a Id),
    Setting(&'a FanSetting),
    Config(&'a Config),
    Stats(&'a Stats),
    Result(&'a Response),
    General(&'a GeneralConfig),
    Empty,
}

#[derive(Format, Debug, Deserialize, PartialEq)]
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
    pub liquid_out_temp: f32,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
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

#[derive(Format, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Error(Error),
}

#[derive(Format, Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct SmartMode {
    pub trigger_above_ambient: f32,
    pub upper_temp: f32,
    pub pump_duty: f32,
}

#[derive(Format, Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct GeneralConfig {
    pub sleep_after: u32,
}

#[derive(Format, Clone, Deserialize, Serialize, Debug, PartialEq)]
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
            general: GeneralConfig {
                sleep_after: DEFAULT_SLEEP_AFTER,
            },
            smart_mode: Some(SmartMode {
                trigger_above_ambient: 5.0,
                upper_temp: 40.0,
                pump_duty: 100.0,
            }),
            settings,
        }
    }
}

impl Config {
    pub fn is_valid(&self) -> bool {
        // running in smart mode only requires pump to be at a decent speed.
        if let Some(ref smart_mode) = self.smart_mode {
            return smart_mode.pump_duty >= 40.0;
        }
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

pub fn get_smart_duty(
    temp: f32,
    ambient_temp: f32,
    min_delta: f32,
    max_temp: f32,
    max_duty_value: u16,
    is_running: bool,
) -> u16 {
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
