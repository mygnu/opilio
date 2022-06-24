#![no_std]

use defmt::Format;
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};

pub const MAX_DUTY_PERCENT: f32 = 100.0;
pub const MIN_DUTY_PERCENT: f32 = 10.0; // 10% usually when a pwm fan starts to spin
pub const MIN_TEMP: f32 = 15.0;
pub const MAX_TEMP: f32 = 40.0;

pub const CONFIG_SIZE: usize = core::mem::size_of::<Config>();

#[derive(Format, Copy, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum FanId {
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
}

#[derive(Copy, Debug, Clone, Format, Deserialize, Serialize, PartialEq)]
pub struct RpmData {
    pub f1: f32,
    pub f2: f32,
    pub f3: f32,
    pub f4: f32,
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

    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        from_bytes(slice).ok()
    }

    pub fn is_valid(&self) -> bool {
        self.min_duty >= MIN_DUTY_PERCENT
            && self.max_duty <= MAX_DUTY_PERCENT
            && self.min_temp >= MIN_TEMP
            && self.max_temp <= MAX_TEMP
    }

    pub fn to_vec(&self) -> Option<Vec<u8, CONFIG_SIZE>> {
        to_vec(&self).ok()
    }
}

#[derive(Format, Serialize, Deserialize, PartialEq)]
pub enum Command {
    SetConfig = 1,
    GetConfig = 2,
    SaveConfig = 3,
    GetTemp = 10,
    GetRpm = 11,
}

#[derive(Format, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct SerialData {
    command: Command,
    value: Vec<u8, 32>,
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
