use std::time::Duration;
use std::time::Instant;

use chrono::Utc;
use iced::widget::toggler;
use iced::{
    alignment,
    widget::{
        horizontal_rule, horizontal_space, vertical_space, Column, Container,
        Row, Text,
    },
    Alignment, Command, Element, Length,
};
use iced_aw::NumberInput;
use opilio_lib::Config;
use opilio_lib::SmartMode;
use opilio_tui::OpilioSerial;

use crate::graphs::{ChartGroup, MonitoringData};
use crate::Message;

pub struct RunningState {
    last_sample_time: Instant,
    opilio_serial: OpilioSerial,
    firmware_version_major: u8,
    firmware_version_minor: u8,
    hardware_version: u32,
    chart: ChartGroup,
    config: Config,
    error_text: Option<String>,
    update_interval: Duration,
}

impl RunningState {
    pub fn new<T>(_serial_port: &T) -> Result<Self, anyhow::Error>
    where
        T: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut opilio_serial = OpilioSerial::new()?;
        let firmware_version_major = 0;
        let firmware_version_minor = 0;
        let hardware_version = 0;

        let config = opilio_serial.get_config()?;

        Ok(RunningState {
            last_sample_time: Instant::now(),
            opilio_serial,
            firmware_version_major,
            firmware_version_minor,
            hardware_version,
            chart: Default::default(),
            config,
            error_text: None,
            update_interval: Duration::from_millis(500),
        })
    }
    #[inline]
    pub fn should_update(&self) -> bool {
        self.last_sample_time.elapsed() > self.update_interval
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                if !self.should_update() {
                    return Command::none();
                }

                self.last_sample_time = Instant::now();
                match self.opilio_serial.get_stats() {
                    Ok(stats) => {
                        let data = MonitoringData {
                            timestamp: Utc::now(),
                            pump_rpm: stats.pump1_rpm,
                            f1_rpm: stats.fan1_rpm,
                            f2_rpm: stats.fan2_rpm,
                            f3_rpm: stats.fan3_rpm,
                            ambient_temp: stats.ambient_temp,
                            liq_in_temp: stats.liquid_temp,
                            liq_out_temp: stats.liquid_out_temp,
                        };
                        self.chart.update(data)
                    }
                    Err(err) => {
                        self.error_text = Some(format!(
                            "Failed to get data from opilio ({err})"
                        ));
                    }
                }
            }
            Message::SetSleepAfter(sleep_after) => {
                self.config.general.sleep_after = Some(sleep_after);
            }
            Message::SetTriggerAboveAmbient(trigger_above_ambient) => {
                if let Some(smart_mode) = self.config.smart_mode.as_mut() {
                    smart_mode.trigger_above_ambient = trigger_above_ambient;
                }
            }
            Message::SetUpperTemp(upper_temp) => {
                if let Some(smart_mode) = self.config.smart_mode.as_mut() {
                    smart_mode.upper_temp = upper_temp;
                }
            }
            Message::SetPumpDuty(pump_duty) => {
                if let Some(smart_mode) = self.config.smart_mode.as_mut() {
                    smart_mode.pump_duty = pump_duty;
                }
            }
            Message::ToggleSmartMode(enable) => {
                if enable {
                    self.config.smart_mode = Some(SmartMode::default());
                } else {
                    self.config.smart_mode = None
                }
            }
            Message::Test => {
                if let Err(e) =
                    self.opilio_serial.upload_config(self.config.clone())
                {
                    self.error_text =
                        Some(format!("Failed to upload settings to opilio {e}"))
                }
            }
            Message::CloseModal => {
                self.error_text = None;
            }
            _ => {}
        }
        Command::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = Row::new().spacing(30);

        let content = content
            .push(self.view_left_column())
            .push(self.view_right_column());

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }

    pub fn view_left_column(&self) -> Element<'_, Message> {
        // let button = |label, tooltip, msg| {
        //     iced::widget::tooltip(
        //         iced::widget::button(
        //             iced::widget::text(label)
        //                 .horizontal_alignment(alignment::Horizontal::Center),
        //         )
        //         .padding(10)
        //         .width(Length::Fixed(110.0))
        //         .on_press(msg),
        //         tooltip,
        //         iced::widget::tooltip::Position::Top,
        //     )
        // };

        let mut content = Column::new()
            .spacing(5)
            .width(Length::Fixed(280.0))
            .push(
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                Row::new().push(
                                    Text::new(format!(
                                        "Firmware Version: {:X}.{:X}",
                                        self.firmware_version_major,
                                        self.firmware_version_minor
                                    ))
                                    .size(28),
                                ),
                            )
                            .push(
                                Row::new().push(
                                    Text::new(format!(
                                        "Hardware Version: {}",
                                        self.hardware_version
                                    ))
                                    .size(28),
                                ),
                            ),
                    )
                    .padding(15),
            )
            .push(horizontal_rule(10))
            .push(Text::new("General").size(28))
            .push(
                Row::new()
                    .push(Text::new("Sleep after seconds"))
                    .push(horizontal_space(Length::Fill))
                    .push(
                        NumberInput::new(
                            self.config.general.sleep_after.unwrap_or_default(),
                            500,
                            Message::SetSleepAfter,
                        )
                        .style(iced_aw::style::NumberInputStyles::Default)
                        .step(1)
                        .min(0),
                    )
                    .padding(5)
                    .spacing(5),
            )
            .push(
                toggler(
                    String::from("Smart Mode"),
                    self.config.smart_mode.is_some(),
                    Message::ToggleSmartMode,
                )
                .width(Length::Shrink)
                .spacing(10),
            );

        if let Some(ref smart_mode) = self.config.smart_mode {
            content = content
                .push(horizontal_rule(10))
                .push(Text::new("Smart Settings").size(28))
                .push(
                    Row::new()
                        .push(Text::new("Trigger Above Ambient"))
                        .push(horizontal_space(Length::Fill))
                        .push(
                            NumberInput::new(
                                smart_mode.trigger_above_ambient,
                                6.0,
                                Message::SetTriggerAboveAmbient,
                            )
                            .style(iced_aw::style::NumberInputStyles::Default)
                            .step(0.5)
                            .min(1.0),
                        )
                        .padding(5)
                        .spacing(5),
                )
                .push(
                    Row::new()
                        .push(Text::new("Upper Temp"))
                        .push(horizontal_space(Length::Fill))
                        .push(
                            NumberInput::new(
                                smart_mode.upper_temp,
                                50.0,
                                Message::SetUpperTemp,
                            )
                            .style(iced_aw::style::NumberInputStyles::Default)
                            .step(0.5)
                            .min(20.0),
                        )
                        .padding(5)
                        .spacing(5),
                )
                .push(
                    Row::new()
                        .push(Text::new("Pump Duty"))
                        .push(horizontal_space(Length::Fill))
                        .push(
                            NumberInput::new(
                                smart_mode.pump_duty,
                                100.0,
                                Message::SetPumpDuty,
                            )
                            .style(iced_aw::style::NumberInputStyles::Default)
                            .step(1.0)
                            .min(30.0),
                        )
                        .padding(5)
                        .spacing(5),
                );
        };
        // let hide_button = button("Hide Window")
        //     .style(iced::theme::Button::Primary)
        //     .on_press(Message::Hide);
        let test_button = iced::widget::tooltip(
            iced::widget::button(
                iced::widget::text("Test")
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .padding(10)
            .width(Length::Fixed(110.0))
            .style(iced::theme::Button::Positive)
            .on_press(Message::Test),
            "Testing settings will\nnot survive power-cycle.",
            iced::widget::tooltip::Position::Top,
        );
        let persist_button = iced::widget::tooltip(
            iced::widget::button(
                iced::widget::text("Save")
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .padding(10)
            .width(Length::Fixed(110.0))
            .style(iced::theme::Button::Primary)
            .on_press(Message::Test),
            "Persist test settings\n to opilio storage.",
            iced::widget::tooltip::Position::Top,
        );

        content = content
            .push(horizontal_rule(10))
            // .push(view_badges(&self.tec_status))
            .push(vertical_space(Length::Fill))
            .push(
                Row::new()
                    // .push(hide_button)
                    .push(test_button)
                    .push(horizontal_space(Length::Fill))
                    .push(persist_button)
                    .padding(2)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
            );

        iced_aw::Modal::new(self.error_text.is_some(), content, || {
            iced_aw::Card::new(
                Text::new("Error"),
                Text::new(
                    self.error_text.clone().unwrap_or_else(|| "".to_owned()),
                ),
            )
            .foot(
                Column::new().padding(5).width(Length::Fill).push(
                    iced::widget::Button::new(
                        Text::new("Ok").horizontal_alignment(
                            alignment::Horizontal::Center,
                        ),
                    )
                    .width(Length::Fixed(100.0))
                    .on_press(Message::CloseModal),
                ),
            )
            .max_width(300.0)
            .on_close(Message::CloseModal)
            .into()
        })
        .backdrop(Message::CloseModal)
        .on_esc(Message::CloseModal)
        .into()
    }

    pub fn view_right_column(&self) -> Element<'_, Message> {
        Column::new()
            .spacing(5)
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(iced::widget::vertical_space(Length::Fixed(5.0)))
            .push(self.chart.view())
            .into()
    }
}

