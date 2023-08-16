use anyhow::Result;
use crossterm::{
    event::{EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{prelude::*, widgets::*};
use roblib_client::{
    roblib::{event::ConcreteValue, roland::RolandAsync},
    transports::tcp::TcpAsync,
    RobotAsync,
};
use std::{fmt::Debug, io::Stdout, time::Duration};
use tokio::task::JoinHandle;
use tui_input::{backend::crossterm::EventHandler, Input};

type Tx = tokio::sync::broadcast::Sender<Msg>;
type Robot = RobotAsync<TcpAsync>;

static TABS: [&str; 3] = ["Main", "Ultra sensor", "Cmd Terminal"];

pub struct TUI {
    term: Terminal<CrosstermBackend<Stdout>>,

    robot: Robot,

    s: State,
}
#[derive(Debug, Default)]
struct State {
    index: usize,

    show_help: bool,

    input: Input,
    show_err: Option<(String, bool)>,
    cmd_hist: Vec<(String, Option<String>)>,

    speed: f64,
    drive: [bool; 4],
    redrive: bool,

    track: [bool; 4],
    ultra: Vec<f64>,
}
#[derive(Debug, Clone)]
pub enum Msg {
    Term(crossterm::event::Event),
    Roblib(roblib_client::roblib::event::ConcreteValue),
}

impl TUI {
    pub async fn new(robot: Robot) -> Result<Self> {
        Ok(Self {
            term: setup_terminal()?,
            robot,
            s: State::default(),
        })
    }

    pub fn spawn(mut self) -> (JoinHandle<Result<()>>, JoinHandle<Result<()>>) {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<Msg>(1024);

        let h1 = tokio::spawn(async move {
            let mut track_rx = self
                .robot
                .subscribe(roblib_client::roblib::event::TrackSensor)
                .await?;
            let mut ultra_rx = self
                .robot
                .subscribe(roblib_client::roblib::event::UltraSensor(
                    Duration::from_millis(100),
                ))
                .await?;

            self.term.clear()?;
            loop {
                self.render()?;
                let msg = tokio::select! {
                    Ok(msg) = rx.recv() => msg,
                    Ok(t) = track_rx.recv() => Msg::Roblib(ConcreteValue::TrackSensor(t)),
                    Ok(u) = ultra_rx.recv() => Msg::Roblib(ConcreteValue::UltraSensor(u)),
                };
                match msg {
                    Msg::Term(crossterm::event::Event::Key(key)) => {
                        // first close err popup
                        if self.s.show_err.is_some() {
                            self.s.show_err.take();
                            continue;
                        }

                        // global unescapable control binds
                        match key.code {
                            // Exit application on `Ctrl-C`
                            KeyCode::Char('c') | KeyCode::Char('C')
                                if key.modifiers == KeyModifiers::CONTROL =>
                            {
                                restore_terminal(&mut self.term)?;
                                println!("Bye!");
                                std::process::exit(0)
                            }
                            KeyCode::Char('?') => {
                                self.s.show_help = !self.s.show_help;
                                continue;
                            }
                            KeyCode::Tab => {
                                self.s.index = (self.s.index + 1) % TABS.len();
                                continue;
                            }
                            KeyCode::BackTab => {
                                if self.s.index > 0 {
                                    self.s.index -= 1;
                                } else {
                                    self.s.index = TABS.len() - 1;
                                }
                                continue;
                            }
                            _ => (),
                        }

                        if self.s.index == 2 {
                            match key.code {
                                KeyCode::Enter => {
                                    let s = self.s.input.value().trim();
                                    let cmd =
                                        match roblib_client::roblib::text_format::de::from_str(s) {
                                            Ok(cmd) => cmd,
                                            Err(e) => {
                                                self.s.show_err = Some((e.to_string(), true));
                                                self.s.input.reset();
                                                continue;
                                            }
                                        };
                                    let s = s.to_owned();
                                    self.s.input.reset();
                                    let res = crate::execute(cmd, &self.robot.transport).await?;
                                    let mut hist = (s, None);
                                    if let Some(s) = res {
                                        hist.1 = Some(s);
                                    }
                                    self.s.cmd_hist.push(hist);
                                    continue;
                                }
                                _ => {
                                    self.s
                                        .input
                                        .handle_event(&crossterm::event::Event::Key(key));
                                }
                            };
                        }
                        match key.code {
                            KeyCode::Char('Q') => {
                                restore_terminal(&mut self.term)?;
                                println!("Bye!");
                                std::process::exit(0)
                            }
                            KeyCode::Up => {
                                if self.s.speed + 5. <= 100. {
                                    self.s.speed += 5.;
                                }
                            }
                            KeyCode::Down => {
                                if self.s.speed - 5. >= 0. {
                                    self.s.speed -= 5.;
                                }
                            }

                            KeyCode::Char('w') | KeyCode::Char('W') => {
                                self.s.drive[0] = !self.s.drive[0];
                                self.s.redrive = true;
                                // self.robot.drive(1., 1.).await?;
                            }
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                self.s.drive[1] = !self.s.drive[1];
                                // self.robot.drive(-1., 1.).await?;
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                self.s.drive[2] = !self.s.drive[2];
                                // self.robot.drive(-1., -1.).await?;
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                self.s.drive[3] = !self.s.drive[3];
                                // self.robot.drive(1., -1.).await?;
                            }
                            KeyCode::Char(' ') => {
                                self.s.drive = Default::default();
                                self.robot.stop().await?;
                            }
                            _ => (),
                        }
                    }
                    Msg::Roblib(ConcreteValue::TrackSensor(t)) => {
                        self.s.track = t;
                    }
                    Msg::Roblib(ConcreteValue::UltraSensor(u)) => {
                        self.s.ultra.push(u);
                    }

                    _ => (),
                }

                if self.s.redrive {
                    // TODO: drive
                }
            }
        });

        let h2 = tokio::spawn(event_listener(tx.clone()));

        (h1, h2)
    }

    fn render(&mut self) -> Result<()> {
        self.term.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(10)].as_ref())
                .split(f.size());

            let tabs = Tabs::new(TABS.to_vec())
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .select(self.s.index)
                .highlight_style(
                    Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                );
            f.render_widget(tabs, layout[0]);

            [Self::render_main, Self::render_ultra, Self::render_cmdterm][self.s.index](
                &self.s, f, layout[1],
            );

            if self.s.show_help {
                Self::render_help(f);
            }

            if let Some((msg, err)) = &self.s.show_err {
                Self::render_err(f, msg, *err)
            }
        })?;
        Ok(())
    }
    fn render_main(s: &State, f: &mut Frame<impl Backend>, frame: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Max(3), Constraint::Min(0)])
            .split(frame);

        {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Max(14), Constraint::Min(0), Constraint::Max(14)])
                .split(layout[0]);

            let mut spans = vec![
                Span::from(" W "),
                Span::from(" A "),
                Span::from(" S "),
                Span::from(" D "),
            ];
            let c = speed_color(s.speed);
            for (i, b) in s.drive.iter().enumerate() {
                if *b {
                    spans[i].patch_style(Style::default().fg(c));
                }
            }
            let controls = Paragraph::new(Line::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Controls"))
                .alignment(Alignment::Center);
            f.render_widget(controls, layout[0]);

            let speed = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Speed"))
                .gauge_style(Style::default().fg(speed_color(s.speed)))
                .use_unicode(true)
                .ratio(s.speed / 100.);
            f.render_widget(speed, layout[1]);

            let track = Paragraph::new(Line::from(
                s.track
                    .iter()
                    .map(|b| {
                        if *b {
                            Span::from(" 0 ")
                        } else {
                            Span::styled(" X ", Style::default().add_modifier(Modifier::DIM))
                        }
                    })
                    .collect::<Vec<_>>(),
            ))
            .block(Block::default().borders(Borders::ALL).title("Track"))
            .alignment(Alignment::Center);
            f.render_widget(track, layout[2]);
        }
    }
    fn render_ultra(_s: &State, f: &mut Frame<impl Backend>, frame: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10),
                    // Constraint::Percentage(10),
                    // Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(frame);
        f.render_widget(Paragraph::new("Ultra"), layout[0])
    }
    fn render_cmdterm(s: &State, f: &mut Frame<impl Backend>, frame: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Max(3), Constraint::Min(0)].as_ref())
            .split(frame);

        let block = Block::default().borders(Borders::ALL);

        let width = layout[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = s.input.visual_scroll(width as usize);
        let inp = Paragraph::new(s.input.value())
            .scroll((0, scroll as u16))
            .block(block.clone().title("Cmd Input"));
        f.render_widget(inp, layout[0]);
        f.set_cursor(
            // Put cursor past the end of the input text
            layout[0].x + ((s.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            // Move one line down, from the border to the input line
            layout[0].y + 1,
        );

        let list: Vec<_> = s
            .cmd_hist
            .iter()
            .rev()
            .map(|s| {
                ListItem::new(Line::from(match &s.1 {
                    Some(ret) => format!("{} - {}", s.0, ret),
                    None => s.0.clone(),
                }))
            })
            .collect();
        let list = List::new(list)
            .block(block.title("Command History"))
            .start_corner(Corner::TopLeft);
        f.render_widget(list, layout[1]);
    }
    fn render_help(f: &mut Frame<impl Backend>) {
        let layout = centered_rect(60, 40, f.size());
        f.render_widget(Clear, layout);

        let i = Style::default().add_modifier(Modifier::ITALIC);
        let b = Style::default().add_modifier(Modifier::BOLD);
        let ctrls = [
            ("WASD", "Drive robot"),
            ("Up Arrow", "Increase drive speed"),
            ("Down Arrow", "Decrease drive speed"),
        ];
        let text = ctrls
            .map(|c| {
                Line::from(vec![
                    // Span::styled(c.0, i),
                    // Span::raw(": "),
                    Span::styled(c.0.to_owned() + ": ", i),
                    Span::styled(c.1, b),
                ])
            })
            .to_vec();
        let p = Paragraph::new(text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(p, layout);
    }
    fn render_err(f: &mut Frame<impl Backend>, msg: &str, err: bool) {
        let layout = centered_rect(40, 20, f.size());
        f.render_widget(Clear, layout);

        let msg = Line::from(msg);

        let mut desc = Line::from("\n\nPress any key to continue...");
        desc.patch_style(Style::default().add_modifier(Modifier::DIM));

        let mut b = Block::default()
            .title(if err { "Error" } else { "Message" })
            .borders(Borders::ALL);
        if err {
            b = b.border_style(Style::default().fg(Color::Red));
        }
        let p = Paragraph::new(vec![msg, desc])
            .block(b)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        f.render_widget(p, layout);
    }
}
impl Drop for TUI {
    fn drop(&mut self) {
        restore_terminal(&mut self.term).unwrap();
        eprintln!("TUI Dropped");
    }
}
impl Debug for TUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.s.fmt(f)
    }
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

async fn event_listener(tx: Tx) -> Result<()> {
    let mut stream = EventStream::new();
    loop {
        let e = stream.next().fuse().await;
        match e {
            Some(Ok(ev)) => {
                tx.send(Msg::Term(ev))?;
            }
            Some(Err(e)) => {
                log::error!("Input event stream error: {e}");
                Err(e)?
            }
            None => Err(anyhow::anyhow!("Input event stream ended"))?,
        }
    }
}

fn speed_color(speed: f64) -> Color {
    let cols = [
        Color::LightGreen,
        Color::Green,
        Color::LightYellow,
        Color::Yellow,
        Color::LightRed,
        Color::Red,
    ];
    cols[speed as usize * (cols.len() - 1) / 100]
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
