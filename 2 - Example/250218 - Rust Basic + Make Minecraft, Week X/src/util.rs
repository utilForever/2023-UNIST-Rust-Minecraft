use nalgebra_glm::{vec3, Vec3};

pub trait Forward {
    fn forward(&self) -> Self;
}

impl Forward for Vec3 {
    fn forward(&self) -> Self {
        vec3(
            self.x.cos() * self.y.cos(),
            self.x.sin(),
            self.x.cos() * self.y.sin(),
        )
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for Vec3 {
    fn zero() -> Self {
        vec3(0.0, 0.0, 0.0)
    }
}
