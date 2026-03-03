use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use lifocalc::{app::App, ui};
use ratatui::{Terminal, prelude::CrosstermBackend};

fn main() -> Result<()> {
    let mut terminal = initialize_terminal()?;
    let run_result = run_app(&mut terminal);
    restore_terminal(&mut terminal)?;
    run_result
}

fn initialize_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }

            if should_exit(key_event) {
                break;
            }

            handle_key(key_event, &mut app);
        }
    }

    Ok(())
}

fn should_exit(key_event: KeyEvent) -> bool {
    key_event.code == KeyCode::Esc
        || (key_event.code == KeyCode::Char('c')
            && key_event.modifiers.contains(KeyModifiers::CONTROL))
        || (key_event.code == KeyCode::Char('d')
            && key_event.modifiers.contains(KeyModifiers::CONTROL))
}

fn handle_key(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        KeyCode::Enter => app.submit_input(),
        KeyCode::Backspace => app.backspace(),
        KeyCode::Up => app.history_up(),
        KeyCode::Down => app.history_down(),
        KeyCode::Char(character)
            if !key_event.modifiers.contains(KeyModifiers::CONTROL)
                && !key_event.modifiers.contains(KeyModifiers::ALT) =>
        {
            app.insert_char(character);
        }
        _ => {}
    }
}
