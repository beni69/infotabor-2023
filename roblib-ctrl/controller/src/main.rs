use anyhow::Result;
use roblib_client::{roblib::roland::Roland, transports::tcp::Tcp, Robot};
use std::{
    io::{BufRead, BufReader},
    time::Duration,
};

const BAUD: u32 = 115_200;

fn main() -> Result<()> {
    let serial = serialport::new("/dev/ttyACM0", BAUD)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let mut robot = Robot::new(Tcp::connect("roland:1110")?);

    let mut reader = BufReader::new(serial);
    let mut buf = String::new();
    let mut state = State::default();

    loop {
        match reader.read_line(&mut buf) {
            Ok(n) => {
                handle_line(&mut state, &mut robot, &buf[..n])?;
                buf.clear();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        };
    }
}

#[derive(Debug, Default)]
struct State {
    speed: u16,
    w: bool,
    a: bool,
    s: bool,
    d: bool,
}

fn handle_line(state: &mut State, robot: &mut Robot<Tcp>, line: &str) -> Result<()> {
    let mut sp = line.splitn(2, ' ');
    let key = sp.next().unwrap();
    let value = sp.next().unwrap().trim();
    eprintln!("{key} - {value}");

    match key {
        "speed" => state.speed = value.parse()?,
        "w" => state.w = value == "1",
        "a" => state.a = value == "1",
        "s" => state.s = value == "1",
        "d" => state.d = value == "1",

        "servo" => {
            let v: f64 = value.parse()?;
            robot.roland_servo(-(v - 45.))?;
            return Ok(());
        }
        "buzzer" => {
            if value == "1" {
                robot.buzzer(0.)?;
            } else {
                robot.buzzer(1.)?;
            }
            return Ok(());
        }

        _ => anyhow::bail!("unknown command: {key}"),
    }

    // move in place or don't move at all
    if state.w == state.s {
        let speed = state.speed as f64 / 100.;
        if state.a && !state.d {
            robot.drive(-speed, speed)?;
        } else if state.d && !state.a {
            robot.drive(speed, -speed)?;
        } else {
            robot.stop()?;
        }
        return Ok(());
    }

    let speed: f64 = if state.w {
        state.speed as f64 / 100.
    } else {
        -(state.speed as f64) / 100.
    };

    // drive straight
    if state.a == state.d {
        robot.drive(speed, speed)?;
        return Ok(());
    }

    // diagonal drive
    let turn_speed = speed / 3.;
    if state.a {
        robot.drive(turn_speed, speed)?;
        return Ok(());
    } else if state.d {
        robot.drive(speed, turn_speed)?;
        return Ok(());
    }

    unreachable!()
}
