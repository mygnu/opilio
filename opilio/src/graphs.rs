use std::{collections::VecDeque, time::Duration};

use chrono::{DateTime, Local};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        canvas::{Cache, Frame, Geometry},
        Column, Container, Row,
    },
    Alignment, Element, Length, Size,
};
use plotters::{
    prelude::ChartBuilder,
    series::AreaSeries,
    style::{Color, IntoFont, RGBAColor, RGBColor, ShapeStyle},
};
use plotters_backend::{DrawingBackend, FontTransform};
use plotters_iced::{Chart, ChartWidget};

use crate::Message;

const PLOT_LINE_COLOR_TEMP: RGBColor = RGBColor(50, 175, 255);
const PLOT_LINE_COLOR_FAN: RGBColor = RGBColor(50, 255, 175);
const PLOT_LINE_COLOR_PUMP: RGBColor = RGBColor(255, 50, 175);
const GRID_BOLD_COLOR: RGBAColor = RGBAColor(100, 100, 100, 0.5);

pub struct MonitoringData {
    pub timestamp: DateTime<Local>,
    pub pump_rpm: f32,
    pub f1_rpm: f32,
    pub f2_rpm: f32,
    pub f3_rpm: f32,
    pub ambient_temp: f32,
    pub liq_in_temp: f32,
    pub liq_out_temp: f32,
}

pub struct ChartGroup {
    pump_rpm_chart: MonitoringChart,
    f1_rpm_chart: MonitoringChart,
    f2_rpm_chart: MonitoringChart,
    f3_rpm_chart: MonitoringChart,
    ambient_temp_chart: MonitoringChart,
    liq_in_temp_chart: MonitoringChart,
    liq_out_temp_chart: MonitoringChart,
    chart_height: f32,
}

const FAN_MIN_RPM: f32 = 400.0;
const FAN_MAX_RPM: f32 = 800.0;

impl Default for ChartGroup {
    fn default() -> Self {
        Self {
            pump_rpm_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Pump".to_owned(),
                2000.0,
                4000.0,
                "RPM".to_owned(),
                PLOT_LINE_COLOR_PUMP.mix(0.20),
            ),
            f1_rpm_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Fan 1".to_owned(),
                FAN_MIN_RPM,
                FAN_MAX_RPM,
                "RPM".to_owned(),
                PLOT_LINE_COLOR_FAN.mix(0.20),
            ),
            f2_rpm_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Fan 2".to_owned(),
                FAN_MIN_RPM,
                FAN_MAX_RPM,
                "RPM".to_owned(),
                PLOT_LINE_COLOR_FAN.mix(0.20),
            ),
            f3_rpm_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Fan 3".to_owned(),
                FAN_MIN_RPM,
                FAN_MAX_RPM,
                "RPM".to_owned(),
                PLOT_LINE_COLOR_FAN.mix(0.20),
            ),
            ambient_temp_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Ambient Temp".to_owned(),
                20.0,
                30.0,
                "C".to_owned(),
                PLOT_LINE_COLOR_TEMP.mix(0.20),
            ),
            liq_in_temp_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Liquid In Temp".to_owned(),
                20.0,
                30.0,
                "C".to_owned(),
                PLOT_LINE_COLOR_TEMP.mix(0.20),
            ),
            liq_out_temp_chart: MonitoringChart::new(
                Vec::new().into_iter(),
                "Liquid Out Temp".to_owned(),
                20.0,
                30.0,
                "C".to_owned(),
                PLOT_LINE_COLOR_TEMP.mix(0.20),
            ),
            chart_height: 140.0,
        }
    }
}

impl ChartGroup {
    pub fn update(&mut self, data: MonitoringData) {
        self.pump_rpm_chart.push_data(data.timestamp, data.pump_rpm);
        self.f1_rpm_chart.push_data(data.timestamp, data.f1_rpm);
        self.f2_rpm_chart.push_data(data.timestamp, data.f2_rpm);
        self.f3_rpm_chart.push_data(data.timestamp, data.f3_rpm);
        self.ambient_temp_chart
            .push_data(data.timestamp, data.ambient_temp);
        self.liq_in_temp_chart
            .push_data(data.timestamp, data.liq_in_temp);
        self.liq_out_temp_chart
            .push_data(data.timestamp, data.liq_out_temp);
    }

    pub fn view(&self) -> Element<Message> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.new_row().push(self.pump_rpm_chart.view()))
            .push(self.new_row().push(self.f2_rpm_chart.view()))
            .push(self.new_row().push(self.f3_rpm_chart.view()))
            .push(self.new_row().push(self.f1_rpm_chart.view()))
            .push(self.new_row().push(self.ambient_temp_chart.view()))
            .push(self.new_row().push(self.liq_in_temp_chart.view()))
            .push(self.new_row().push(self.liq_out_temp_chart.view()))
            .into()
    }

    pub fn new_row(&self) -> Row<Message> {
        Row::new()
            .spacing(0)
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fixed(self.chart_height))
            .align_items(Alignment::Center)
    }
}

struct MonitoringChart {
    title: String,
    min: f32,
    max: f32,
    unit: String,
    cache: Cache,
    data_points: VecDeque<(DateTime<Local>, f32)>,
    limit: Duration,
    color: RGBAColor,
}

