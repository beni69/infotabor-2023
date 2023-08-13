use roblib_client::{
    roblib::{
        cmd::{self, Command},
        event,
        roland::Roland,
    },
    transports::tcp::Tcp,
    Result, Robot,
};

static WEIGHTS: [f64; 4] = [-2., -1., 1., 2.];

fn main() -> Result<()> {
    let robot = Box::leak(Box::new(Robot::new(Tcp::connect("roland:1110")?)));

    let on_track = |d: <cmd::TrackSensor as Command>::Return| {
        if d == [true, true, true, true] {
            println!("track lost");
            // robot.stop()?;
            return Ok(());
        }

        let mut left = 0.;
        let mut right = 0.;

        // if !d[0] {
        //     right += 0.2;
        // }
        // if !d[1] {
        //     right += 0.2;
        // }
        // if !d[2] {
        //     left += 0.2;
        // }
        // if !d[3] {
        //     left += 0.2;
        // }

        // d.iter()
        //     .enumerate()
        //     .map(|(i, b)| (!b as u8 as f64) * WEIGHTS[i]);

        for (b, w) in d.iter().filter(|b| !**b).zip(WEIGHTS.iter()) {
            if *w < 0. {
                left += -w;
            } else {
                right += w;
            }
        }

        println!("{d:?}, {left}:{right}");
        robot.drive(left, right)?;

        Ok(())
    };
    robot.subscribe(event::TrackSensor, on_track)?;

    on_track(robot.track_sensor()?)?;

    loop {
        std::thread::park()
    }
}
