use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }

        KeyCode::Up => {
            app.ch_speed(5.);
        }
        KeyCode::Down => {
            app.ch_speed(-5.);
        }

        KeyCode::Char('w') | KeyCode::Char('W') => {
            app.drive(1., 1.);
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.drive(-1., 1.);
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.drive(-1., -1.);
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.drive(1., -1.);
        }
        KeyCode::Char(' ') => {
            app.robot_stop();
        }

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
