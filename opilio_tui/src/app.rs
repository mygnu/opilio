use crate::serial_port::OpilioSerial;
use anyhow::Result;

use shared::{PID, VID};

pub const TIME_SPAN: f64 = 60.0;
pub const TICK_DISTANCE: f64 = 0.5;
pub const TICKS_OVER_TIME: usize = (TIME_SPAN / TICK_DISTANCE) as usize;
pub const ZERO: f64 = 0.0;
pub const RPM_CHART_RATIO: u16 = 60;
pub const TEMP_CHART_RATIO: u16 = 40;

pub const RPM_Y_AXIS_MIN: f64 = 200.0;
pub const RPM_Y_AXIS_MAX: f64 = 2500.0;

pub const TEMP_Y_AXIS_MIN: f64 = 10.0;
pub const TEMP_Y_AXIS_MAX: f64 = 40.0;

pub struct App {
    pub opilio: OpilioSerial,
    pub last_point: f64,
    pub temp: Vec<(f64, f64)>,
    pub fan1: Vec<(f64, f64)>,
    pub fan2: Vec<(f64, f64)>,
    pub fan3: Vec<(f64, f64)>,
    pub fan4: Vec<(f64, f64)>,
    pub window: [f64; 2],
    pub current_temp: f64,
    pub current_rpms: [f64; 4],
}

impl App {
    pub fn new() -> Result<App> {
        let fan1 = vec![(ZERO, ZERO)];
        let fan2 = vec![(ZERO, ZERO)];
        let fan3 = vec![(ZERO, ZERO)];
        let fan4 = vec![(ZERO, ZERO)];
        let temp = vec![(ZERO, ZERO)];
        let opilio = OpilioSerial::new(VID, PID)?;
        Ok(App {
            opilio,
            fan1,
            fan2,
            fan3,
            fan4,
            window: [ZERO, TIME_SPAN],
            temp,
            current_temp: ZERO,
            current_rpms: [ZERO; 4],
            last_point: TIME_SPAN,
        })
    }

    pub fn on_tick(&mut self) {
        self.window[0] += TICK_DISTANCE;
        self.window[1] += TICK_DISTANCE;

        if self.temp.len() > TICKS_OVER_TIME {
            self.temp.remove(0);
            self.fan1.remove(0);
            self.fan2.remove(0);
            self.fan3.remove(0);
            self.fan4.remove(0);
        }

        self.last_point += TICK_DISTANCE;
        match self.opilio.get_stats() {
            Ok(stats) => {
                self.current_temp =
                    (self.current_temp + stats.temp1 as f64) / 2.0;
                let [rpm1, rpm2, rpm3, rpm4] = self.current_rpms;
                self.current_rpms = [
                    (stats.rpm1 as f64 + rpm1) / 2.0,
                    (stats.rpm2 as f64 + rpm2) / 2.0,
                    (stats.rpm3 as f64 + rpm3) / 2.0,
                    (stats.rpm4 as f64 + rpm4) / 2.0,
                ];
            }
            Err(e) => {
                log::error!("{:?}", e);
                // no reading but graph is still ticking
                self.current_temp = ZERO;
                self.current_rpms = [ZERO, ZERO, ZERO, ZERO];
            }
        };

        self.fan1.push((self.last_point, self.current_rpms[0]));
        self.fan2.push((self.last_point, self.current_rpms[1]));
        self.fan3.push((self.last_point, self.current_rpms[2]));
        self.fan4.push((self.last_point, self.current_rpms[3]));
        self.temp.push((self.last_point, self.current_temp));
    }
}
