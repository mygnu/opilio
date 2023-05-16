use anyhow::Result;
use app::App;
use fast_log::Config;
use log::error;

mod app;
mod config;

use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
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
    Frame, Terminal,
};

use crate::app::InputMode;

fn main() -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    listen_tcp(shutdown.clone());

    fast_log::init(Config::new().file("/tmp/opilio.log"))?;

    let mut app = App::new()?;
    // setup terminal
    enable_raw_mode().map_err(|e| {
        log::error!("Failed to enable raw mode: {}", e);
        e
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        log::error!("Failed to initialize terminal: {}", e);
        e
    })?;

    // create app and run it
    let tick_rate = Duration::from_millis(500);
    let res = run_app(&mut terminal, &mut app, tick_rate, shutdown);

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
    app: &mut App,
    tick_rate: Duration,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let mut prompt_tick = 0;
    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        let current_input_mode = app.input_mode;
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.input_mode = InputMode::ShowHelp,
                    KeyCode::Char('u') => {
                        app.input_mode = InputMode::UploadPrompt
                    }
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::SavePrompt
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        match current_input_mode {
                            InputMode::UploadPrompt => {
                                match app.upload_config() {
                                    Err(e) => {
                                        app.msg = e.to_string();
                                        app.input_mode = InputMode::ShowError
                                    }
                                    _ => {
                                        app.msg = "Uploaded config, settings will not survive power cycle, unless persisted!".to_string();
                                        app.input_mode = InputMode::ShowSuccess
                                    }
                                }
                            }
                            InputMode::SavePrompt => match app.save_config() {
                                Err(e) => {
                                    app.msg = e.to_string();
                                    app.input_mode = InputMode::ShowError
                                }
                                _ => {
                                    app.msg = "Settings Persisted on Chip!"
                                        .to_string();
                                    app.input_mode = InputMode::ShowSuccess
                                }
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            if matches!(
                current_input_mode,
                InputMode::ShowSuccess
                    | InputMode::ShowError
                    | InputMode::ShowHelp
            ) {
                prompt_tick += 1;
            } else {
                prompt_tick = 0;
            }
            if prompt_tick > 10 {
                app.input_mode = InputMode::Normal;
            }
            last_tick = Instant::now();
        }
        if shutdown.load(Ordering::Relaxed) {
            return Ok(());
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(40),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let rpm_chart = app.rpm_chart();
    f.render_widget(rpm_chart, layout_chunks[0]);

    let temp_chart = app.temp_chart();
    f.render_widget(temp_chart, layout_chunks[1]);

    let info_block = app.info_block();
    f.render_widget(info_block, layout_chunks[2]);
}

const SOCKET_ADDR: &str = "127.0.0.1:34254";
const QUIT: &str = "$'J>0w.2e&_]0W_B{|x5+d>;'PsxVGyw";
fn listen_tcp(bool: Arc<AtomicBool>) {
    while let Ok(mut stream) = TcpStream::connect(SOCKET_ADDR) {
        stream.write_all(QUIT.as_bytes()).ok();
        thread::sleep(Duration::from_secs(1));
    }
    let listener = TcpListener::bind(SOCKET_ADDR).unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            log::info!("connection");
            match stream {
                Ok(stream) => {
                    let b = bool.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stream);
                        for line in reader.lines().flatten() {
                            log::info!("{}", line);
                            if line == QUIT {
                                b.store(true, Ordering::Relaxed);
                                return;
                            }
                        }
                    });
                }
                Err(err) => {
                    log::info!("Error: {}", err);
                    break;
                }
            }
        }
    });
}
