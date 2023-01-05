use iced::{
    executor,
    widget::{column, text},
    Alignment, Application, Command, Element, Event, Length, Settings,
    Subscription, Theme,
};
use models::RpmData;
use opilio_tui::OpilioSerial;
use rand::{thread_rng, Rng};

mod graph;
mod models;

pub fn main() -> iced::Result {
    Opilio::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Opilio {
    rpm_graph: graph::Graph,
    tmp_graph: graph::Graph,
    serial: opilio_tui::OpilioSerial,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Message(graph::Message),
    WindowEvent(Event),
}

impl Application for Opilio {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Opilio {
                rpm_graph: graph::Graph::new(),
                tmp_graph: graph::Graph::new(),
                serial: OpilioSerial::new().unwrap(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Opilio - Water Cooling Controller")
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Tick => {
                // let mut rng = thread_rng();

                if let Ok(stats) = self.serial.get_stats() {
                    self.rpm_graph.update(graph::Message::RpmData(RpmData {
                        pump: stats.pump1_rpm,
                        fan1: stats.fan1_rpm,
                        fan2: stats.fan2_rpm,
                        fan3: stats.fan3_rpm,
                    }));
                    self.tmp_graph.update(graph::Message::RpmData(RpmData {
                        pump: stats.liquid_temp,
                        fan1: stats.ambient_temp,
                        fan2: 0.0,
                        fan3: stats.liquid_out_temp,
                    }));
                }

                // self.rpm_graph.update(graph::Message::RpmData(RpmData {
                //     pump: rng.gen_range(100.0..1000.0),
                //     fan1: rng.gen_range(100.0..1000.0),
                //     fan2: rng.gen_range(100.0..1000.0),
                //     fan3: rng.gen_range(100.0..1000.0),
                // }));
                // self.tmp_graph.update(graph::Message::RpmData(RpmData {
                //     pump: rng.gen_range(22.0..24.0),
                //     fan1: rng.gen_range(22.0..24.0),
                //     fan2: rng.gen_range(22.0..24.0),
                //     fan3: rng.gen_range(22.0..24.0),
                // }));
            }
            Message::Message(_) => {
                println!("{:?}", msg);
            }
            Message::WindowEvent(_event) => {
                // println!("{:?}", event);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        column![
            text("RPM").width(Length::Shrink).size(20),
            self.rpm_graph
                .view()
                .map(move |message| Message::Message(message)),
            text("Temperature").width(Length::Shrink).size(20),
            self.tmp_graph
                .view()
                .map(move |message| Message::Message(message))
        ]
        // .padding(20)
        // .spacing(20)
        .align_items(Alignment::Center)
        .into()

        // container(content)
        //     .width(Length::Fill)
        //     .height(Length::Fill)
        //     .padding(20)
        //     .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch([
            iced::subscription::events().map(Message::WindowEvent),
            iced::time::every(std::time::Duration::from_millis(500))
                .map(|_| Message::Tick),
        ])
    }
}
