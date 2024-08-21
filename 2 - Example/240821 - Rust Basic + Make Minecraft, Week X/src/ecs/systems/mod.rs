pub mod fps_counter;
pub mod hand;
pub mod input;
pub mod inventory;
pub mod physics;
pub mod player;

use crate::timer::Timer;
use specs::{System, Write};

pub use fps_counter::*;
pub use hand::*;
pub use input::*;
pub use inventory::*;
pub use physics::*;
pub use player::*;

pub struct AdvanceGlobalTime;

impl<'a> System<'a> for AdvanceGlobalTime {
    type SystemData = Write<'a, Timer>;

    fn run(&mut self, mut global_timer: Self::SystemData) {
        global_timer.tick();
    }
}
