use iced::{
    executor, widget::column, widget::container, Application, Command, Element,
    Length, Settings, Subscription, Theme,
};
use models::RpmData;
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

        self.graph.update(graph::Message::Data(RpmData {
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