impl MonitoringChart {
    fn new(
        data: impl Iterator<Item = (DateTime<Local>, f32)>,
        title: String,
        min: f32,
        max: f32,
        unit: String,
        color: RGBAColor,
    ) -> Self {
        let data_points: VecDeque<_> = data.collect();
        Self {
            title,
            min,
            max,
            unit,
            cache: Cache::new(),
            data_points,
            limit: Duration::from_secs(300),
            color,
        }
    }

    fn push_data(&mut self, time: DateTime<Local>, value: f32) {
        let cur_ms = time.timestamp_millis();
        if value > self.max {
            self.max = (value - self.min) * 0.05 + value;
        }
        if value < self.min {
            self.min = value - (self.min - value) * 0.05;
        }

        self.data_points.push_front((time, value));
        loop {
            if let Some((time, _)) = self.data_points.back() {
                let diff = Duration::from_millis(
                    (cur_ms - time.timestamp_millis()) as u64,
                );
                if diff > self.limit {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    fn view(&self) -> Element<Message> {
        Container::new(
            Column::new().width(Length::Fill).height(Length::Fill).push(
                ChartWidget::new(self).height(Length::Fill).resolve_font(
                    |_, style| match style {
                        plotters_backend::FontStyle::Bold => crate::FONT_BOLD,
                        _ => crate::FONT_REGULAR,
                    },
                ),
            ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}

#[derive(Default)]
struct ChartState {
    mouse_x_position: Option<f32>,
    bounds: iced::Rectangle,
}

impl Chart<Message> for MonitoringChart {
    type State = ChartState;

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::widget::canvas::Cursor,
    ) -> (iced_native::event::Status, Option<Message>) {
        if let iced::widget::canvas::Event::Mouse(mouse_event) = event {
            if mouse_event == iced_native::mouse::Event::CursorLeft {
                state.mouse_x_position = None;
                return (iced_native::event::Status::Ignored, None);
            }
        }
        if let iced::widget::canvas::Cursor::Available(point) = cursor {
            if point.x >= bounds.x && point.x <= bounds.x + bounds.width {
                state.mouse_x_position = Some(point.x);
                state.bounds = bounds;
            } else {
                state.mouse_x_position = None;
            }
        }
        (iced_native::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::widget::canvas::Cursor,
    ) -> iced_native::mouse::Interaction {
        if state.mouse_x_position.is_some() {
            iced_native::mouse::Interaction::Crosshair
        } else {
            iced_native::mouse::Interaction::Idle
        }
    }

    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(
        &self,
        state: &Self::State,
        mut chart: ChartBuilder<DB>,
    ) {
        //! This silently ignores error because there is nothing useful that can
        //! be done about them.
        let newest_time = self
            .data_points
            .front()
            .unwrap_or(&(Default::default(), 0.0))
            .0;
        let oldest_time = self
            .data_points
            .back()
            .unwrap_or(&(Default::default(), 0.0))
            .0;

        let hover_index = calc_hover_index(
            state.mouse_x_position,
            self.data_points.len(),
            state.bounds.width,
            state.bounds.x,
        )
        .unwrap_or(0);

        let caption = if let Some(value) = self.data_points.get(hover_index) {
            format!("{}  -  {:.2} {}", self.title, value.1, self.unit)
        } else {
            self.title.clone()
        };

        let mut chart = match chart
            .caption(
                caption,
                ("sans-serif", 22, &plotters::style::colors::WHITE),
            )
            .x_label_area_size(14)
            .y_label_area_size(28)
            .margin(10)
            .build_cartesian_2d(oldest_time..newest_time, self.min..self.max)
        {
            Ok(chart) => chart,
            Err(_) => return,
        };

        let _ = chart
            .configure_mesh()
            .bold_line_style(GRID_BOLD_COLOR)
            .axis_style(ShapeStyle::from(self.color).stroke_width(0))
            .y_labels(10)
            .x_labels(5)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&plotters::style::colors::WHITE)
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y| format!("{} {}", y, self.unit))
            .x_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&plotters::style::colors::WHITE),
            )
            .x_label_formatter(&|x| format!("{} ", x.time()))
            .draw();

        let _ = chart.draw_series(
            AreaSeries::new(
                self.data_points.iter().map(|x| (x.0, x.1)),
                self.min,
                self.color.mix(0.175),
            )
            .border_style(ShapeStyle::from(self.color).stroke_width(2)),
        );

        if let Some(data) = self.data_points.get(hover_index) {
            let _ = chart.draw_series(std::iter::once(
                plotters::prelude::Circle::new(
                    (data.0, data.1),
                    3_i32,
                    self.color.filled(),
                ),
            ));
        }
    }
}

fn calc_hover_index(
    x_pos_option: Option<f32>,
    data_size: usize,
    bounds_width: f32,
    bounds_x: f32,
) -> Option<usize> {
    if let Some(x_pos) = x_pos_option {
        let translation_factor = (data_size as f32) / bounds_width;
        let mut hover_index = data_size.saturating_sub(
            ((x_pos - bounds_x) * translation_factor).round() as usize,
        );
        if hover_index >= data_size {
            hover_index = data_size - 1;
        }
        return Some(hover_index);
    }
    None
}
