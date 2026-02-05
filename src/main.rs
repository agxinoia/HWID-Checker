mod app;
mod ui;
mod info;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;
use ui::draw_ui;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Up | KeyCode::Char('k') => app.previous_tab(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_tab(),
                    KeyCode::Left | KeyCode::Char('h') => app.scroll_up(),
                    KeyCode::Right | KeyCode::Char('l') => app.scroll_down(),
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        app.goto_advanced();
                        app.set_status("Advanced mode - Serial comparison & spoofing advice".to_string());
                    }
                    KeyCode::Tab => {
                        match app.export_serials() {
                            Ok(filename) => {
                                app.set_status(format!("Exported to {}", filename));
                                app.reload_previous_serials();
                            }
                            Err(e) => app.set_status(format!("Export failed: {}", e)),
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
