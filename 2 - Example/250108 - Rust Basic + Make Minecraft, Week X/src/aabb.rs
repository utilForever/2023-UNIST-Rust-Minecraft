use nalgebra_glm::{vec3, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct AABB {
    pub mins: Vec3,
    pub maxs: Vec3,
}

impl AABB {
    pub fn new(mins: Vec3, maxs: Vec3) -> Self {
        Self { mins, maxs }
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.mins += translation;
        self.maxs += translation;
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.mins.x < other.maxs.x
            && self.maxs.x > other.mins.x
            && self.mins.y < other.maxs.y
            && self.maxs.y > other.mins.y
            && self.mins.z < other.maxs.z
            && self.maxs.z > other.mins.z
    }
}

pub fn get_block_aabb(mins: &Vec3) -> AABB {
    AABB::new(mins.clone(), mins + vec3(1.0, 1.0, 1.0))
}
