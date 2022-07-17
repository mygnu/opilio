use anyhow::Result;
use opilio_lib::{PID, VID};
use tui::{
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

use crate::{
    config::{config_file, from_disk},
    serial_port::OpilioSerial,
};

const TIME_SPAN: f64 = 60.0;
const TICK_DISTANCE: f64 = 0.5;
const TICKS_OVER_TIME: usize = (TIME_SPAN / TICK_DISTANCE) as usize;
const ZERO: f64 = 0.0;

const RPM_Y_AXIS_MIN: f64 = 100.0;
// const RPM_Y_AXIS_MAX: f64 = 2500.0;

const TEMP_Y_AXIS_MIN: f64 = 10.0;
const TEMP_Y_AXIS_MAX: f64 = 40.0;

#[derive(Default, Copy, Clone)]
pub enum InputMode {
    #[default]
    Normal,
    ShowHelp,
    UploadPrompt,
    ShowError,
    ShowSuccess,
}

pub struct App {
    config_path: String,
    serial: OpilioSerial,
    last_point: f64,
    liquid_temp: Vec<(f64, f64)>,
    ambient_temp: Vec<(f64, f64)>,
    pump1: Vec<(f64, f64)>,
    fan1: Vec<(f64, f64)>,
    fan2: Vec<(f64, f64)>,
    fan3: Vec<(f64, f64)>,
    window: [f64; 2],
    current_temps: [f64; 2],
    current_rpms: [f64; 4],
    pub input_mode: InputMode,
    pub msg: String,
}

impl App {
    pub fn new() -> Result<App> {
        let pump1 = vec![(ZERO, ZERO)];
        let fan1 = vec![(ZERO, ZERO)];
        let fan2 = vec![(ZERO, ZERO)];
        let fan3 = vec![(ZERO, ZERO)];
        let liquid_temp = vec![(ZERO, ZERO)];
        let ambient_temp = vec![(ZERO, ZERO)];
        let mut serial = OpilioSerial::new(VID, PID)?;
        let config_path = config_file()?.display().to_string();

        let config = serial.get_config()?;
        log::info!("{config:?}");

        Ok(App {
            serial,
            pump1,
            fan1,
            fan2,
            fan3,
            window: [ZERO, TIME_SPAN],
            liquid_temp,
            config_path,
            ambient_temp,
            current_temps: [ZERO; 2],
            current_rpms: [ZERO; 4],
            last_point: TIME_SPAN,
            input_mode: InputMode::default(),
            msg: String::new(),
        })
    }

    pub fn upload_config(&mut self) -> Result<()> {
        let config = from_disk()?;
        log::info!("{:#?}", &config);
        self.serial.upload_config(config)?;

        Ok(())
    }

    pub fn on_tick(&mut self) {
        self.window[0] += TICK_DISTANCE;
        self.window[1] += TICK_DISTANCE;

        if self.liquid_temp.len() > TICKS_OVER_TIME {
            self.liquid_temp.remove(0);
            self.ambient_temp.remove(0);
            self.pump1.remove(0);
            self.fan1.remove(0);
            self.fan2.remove(0);
            self.fan3.remove(0);
        }

        self.last_point += TICK_DISTANCE;
        match self.serial.get_stats() {
            Ok(stats) => {
                let [current_liquid, current_ambient] = self.current_temps;
                self.current_temps = [
                    (stats.liquid_temp as f64 + current_liquid) / 2.0,
                    (stats.ambient_temp as f64 + current_ambient) / 2.0,
                ];

                let [rpm1, rpm2, rpm3, rpm4] = self.current_rpms;
                self.current_rpms = [
                    (stats.pump1_rpm as f64 + rpm1) / 2.0,
                    (stats.fan1_rpm as f64 + rpm2) / 2.0,
                    (stats.fan2_rpm as f64 + rpm3) / 2.0,
                    (stats.fan3_rpm as f64 + rpm4) / 2.0,
                ];
            }
            Err(e) => {
                log::error!("{:?}", e);
                // no reading but graph is still ticking
                self.current_temps = [ZERO, ZERO];
                self.current_rpms = [ZERO, ZERO, ZERO, ZERO];
            }
        };

        self.pump1.push((self.last_point, self.current_rpms[0]));
        self.fan1.push((self.last_point, self.current_rpms[1]));
        self.fan2.push((self.last_point, self.current_rpms[2]));
        self.fan3.push((self.last_point, self.current_rpms[3]));
        self.liquid_temp
            .push((self.last_point, self.current_temps[0]));
        self.ambient_temp
            .push((self.last_point, self.current_temps[1]));
    }

    pub fn temp_chart(&self) -> Chart {
        let temp_datasets = vec![
            Dataset::default()
                .name(format!("Liq: {:.2}°C", self.current_temps[0]))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Green))
                .graph_type(GraphType::Line)
                .data(&self.liquid_temp),
            Dataset::default()
                .name(format!("Amb: {:.2}°C", self.current_temps[1]))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&self.ambient_temp),
        ];
        let temp_chart = Chart::new(temp_datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Thermistor",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.window)
                    .labels(vec![
                        Span::styled(
                            "60s",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("30s"),
                        Span::styled(
                            "0s",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("°C")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([TEMP_Y_AXIS_MIN, TEMP_Y_AXIS_MAX])
                    .labels(vec![
                        Span::raw(format!("{:.1}", TEMP_Y_AXIS_MIN)),
                        // Span::raw(format!(
                        //     "{:.1}",
                        //     ((TEMP_Y_AXIS_MAX - TEMP_Y_AXIS_MIN) * 0.25)
                        //         + TEMP_Y_AXIS_MIN
                        // )),
                        // Span::raw(format!(
                        //     "{:.1}",
                        //     ((TEMP_Y_AXIS_MAX - TEMP_Y_AXIS_MIN) * 0.50)
                        //         + TEMP_Y_AXIS_MIN
                        // )),
                        // Span::raw(format!(
                        //     "{:.1}",
                        //     ((TEMP_Y_AXIS_MAX - TEMP_Y_AXIS_MIN) * 0.75)
                        //         + TEMP_Y_AXIS_MIN
                        // )),
                        Span::raw(format!("{:.1}", TEMP_Y_AXIS_MAX)),
                    ]),
            );
        temp_chart
    }

    pub fn rpm_chart(&self) -> Chart {
        let rpm_datasets = vec![
            Dataset::default()
                .name(format!("P1: {:.1}", self.current_rpms[0]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::LightCyan))
                .data(&self.pump1),
            Dataset::default()
                .name(format!("F1: {:.1}", self.current_rpms[1]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.fan1),
            Dataset::default()
                .name(format!("F2: {:.1}", self.current_rpms[2]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Blue))
                .data(&self.fan2),
            Dataset::default()
                .name(format!("F3: {:.1}", self.current_rpms[3]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&self.fan3),
        ];

        let pump_max = self
            .pump1
            .iter()
            .map(|v| v.1)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let fan1_max = self
            .fan1
            .iter()
            .map(|v| v.1)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let fan2_max = self
            .fan2
            .iter()
            .map(|v| v.1)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let fan3_max = self
            .fan3
            .iter()
            .map(|v| v.1)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        let max_rpm = [pump_max, fan1_max, fan2_max, fan3_max]
            .into_iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(RPM_Y_AXIS_MIN);
        let rpm_y_axis_max = (max_rpm * 1.2) + RPM_Y_AXIS_MIN;

        let rpm_y_axis_max = (((rpm_y_axis_max as usize) / 50) * 50) as f64;

        Chart::new(rpm_datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Opilio",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    // .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    // .labels(x_labels)
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("RPM")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::styled(
                            format!("{:.0}", RPM_Y_AXIS_MIN),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{:.0}", rpm_y_axis_max),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ])
                    .bounds([RPM_Y_AXIS_MIN, rpm_y_axis_max]),
            )
    }

    pub fn info_block(&self) -> Paragraph {
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::styled(
                        "q",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::raw("uit, "),
                    Span::styled(
                        "u",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Blue),
                    ),
                    Span::raw("pload config, "),
                    Span::styled(
                        "p",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Red),
                    ),
                    Span::raw("ersist, "),
                    Span::styled(
                        "h",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::raw("elp, "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green),
                    ),
                    Span::raw(" to go back."),
                ],
                Style::default(),
            ),
            InputMode::ShowHelp => (
                vec![
                    Span::raw("Press "),
                    Span::styled(
                        "Esc",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to stop editing, "),
                    Span::styled(
                        "Enter",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to record the message"),
                ],
                Style::default(),
            ),
            InputMode::ShowSuccess => (
                vec![
                    Span::styled(
                        "Success: ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green),
                    ),
                    Span::raw(&self.msg),
                ],
                Style::default(),
            ),
            InputMode::ShowError => (
                vec![
                    Span::styled(
                        "Error: ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Red),
                    ),
                    Span::raw(&self.msg),
                ],
                Style::default(),
            ),
            InputMode::UploadPrompt => (
                vec![Span::raw(format!(
                    "Uploading config '{}' to opilio board? Y/N:",
                    self.config_path
                ))],
                Style::default(),
            ),
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        Paragraph::new(text)
    }
}
