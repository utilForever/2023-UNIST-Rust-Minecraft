use crate::block_texture_faces::BlockFaces;
use crate::chunk::BlockID;
use std::collections::HashMap;

pub type TextureLayer = u32;
pub type UVFaces = (
    TextureLayer,
    TextureLayer,
    TextureLayer,
    TextureLayer,
    TextureLayer,
    TextureLayer,
);
pub type UVMap = HashMap<BlockID, BlockFaces<TextureLayer>>;
