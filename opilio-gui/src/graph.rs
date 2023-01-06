use std::{collections::VecDeque, thread, time::Duration};

use cubic_spline::{Points, SplineOpts};
use iced::{
    widget::{
        canvas,
        canvas::{
            stroke, Cache, Canvas, Cursor, Geometry, LineCap, Path, Stroke,
            Text,
        },
    },
    Color, Element, Length, Point, Rectangle, Theme,
};

use crate::models::{RpmData, TempData};

#[derive(Default)]
pub(crate) struct Graph {
    pub data: VecDeque<RpmData>,
    pub max_val: f32,
    pub max_size: usize,
    pub graph_cache: Cache,
    pub background_cache: Cache,
    pub color_p: Color,
    pub color_f: Color,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    RpmData(RpmData),
    TempData(TempData),
}

impl Graph {
    pub(crate) fn new(max_size: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_size),
            graph_cache: Cache::new(),
            background_cache: Cache::new(),
            max_val: 0.0,
            max_size,
            color_p: Color::from_rgba8(255, 0, 0, 1.0),
            color_f: Color::from_rgba8(255, 255, 0, 0.6),
        }
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::RpmData(data) => {
                self.graph_cache.clear();

                self.max_val = self
                    .max_val
                    .max(data.pump)
                    .max(data.fan1)
                    .max(data.fan2)
                    .max(data.fan3);

                self.data.push_back(data);

                if self.data.len() >= 50 {
                    self.data.pop_front();
                }
            }
            Message::TempData(_temp_data) => {
                // self.cache.clear();

                // self.max_rpm = self
                //     .max_rpm
                //     .max(temp_data.pump)
                //     .max(temp_data.fan1)
                //     .max(temp_data.fan2)
                //     .max(temp_data.fan3);

                // self.data.push_back(temp_data);

                // if self.data.len() >= 50 {
                //     self.data.pop_front();
                // }
            }
        }
    }
}

impl<Message> canvas::Program<Message> for Graph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        if self.data.len() < 2 {
            return vec![];
        }

        let background = self.background_cache.draw(bounds.size(), |frame| {
            let mut text = Text::from("20.0".to_string());
            text.color = Color::WHITE;

            text.position = Point::new(frame.width() - 100.0, 100.0);
            // frame.translate(Point::new(frame.width(), 0.0));
            frame.fill_text(text);
        });

        let graph = self.graph_cache.draw(bounds.size(), |frame| {
            let size = self.data.len();
            let height = frame.height();
            let width = frame.width();
            let section = (width / (size as f32 - 1.0)) as f64;

            let mut pump = Vec::with_capacity(size);
            let mut fan1 = Vec::with_capacity(size);
            let mut fan2 = Vec::with_capacity(size);
            let mut fan3 = Vec::with_capacity(size);
            let opts = SplineOpts::new().tension(0.5).num_of_segments(10);

            let max_val = self.max_val;
            let default = RpmData::default();

            for i in 0..size {
                let x = (i as f64 * section) + 4.0;

                let data = self.data.get(i).unwrap_or_else(|| &default);
                pump.push((
                    x,
                    (height - data.pump / max_val * height) as f64 + 4.0,
                ));
                fan1.push((
                    x,
                    (height - data.fan1 / max_val * height) as f64 + 4.0,
                ));
                fan2.push((
                    x,
                    (height - data.fan2 / max_val * height) as f64 + 4.0,
                ));
                fan3.push((
                    x,
                    (height - data.fan3 / max_val * height) as f64 + 4.0,
                ));
            }

            let latest = self.data[size - 1];

            let mut text = Text::from(format!(
                "PUMP: {:.2}\nF1: {:.2}\nF2: {:.2}\nF3: {:.2}",
                latest.pump, latest.fan1, latest.fan2, latest.fan3
            ));
            text.color = Color::WHITE;

            text.position.x = frame.width() - 100.0;

            frame.fill_text(text);

            draw_line(frame, &opts, self.color_p, &pump);
            draw_line(frame, &opts, self.color_f, &fan1);
            draw_line(frame, &opts, self.color_f, &fan2);
            draw_line(frame, &opts, self.color_f, &fan3);
        });

        vec![background, graph]
    }
}

fn draw_line(
    frame: &mut canvas::Frame,
    opts: &SplineOpts,
    color: Color,
    data_xy: &[(f64, f64)],
) {
    let spline = Points::from(data_xy)
        .calc_spline(opts)
        .expect("cant construct spline points");
    let pump_path = build_path(spline);
    frame.stroke(
        &pump_path,
        Stroke {
            style: stroke::Style::Solid(color),
            line_cap: LineCap::Round,
            ..Stroke::default()
        },
    );
}

fn build_path(pump_spline: Points) -> Path {
    let pump_path = Path::new(|b| {
        let points = pump_spline.get_ref();
        // println!("points {}", points.len());
        let first = points.first().unwrap();
        b.move_to(Point::new(first.x as f32, first.y as f32));

        for point in pump_spline.get_ref().iter().skip(1) {
            b.line_to(Point::new(point.x as f32, point.y as f32))
        }
    });
    pump_path
}
