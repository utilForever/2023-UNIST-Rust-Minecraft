use crate::types::{UVCoords, UVFaces};

#[derive(Copy, Clone)]
pub enum BlockFaces<T> {
    All(T),
    Sides {
        sides: T,
        top: T,
        bottom: T,
    },
    Each {
        top: T,
        bottom: T,
        front: T,
        back: T,
        left: T,
        right: T,
    },
}

pub fn get_uv_every_side(faces: BlockFaces<UVCoords>) -> UVFaces {
    match faces {
        BlockFaces::All(uv) => (uv, uv, uv, uv, uv, uv),
        BlockFaces::Sides { sides, top, bottom } => (sides, sides, top, bottom, sides, sides),
        BlockFaces::Each {
            top,
            bottom,
            front,
            back,
            left,
            right,
        } => (front, back, top, bottom, left, right),
    }
}
