mod animation;
mod app;
mod bad_apple_frames;
mod config;
mod metrics;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = config::settings::Settings::load();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (metric_tx, metric_rx) = mpsc::channel(8);
    tokio::spawn(metrics::collect_loop(metric_tx));

    let mut app = App::new(metric_rx, settings);
    let tick = Duration::from_millis(app.settings.refresh_rate_ms);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        (KeyCode::Char('1'), _) => app.set_tab(0),
                        (KeyCode::Char('2'), _) => app.set_tab(1),
                        (KeyCode::Char('3'), _) => app.set_tab(2),
                        (KeyCode::Char('t'), _) => app.cycle_theme(),
                        (KeyCode::Char('f'), _) => app.toggle_flow(),
                        _ => {}
                    }
                }
            }
        }

        app.tick();
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
