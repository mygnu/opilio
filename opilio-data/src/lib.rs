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
pub struct Config {
    pub id: FanId,
    enabled: bool,
    min_duty: f32,
    max_duty: f32,
    min_temp: f32,
    max_temp: f32,
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
enum Command {
    SetConfig = 1,
    GetConfig = 2,
    SaveConfig = 3,
    GetTemp = 10,
    GetRpm = 11,
}

#[derive(Serialize, Deserialize)]
struct SerialData {
    command: Command,
    value: Vec<u8, 32>,
}
