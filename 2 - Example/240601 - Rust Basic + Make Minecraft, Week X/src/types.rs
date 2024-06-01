use crate::block_texture_faces::BlockFaces;
use crate::chunk::BlockID;
use std::collections::HashMap;

pub type UVCoords = (f32, f32, f32, f32);
pub type UVFaces = (UVCoords, UVCoords, UVCoords, UVCoords, UVCoords, UVCoords);
pub type UVMap = HashMap<BlockID, BlockFaces<UVCoords>>;
