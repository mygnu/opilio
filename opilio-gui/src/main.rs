use iced::{
    executor, widget::column, widget::container, Application, Command, Element,
    Length, Settings, Subscription, Theme,
};
use rand::{thread_rng, Rng};

pub fn main() -> iced::Result {
    Opilio::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Opilio {
    graph: graph::Graph,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
    Message(graph::Message),
}

impl Application for Opilio {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Opilio {
                graph: graph::Graph::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Opilio - Water Cooling Controller")
    }

    fn update(&mut self, _msg: Message) -> Command<Message> {
        let mut rng = thread_rng();

        self.graph.update(graph::Message::Data(TempData {
            pump: rng.gen_range(22.0..24.0),
            fan1: rng.gen_range(22.0..24.0),
            fan2: rng.gen_range(22.0..24.0),
            fan3: rng.gen_range(22.0..24.0),
        }));

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![self
            .graph
            .view()
            .map(move |message| Message::Message(message))];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::Tick)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TempData {
    pump: f32,
    fan1: f32,
    fan2: f32,
    fan3: f32,
}

mod graph {
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

    use crate::TempData;

    #[derive(Default)]
    pub(crate) struct Graph {
        pub data: VecDeque<TempData>,
        pub cache: Cache,
        pub color_p: Color,
        pub color_f: Color,
    }

    #[derive(Copy, Clone, Debug)]
    pub enum Message {
        Data(TempData),
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
                let size = self.data.len() as f32;
                let height = frame.height();
                let width = frame.width();
                let section = width / (size - 1.0);

                let mut pump = Vec::with_capacity(self.data.len());
                let mut fan1 = Vec::with_capacity(self.data.len());
                let mut fan2 = Vec::with_capacity(self.data.len());
                let mut fan3 = Vec::with_capacity(self.data.len());
                let opts = SplineOpts::new().tension(0.5).num_of_segments(10);
                for i in 0..self.data.len() {
                    pump.push((
                        i as f64 * section as f64,
                        (height - self.data[i].pump / 50.0 * height) as f64,
                    ));
                    fan1.push((
                        i as f64 * section as f64,
                        (height - self.data[i].fan1 / 50.0 * height) as f64,
                    ));
                    fan2.push((
                        i as f64 * section as f64,
                        (height - self.data[i].fan2 / 50.0 * height) as f64,
                    ));
                    fan3.push((
                        i as f64 * section as f64,
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
}
