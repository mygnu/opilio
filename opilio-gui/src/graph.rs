use crate::models::RpmData;
use std::collections::VecDeque;

use cubic_spline::{Points, SplineOpts};
use iced::{
    widget::{
        canvas,
        canvas::{
            stroke, Cache, Canvas, Cursor, Geometry, LineCap, Path, Stroke,
        },
    },
    Color, Element, Length, Point, Rectangle, Theme,
};

#[derive(Default)]
pub(crate) struct Graph {
    pub data: VecDeque<RpmData>,
    pub cache: Cache,
    pub color_p: Color,
    pub color_f: Color,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    Data(RpmData),
}

impl Graph {
    pub(crate) fn new() -> Self {
        Self {
            data: VecDeque::with_capacity(50),
            cache: Cache::new(),
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
            Message::Data(temp_data) => {
                self.cache.clear();

                self.data.push_back(temp_data);

                if self.data.len() >= 50 {
                    self.data.pop_front();
                }
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
        let geometry = self.cache.draw(bounds.size(), |frame| {
            let size = self.data.len();
            let height = frame.height();
            let width = frame.width();
            let section = (width / (size as f32 - 1.0)) as f64;

            let mut pump = Vec::with_capacity(size);
            let mut fan1 = Vec::with_capacity(size);
            let mut fan2 = Vec::with_capacity(size);
            let mut fan3 = Vec::with_capacity(size);
            let opts = SplineOpts::new().tension(0.5).num_of_segments(10);
            for i in 0..size {
                let x = i as f64 * section;
                pump.push((
                    x,
                    (height - self.data[i].pump / 50.0 * height) as f64,
                ));
                fan1.push((
                    x,
                    (height - self.data[i].fan1 / 50.0 * height) as f64,
                ));
                fan2.push((
                    x,
                    (height - self.data[i].fan2 / 50.0 * height) as f64,
                ));
                fan3.push((
                    x,
                    (height - self.data[i].fan3 / 50.0 * height) as f64,
                ));
            }

            draw_line(frame, &opts, self.color_p, pump);
            draw_line(frame, &opts, self.color_f, fan1);
            draw_line(frame, &opts, self.color_f, fan2);
            draw_line(frame, &opts, self.color_f, fan3);
        });

        vec![geometry]
    }
}

fn draw_line(
    frame: &mut canvas::Frame,
    opts: &SplineOpts,
    color: Color,
    data_xy: Vec<(f64, f64)>,
) {
    let pump_spline = Points::from(&data_xy)
        .calc_spline(opts)
        .expect("cant construct pump spline points");
    let pump_path = build_path(pump_spline);
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
