use specs::System;
use std::time::Instant;

const MAXIMUM_OPTIMAL_FRAMETIME: f32 = 1.0 / 60.0;

pub struct FpsCounter {
    nb_frames: u64,
    last_frame: Instant,
    last_second: Instant,
}

impl<'a> System<'a> for FpsCounter {
    type SystemData = ();

    fn run(&mut self, _data: Self::SystemData) {
        let current_time = Instant::now();
        self.nb_frames += 1;

        {
            let frame_time = current_time.duration_since(self.last_frame).as_secs_f32();

            if frame_time > MAXIMUM_OPTIMAL_FRAMETIME {
                warn!("Suboptimal frame time: {:.2} ms", frame_time * 1000.0);
            }

            self.last_frame = current_time;
        }

        if current_time.duration_since(self.last_second).as_secs_f32() >= 1.0 {
            info!("{} FPS", self.nb_frames);

            self.nb_frames = 0;
            self.last_second = current_time;
        }
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            nb_frames: 0,
            last_frame: Instant::now(),
            last_second: Instant::now(),
        }
    }
}
