use roblib_client::transports::tcp::Tcp;
use roblib_client::Robot;
use roblib_tui::app::{App, AppResult};
use roblib_tui::event::{Event, EventHandler};
use roblib_tui::handler::handle_key_events;
use roblib_tui::tui::Tui;
use std::io;
use std::sync::Arc;
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> AppResult<()> {
    let robot = Arc::new(Robot::new(Tcp::connect("localhost:1110")?));

    // Create an application.
    let mut app = App::new(robot);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(&app, 250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Track()
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
