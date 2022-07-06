use crate::serial_port::OpilioSerial;
use anyhow::Result;

use shared::{PID, VID};
use tui::{
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

const TIME_SPAN: f64 = 60.0;
const TICK_DISTANCE: f64 = 0.5;
const TICKS_OVER_TIME: usize = (TIME_SPAN / TICK_DISTANCE) as usize;
const ZERO: f64 = 0.0;

const RPM_Y_AXIS_MIN: f64 = 200.0;
const RPM_Y_AXIS_MAX: f64 = 2500.0;

const TEMP_Y_AXIS_MIN: f64 = 10.0;
const TEMP_Y_AXIS_MAX: f64 = 40.0;

#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Help,
}

pub struct App {
    opilio: OpilioSerial,
    last_point: f64,
    temp: Vec<(f64, f64)>,
    fan1: Vec<(f64, f64)>,
    fan2: Vec<(f64, f64)>,
    fan3: Vec<(f64, f64)>,
    fan4: Vec<(f64, f64)>,
    window: [f64; 2],
    current_temp: f64,
    current_rpms: [f64; 4],
    pub input_mode: InputMode,
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
            input_mode: InputMode::default(),
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

    pub fn temp_chart(&self) -> Chart {
        let temp_datasets = vec![Dataset::default()
            .name(format!("{:.2}°C", self.current_temp))
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .graph_type(GraphType::Line)
            .data(&self.temp)];
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
                        Span::raw(format!(
                            "{:.1}",
                            ((TEMP_Y_AXIS_MAX - TEMP_Y_AXIS_MIN) * 0.25)
                                + TEMP_Y_AXIS_MIN
                        )),
                        Span::raw(format!(
                            "{:.1}",
                            ((TEMP_Y_AXIS_MAX - TEMP_Y_AXIS_MIN) * 0.50)
                                + TEMP_Y_AXIS_MIN
                        )),
                        Span::raw(format!(
                            "{:.1}",
                            ((TEMP_Y_AXIS_MAX - TEMP_Y_AXIS_MIN) * 0.75)
                                + TEMP_Y_AXIS_MIN
                        )),
                        Span::raw(format!("{:.1}", TEMP_Y_AXIS_MAX)),
                    ]),
            );
        temp_chart
    }

    pub fn rpm_chart(&self) -> Chart {
        let rpm_datasets = vec![
            Dataset::default()
                .name(format!("{:.1}", self.current_rpms[0]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.fan1),
            Dataset::default()
                .name(format!("{:.1}", self.current_rpms[1]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.fan2),
            Dataset::default()
                .name(format!("{:.1}", self.current_rpms[2]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.fan2),
            Dataset::default()
                .name(format!("{:.1}", self.current_rpms[3]))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.fan2),
        ];

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
                            format!("{:.0}", RPM_Y_AXIS_MAX),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ])
                    .bounds([RPM_Y_AXIS_MIN, RPM_Y_AXIS_MAX]),
            )
    }

    pub fn info_block(&self) -> Paragraph {
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Commands: "),
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
                            .fg(Color::Yellow),
                    ),
                    Span::raw("pload config, "),
                    Span::styled(
                        "s",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::raw("ave, "),
                    Span::styled(
                        "t",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::raw("oggle persistent, "),
                    Span::styled(
                        "h",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::raw("elp."),
                ],
                Style::default().add_modifier(Modifier::SLOW_BLINK),
            ),
            InputMode::Help => (
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
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        Paragraph::new(text)
    }
}
