pub mod input;
pub mod physics;
pub mod player;

use crate::timer::Timer;
use specs::{System, Write};

pub use input::*;
pub use physics::*;
pub use player::*;

pub struct AdvanceGlobalTime;

impl<'a> System<'a> for AdvanceGlobalTime {
    type SystemData = Write<'a, Timer>;

    fn run(&mut self, mut global_timer: Self::SystemData) {
        global_timer.tick();
    }
}
