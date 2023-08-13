use roblib_client::{roblib::roland::Roland, transports::tcp::Tcp, Robot};
use std::{error, sync::Arc};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub speed: f64,

    pub robot: Arc<Robot<Tcp>>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(robot: Arc<Robot<Tcp>>) -> Self {
        Self {
            running: true,
            speed: 25.,
            robot,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.robot_stop();
        self.running = false;
    }

    pub fn ch_speed(&mut self, v: f64) {
        self.speed += v;
    }

    pub fn drive(&mut self, lmod: f64, rmod: f64) {
        self.robot
            .drive(self.speed * lmod, self.speed * rmod)
            .unwrap();
    }
    pub fn robot_stop(&mut self) {
        self.robot.stop().unwrap();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.quit();
    }
}
