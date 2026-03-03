use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use lifocalc::{
    app::App,
    keybindings::{Action, KeyBindings},
    ui,
};
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
    let keybindings = KeyBindings::load();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }

            if dispatch_action(key_event, &keybindings, &mut app) {
                break;
            }
        }
    }

    Ok(())
}

fn dispatch_action(key_event: KeyEvent, keybindings: &KeyBindings, app: &mut App) -> bool {
    if let Some(action) = keybindings.action_for_event(key_event) {
        match action {
            Action::Exit => return true,
            Action::Submit => app.submit_input(),
            Action::Backspace => app.backspace(),
            Action::DeleteWordBackward => app.delete_word_backward(),
            Action::CursorLeft => app.move_cursor_left(),
            Action::CursorRight => app.move_cursor_right(),
            Action::HistoryPrev => app.history_up(),
            Action::HistoryNext => app.history_down(),
            Action::ClearInput => app.clear_input(),
        }

        return false;
    }

    if let KeyCode::Char(character) = key_event.code {
        if !key_event.modifiers.contains(KeyModifiers::CONTROL)
            && !key_event.modifiers.contains(KeyModifiers::ALT)
        {
            app.insert_char(character);
        }
    }

    false
}
