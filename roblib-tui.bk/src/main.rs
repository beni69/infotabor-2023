use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use roblib_client::{roblib::event, transports::tcp::TcpAsync, RobotAsync};
use std::{fmt::Debug, io::Stdout, time::Duration};
use tokio::sync::mpsc;

type Robot = RobotAsync<TcpAsync>;
struct TermState {
    term: Terminal<CrosstermBackend<Stdout>>,

    w: bool,
    a: bool,
    s: bool,
    d: bool,
    track: [bool; 4],
}
#[derive(Debug)]
enum Msg {
    W(bool),
    A(bool),
    S(bool),
    D(bool),
    Track([bool; 4]),
}

impl TermState {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            term: setup_terminal()?,
            w: false,
            a: false,
            s: false,
            d: false,
            track: Default::default(),
        })
    }

    pub fn spawn(self) -> mpsc::UnboundedSender<Msg> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut state = TermState::new().await?;
            while let Some(msg) = rx.recv().await {
                match msg {
                    Msg::W(b) => state.w = b,
                    Msg::A(b) => state.a = b,
                    Msg::S(b) => state.s = b,
                    Msg::D(b) => state.d = b,
                    Msg::Track(d) => state.track = d,
                }
            }

            // type inference moment
            anyhow::bail!("Senders dropped");
            #[allow(unreachable_code)]
            anyhow::Ok(())
        });
        tx
    }

    fn render(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .split(frame.size());

            let track = Paragraph::new(format!("{:?}", self.track));
        })?;
        Ok(())
    }
}
impl Drop for TermState {
    fn drop(&mut self) {
        restore_terminal(&mut self.term).unwrap();
    }
}
impl Debug for TermState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TermState")
            .field("w", &self.w)
            .field("a", &self.a)
            .field("s", &self.s)
            .field("d", &self.d)
            .field("track", &self.track)
            .finish()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let robot = RobotAsync::new(TcpAsync::connect("localhost:1110").await?);
    let state = TermState::new().await?;

    let mut track_rx = robot.subscribe(event::TrackSensor).await?;
    tokio::spawn(async move {
        while let Ok(d) = track_rx.recv().await {
            // state
            //     .mutate(move |s| {
            //         s.track = d;
            //         Ok(())
            //     })
            //     .await?;
        }
        anyhow::Ok(())
    });

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(terminal.show_cursor()?)
}

fn render(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    Ok(loop {
        terminal.draw(|frame| {
            let greeting = Paragraph::new("Hello World!");
            frame.render_widget(greeting, frame.size());
        })?;
        if crossterm::event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if KeyCode::Char('q') == key.code {
                    break;
                }
            }
        }
    })
}
