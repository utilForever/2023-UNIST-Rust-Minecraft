use crate::block_texture_faces::BlockFaces;
use crate::chunk::BlockID;
use crate::particle_system::ParticleSystem;
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
pub type TexturePack = HashMap<BlockID, BlockFaces<TextureLayer>>;
pub type ParticleSystems = HashMap<&'static str, ParticleSystem>;
