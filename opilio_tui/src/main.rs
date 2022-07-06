use anyhow::Result;
use app::{
    App, RPM_CHART_RATIO, RPM_Y_AXIS_MAX, RPM_Y_AXIS_MIN, TEMP_CHART_RATIO,
    TEMP_Y_AXIS_MAX, TEMP_Y_AXIS_MIN,
};
use fast_log::Config;
use log::error;

mod app;
mod serial_port;

use std::{
    fs, io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
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

fn main() -> Result<()> {
    fs::write("target/test.log", "")?;
    fast_log::init(Config::new().file("target/test.log")).unwrap();

    let app = App::new()?;
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(500);
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
) -> Result<()> {
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
    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(RPM_CHART_RATIO),
                Constraint::Percentage(TEMP_CHART_RATIO),
            ]
            .as_ref(),
        )
        .split(size);

    let rpm_chart = rpm_chart(app);
    f.render_widget(rpm_chart, layout_chunks[0]);

    let temp_chart = temp_chart(app);
    f.render_widget(temp_chart, layout_chunks[1]);
}

fn temp_chart(app: &App) -> Chart {
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
    temp_chart
}

fn rpm_chart(app: &App) -> Chart {
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
        )
}
