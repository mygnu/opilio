use anyhow::Result;
use app::App;
use fast_log::Config;
use log::error;

mod app;
mod serial_port;

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
    // fs::write("opilio.log", "")?;
    fast_log::init(Config::new().file("opilio.log"))?;

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
    loop {
        log::info!("tick");
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),

                    KeyCode::Char('h') => app.input_mode = InputMode::Help,
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    _ => (),
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
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
                Constraint::Length(30),
                Constraint::Length(20),
                Constraint::Length(2),
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
    thread::spawn(move || {
        let listener = TcpListener::bind(SOCKET_ADDR).unwrap();
        for stream in listener.incoming() {
            log::info!("connection");
            match stream {
                Ok(stream) => {
                    let b = bool.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stream);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                log::info!("{}", line);
                                if line == QUIT {
                                    b.store(true, Ordering::Relaxed);
                                    return;
                                }
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