// fn add_badge_if_flag_missing<'a, T>(
//     mut column: Column<'a, Message, iced::Renderer<T>>,
//     status: &'a TecStatus,
//     flag: TecStatus,
//     text: &'static str,
// ) -> Column<'a, Message, iced::Renderer<T>>
// where
//     T: iced::widget::text::StyleSheet
//         + iced_aw::badge::StyleSheet<Style = iced_aw::style::BadgeStyles>
//         + 'a,
// {
//     if !status.contains(flag) {
//         column = column.push(
//             iced_aw::Badge::new(Text::new(text).size(20).width(Length::Fill))
//                 .style(iced_aw::style::BadgeStyles::Danger),
//         )
//     }
//     column
// }

// fn add_badge_if_flag_set<'a, T>(
//     mut column: Column<'a, Message, iced::Renderer<T>>,
//     status: &'a TecStatus,
//     flag: TecStatus,
//     text: &'static str,
// ) -> Column<'a, Message, iced::Renderer<T>>
// where
//     T: iced::widget::text::StyleSheet
//         + iced_aw::badge::StyleSheet<Style = iced_aw::style::BadgeStyles>
//         + 'a,
// {
//     if status.contains(flag) {
//         column = column.push(
//             iced_aw::Badge::new(Text::new(text).size(20).width(Length::Fill))
//                 .style(iced_aw::style::BadgeStyles::Danger),
//         )
//     }
//     column
// }

