use crate::aabb::AABB;
use crate::physics::Interpolator;
use crate::player::{PlayerPhysicsState, PlayerState};
use nalgebra_glm::Vec3;
use specs::{Component, DenseVecStorage, NullStorage};

#[derive(Component, Debug)]
pub struct Position(Vec3);

#[derive(Component, Debug)]
pub struct Velocity(Vec3);

#[derive(Component, Debug)]
pub struct Acceleration(Vec3);

#[derive(Component, Debug)]
pub struct BoundingBox(AABB);

#[derive(Component, Default, Debug)]
#[storage(NullStorage)]
pub struct Player;

impl Component for Interpolator<PlayerPhysicsState> {
    type Storage = DenseVecStorage<Self>;
}

impl Component for PlayerState {
    type Storage = DenseVecStorage<Self>;
}
