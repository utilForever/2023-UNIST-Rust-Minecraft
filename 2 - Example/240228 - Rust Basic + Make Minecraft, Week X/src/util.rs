use nalgebra_glm::{vec3, Vec3};

pub fn forward(rotation: &Vec3) -> Vec3 {
    vec3(
        rotation.x.cos() * rotation.y.cos(),
        rotation.x.sin(),
        rotation.x.cos() * rotation.y.sin(),
    )
}