// pub fn view_badges(status: &TecStatus) -> Element<'_, Message> {
//     let mut col = Column::new()
//         .spacing(12)
//         .align_items(Alignment::Center)
//         .width(Length::Fill);

//     col = add_badge_if_flag_missing(
//         col,
//         status,
//         TecStatus::POWER_OK,
//         "TEC NO POWER",
//     );
//     col = add_badge_if_flag_missing(
//         col,
//         status,
//         TecStatus::TEMP_SENSE_OK,
//         "TEMP SENSOR ERROR",
//     );
//     col = add_badge_if_flag_missing(
//         col,
//         status,
//         TecStatus::HUM_SENSE_OK,
//         "HUM SENSOR ERROR",
//     );

//     col = add_badge_if_flag_set(
//         col,
//         status,
//         TecStatus::PID_OUT_OF_RANGE,
//         "PID OUT OF RANGE",
//     );
//     col = add_badge_if_flag_set(
//         col,
//         status,
//         TecStatus::PID_INVALID,
//         "PID INVALID",
//     );
//     col =
//         add_badge_if_flag_set(col, status, TecStatus::OCP_ACTIVE, "OCP ACTIVE");

//     if status.contains(TecStatus::LOW_POWER_MODE_ACTIVE) {
//         col = col.push(
//             iced_aw::Badge::new(
//                 Text::new("TEC DISABLED").size(20).width(Length::Fill),
//             )
//             .style(iced_aw::style::BadgeStyles::Primary),
//         )
//     } else {
//         col = col.push(
//             iced_aw::Badge::new(
//                 Text::new("TEC ENABLED").size(20).width(Length::Fill),
//             )
//             .style(iced_aw::style::BadgeStyles::Primary),
//         )
//     }

//     col.into()
// }
