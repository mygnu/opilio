#![windows_subsystem = "windows"]
#![forbid(unsafe_code)]

extern crate iced;
extern crate plotters;

mod graphs;
mod running;

use std::time::Duration;

use iced::{
    alignment, executor,
    widget::{Column, Container, Row, Text},
    window::icon,
    Application, Color, Command, Element, Font, Length, Settings, Subscription,
    Theme,
};
use opilio_lib::{
    serial::{OpilioSerialDevice, PortWithSerialNumber},
    PID, VID,
};
use running::RunningState;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

const FONT_REGULAR: Font = Font::External {
    name: "sans-serif-regular",
    bytes: include_bytes!("../fonts/notosans-regular.ttf"),
};

const FONT_BOLD: Font = Font::External {
    name: "sans-serif-bold",
    bytes: include_bytes!("../fonts/notosans-bold.ttf"),
};

const ICON: &[u8; 16384] =
    include_bytes!(concat!(env!("OUT_DIR"), "/icon.bin"));

fn main() {
    let icon = tray_icon::icon::Icon::from_rgba(ICON.to_vec(), 64, 64)
        .expect("Failed to open icon");

    #[cfg(target_os = "linux")]
    let _event_loop = tao::event_loop::EventLoop::new();

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    tray_menu.append_items(&[
        &MenuItem::new("Show", true, None),
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Opilio Controller")
        .with_icon(icon)
        .build();

    if let Err(error) = tray_icon {
        match error {
            tray_icon::Error::OsError(err) => {
                std::process::exit(err.raw_os_error().unwrap_or(-1))
            }
            _ => std::process::exit(-1),
        }
    }

    let _ = OpilioController::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../fonts/notosans-regular.ttf")),
        window: iced::window::Settings {
            size: (550, 350),
            resizable: true,
            decorations: true,
            icon: Some(
                icon::from_rgba(ICON.to_vec(), 64, 64)
                    .expect("icon.bin contains valid rgba"),
            ),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    });
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    CloseModal,
    Enable,
    Disable,
    SetSleepAfter(u32),
    SetTriggerAboveAmbient(f32),
    SetUpperTemp(f32),
    SetPumpDuty(f32),
    ToggleBuzzer(bool),
    ToggleLed(bool),
    PortSelected(PortWithSerialNumber),
    ChangeState,
    Hide,
    Test,
    Save,
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortIdent {
    path: std::path::PathBuf,
}

#[derive(Default)]
struct HomeState {
    selected_port: Option<PortWithSerialNumber>,
    error_text: Option<String>,
}

impl HomeState {
    pub fn new() -> Self {
        let mut ports: Vec<PortWithSerialNumber> =
            OpilioSerialDevice::find_ports(VID, PID).unwrap_or_default();
        let selected_port = if ports.len() == 1 { ports.pop() } else { None };

        Self {
            selected_port,
            error_text: None,
        }
    }
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PortSelected(port) => {
                self.selected_port = Some(port);
            }
            Message::CloseModal => {
                self.error_text = None;
            }
            _ => {}
        }
        Command::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let label = Text::new("Select Serial Port").size(48);

        let options: Vec<_> = OpilioSerialDevice::find_ports(VID, PID)
            .unwrap_or_default()
            .into_iter()
            .collect();

        let pick_list = iced::widget::pick_list(
            options,
            self.selected_port.clone(),
            Message::PortSelected,
        )
        .width(Length::Fixed(250.0));

        let content = Column::new()
            .align_items(iced::Alignment::Center)
            .push(Row::new().spacing(20).push(label))
            .push(iced::widget::vertical_space(Length::Fixed(50.0)))
            .push(Row::new().spacing(20).push(Column::new().push(pick_list)))
            .push(Row::new().push(Text::new(format!(
                "Version {}",
                env!("CARGO_PKG_VERSION")
            ))));

        let content = iced_aw::Modal::new(self.error_text.is_some(), content, || {
            iced_aw::Card::new(
                Text::new("Failed to connect to Opilio Controller"),
                Text::new(self.error_text.clone().unwrap_or_else(|| "".to_owned())),
            )
            .foot(
                Column::new().padding(5).width(Length::Fill).push(
                    iced::widget::Button::new(
                        Text::new("Ok").horizontal_alignment(alignment::Horizontal::Center),
                    )
                    .width(Length::Fixed(100.0))
                    .on_press(Message::CloseModal),
                ).push(iced::widget::horizontal_rule(20)).push(Text::new("If you are unsure which port belongs to the controller, replug it and see which port temporarily disappears")),
            )
            .max_width(300.0)
            .on_close(Message::CloseModal)
            .into()

        })
        .backdrop(Message::CloseModal)
        .on_esc(Message::CloseModal);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2)
            .center_x()
            .center_y()
            .into()
    }
}

struct OpilioController {
    state: State,
}

enum State {
    Home(HomeState),
    Running(RunningState),
}

impl Application for OpilioController {
    type Message = self::Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn theme(&self) -> Self::Theme {
        Theme::custom(iced::theme::Palette {
            background: Color::from_rgb(
                0x20 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x25 as f32 / 255.0,
            ),
            text: Color::WHITE,
            primary: Color::from_rgb(
                0x5E as f32 / 255.0,
                0x7C as f32 / 255.0,
                0xE2 as f32 / 255.0,
            ),
            success: Color::from_rgb(
                0x12 as f32 / 255.0,
                0x66 as f32 / 255.0,
                0x4F as f32 / 255.0,
            ),
            danger: Color::from_rgb(
                0xC3 as f32 / 255.0,
                0x42 as f32 / 255.0,
                0x3F as f32 / 255.0,
            ),
        })
    }

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            OpilioController {
                state: State::Home(HomeState::new()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Opilio Controller".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id {
                1000 => {
                    return Command::single(
                        iced_native::command::Action::Window(
                            iced_native::window::Action::Close,
                        ),
                    );
                }
                1001 => {
                    return Command::single(
                        iced_native::command::Action::Window(
                            iced_native::window::Action::ChangeMode(
                                iced::window::Mode::Windowed,
                            ),
                        ),
                    );
                }
                _ => {}
            }
        }

        if let Message::Hide = message {
            return Command::single(iced_native::command::Action::Window(
                iced_native::window::Action::ChangeMode(
                    iced::window::Mode::Hidden,
                ),
            ));
        }
        if let Some(value) = self.run_if_port_is_selected() {
            return value;
        }
        match &mut self.state {
            State::Home(state) => state.update(message),
            State::Running(state) => state.update(message),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.state {
            State::Home(state) => state.view(),
            State::Running(state) => state.view(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        const FPS: u64 = 60;
        iced::time::every(Duration::from_millis(1000 / FPS))
            .map(|_| Message::Tick)
    }
}

impl OpilioController {
    fn run_if_port_is_selected(&mut self) -> Option<Command<Message>> {
        if let State::Home(ref mut home) = &mut self.state {
            if let Some(port) = home.selected_port.take() {
                match RunningState::new(port) {
                    Ok(running_state) => {
                        self.state = State::Running(running_state);
                    }
                    Err(error) => {
                        home.error_text = Some(format!("{error}"));
                        return Some(iced_native::Command::none());
                    }
                }

                return Some(Command::single(
                    iced_native::command::Action::Window(
                        iced_native::window::Action::Resize {
                            width: 1400,
                            height: 1000,
                        },
                    ),
                ));
            }
        }
        None
    }
}
