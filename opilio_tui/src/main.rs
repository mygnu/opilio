use anyhow::Result;
use fast_log::Config;
use log::error;
use serial_port::OpilioSerial;
use shared::{PID, VID};

mod serial_port;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use std::{
    fs, io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame, Terminal,
};
const TIME_SPAN: f64 = 60.0;
const TICK_DISTANCE: f64 = 0.5;
const TICKS_OVER_TIME: usize = (TIME_SPAN / TICK_DISTANCE) as usize;
const ZERO: f64 = 0.0;
const RPM_CHART_RATIO: u16 = 60;
const TEMP_CHART_RATIO: u16 = 40;

const RPM_Y_AXIS_MIN: f64 = 200.0;
const RPM_Y_AXIS_MAX: f64 = 2500.0;

const TEMP_Y_AXIS_MIN: f64 = 10.0;
const TEMP_Y_AXIS_MAX: f64 = 40.0;

struct App {
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
}

impl App {
    fn new() -> App {
        let fan1 = vec![(ZERO, ZERO)];
        let fan2 = vec![(ZERO, ZERO)];
        let fan3 = vec![(ZERO, ZERO)];
        let fan4 = vec![(ZERO, ZERO)];
        let temp = vec![(ZERO, ZERO)];
        let opilio = OpilioSerial::new(VID, PID).unwrap();
        App {
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
        }
    }

    fn on_tick(&mut self) {
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
                self.current_temp = stats.t1 as f64;
                self.current_rpms = [
                    stats.f1 as f64,
                    stats.f2 as f64,
                    stats.f3 as f64,
                    stats.f4 as f64,
                ];
            }
            Err(e) => {
                log::error!("{:?}", e);
            }
        };

        self.fan1.push((self.last_point, self.current_rpms[0]));
        self.fan2.push((self.last_point, self.current_rpms[1]));
        self.fan3.push((self.last_point, self.current_rpms[2]));
        self.fan4.push((self.last_point, self.current_rpms[3]));
        self.temp.push((self.last_point, self.current_temp));
    }
}

fn main() -> Result<()> {
    fs::write("target/test.log", "")?;
    fast_log::init(Config::new().file("target/test.log")).unwrap();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(500);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(RPM_CHART_RATIO),
                Constraint::Percentage(TEMP_CHART_RATIO),
            ]
            .as_ref(),
        )
        .split(size);

    let rpm_datasets = vec![
        Dataset::default()
            .name(format!("{:.1}", app.current_rpms[0]))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&app.fan1),
        Dataset::default()
            .name(format!("{:.1}", app.current_rpms[1]))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&app.fan2),
        Dataset::default()
            .name(format!("{:.1}", app.current_rpms[2]))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&app.fan2),
        Dataset::default()
            .name(format!("{:.1}", app.current_rpms[3]))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&app.fan2),
    ];

    let rpm_charts = Chart::new(rpm_datasets)
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
                .bounds(app.window),
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
        );
    f.render_widget(rpm_charts, chunks[0]);

    let temp_datasets = vec![Dataset::default()
        .name(format!("{:.2}°C", app.current_temp))
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(&app.temp)];
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
                .bounds(app.window)
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
    f.render_widget(temp_chart, chunks[1]);
}
