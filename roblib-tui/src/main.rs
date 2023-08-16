mod render;

use anyhow::Result;
use clap::Parser;
use roblib_client::{
    roblib::cmd::Concrete,
    transports::{tcp::TcpAsync, TransportAsync},
    RobotAsync,
};

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    #[arg(short, long)]
    exec: Option<String>,

    #[arg(short, long)]
    shell: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let robot = RobotAsync::new(TcpAsync::connect("localhost:1110").await?);

    if let Some(txt) = args.exec {
        let cmd: Concrete = roblib_client::roblib::text_format::de::from_str(&txt)?;
        dbg!(&cmd);
        let ret = execute(cmd, &robot.transport).await?;
        if let Some(s) = ret {
            println!("{s}");
        }
        // needed to ensure send before exit
        tokio::task::yield_now().await;
        return Ok(());
    }

    let (h1, h2) = render::TUI::new(robot).await?.spawn();
    let (h1, h2) = tokio::join!(h1, h2);
    h1??;
    h2??;

    Ok(())
}

// L workaround
pub(crate) async fn execute(cmd: Concrete, robot: &impl TransportAsync) -> Result<Option<String>> {
    Ok(match cmd {
        Concrete::MoveRobot(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::MoveRobotByAngle(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::StopRobot(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::Led(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::RolandServo(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::Buzzer(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::TrackSensor(c) => Some(format!("{:?}", robot.cmd(c).await?)),
        Concrete::UltraSensor(c) => Some(format!("{}", robot.cmd(c).await?)),

        Concrete::PinMode(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::ReadPin(c) => Some(format!("{}", robot.cmd(c).await?)),
        Concrete::WritePin(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::Pwm(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::Servo(c) => {
            robot.cmd(c).await?;
            None
        }

        Concrete::Subscribe(_) => Some(format!("Subscribe no supported")),
        Concrete::Unsubscribe(_) => Some(format!("Unsubscribe no supported")),

        Concrete::Nop(c) => {
            robot.cmd(c).await?;
            None
        }
        Concrete::GetUptime(c) => Some(format!("{:?}", robot.cmd(c).await?)),

        // Concrete::GetPosition(c) => {
        //     if let Some(p) = robot.cmd(c).await? {
        //         println!("{}", p)
        //     } else {
        //         println!("<")
        //     }
        // }
        Concrete::Abort(c) => {
            robot.cmd(c).await?;
            None
        }
    })
}
