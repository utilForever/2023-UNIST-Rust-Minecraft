use crate::gl_call;
use crate::shapes::unit_cube_array;
use crate::util::UVCoordinate;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::random;
use std::collections::HashMap;
use std::os::raw::c_void;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum BlockID {
    AIR,
    DIRT,
    COBBLESTONE,
    OBSIDIAN,
}

impl Distribution<BlockID> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> BlockID {
        match rng.gen_range(0..4) {
            0 => BlockID::AIR,
            1 => BlockID::DIRT,
            2 => BlockID::COBBLESTONE,
            3 => BlockID::OBSIDIAN,
            _ => BlockID::AIR,
        }
    }
}

const CHUNK_SIZE: u32 = 16;
const CHUNK_VOLUME: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

fn create_vao_vbo() -> (u32, u32) {
    let mut vao = 0;
    gl_call!(gl::CreateVertexArrays(1, &mut vao));

    // Position
    gl_call!(gl::EnableVertexArrayAttrib(vao, 0));
    gl_call!(gl::VertexArrayAttribFormat(
        vao,
        0,
        3_i32,
        gl::FLOAT,
        gl::FALSE,
        0
    ));
    gl_call!(gl::VertexArrayAttribBinding(vao, 0, 0));

    // Texture coords
    gl_call!(gl::EnableVertexArrayAttrib(vao, 1));
    gl_call!(gl::VertexArrayAttribFormat(
        vao,
        1,
        2_i32,
        gl::FLOAT,
        gl::FALSE,
        3 * std::mem::size_of::<f32>() as u32
    ));
    gl_call!(gl::VertexArrayAttribBinding(vao, 1, 0));

    let mut vbo = 0;
    gl_call!(gl::CreateBuffers(1, &mut vbo));
    gl_call!(gl::NamedBufferData(
        vbo,
        (180 * CHUNK_VOLUME as usize * std::mem::size_of::<f32>()) as isize,
        std::ptr::null(),
        gl::DYNAMIC_DRAW
    ));

    gl_call!(gl::VertexArrayVertexBuffer(
        vao,
        0,
        vbo,
        0,
        (5 * std::mem::size_of::<f32>()) as i32
    ));

    (vao, vbo)
}

pub struct Chunk {
    blocks: [BlockID; CHUNK_VOLUME as usize],
    pub vao: u32,
    vbo: u32,
    pub vertices_drawn: u32,
    pub dirty: bool,
}

impl Chunk {
    pub fn empty() -> Chunk {
        let (vao, vbo) = create_vao_vbo();

        Chunk {
            blocks: [BlockID::AIR; CHUNK_VOLUME as usize],
            vao,
            vbo,
            vertices_drawn: 0,
            dirty: false,
        }
    }

    pub fn full_of_block(block: BlockID) -> Chunk {
        let (vao, vbo) = create_vao_vbo();

        Chunk {
            blocks: [block; CHUNK_VOLUME as usize],
            vao,
            vbo,
            vertices_drawn: 0,
            dirty: true,
        }
    }

    pub fn random() -> Chunk {
        let (vao, vbo) = create_vao_vbo();

        let mut chunk = Chunk {
            blocks: [BlockID::AIR; CHUNK_VOLUME as usize],
            vao,
            vbo,
            vertices_drawn: 0,
            dirty: true,
        };

        for i in 0..chunk.blocks.len() {
            chunk.blocks[i] = random::<BlockID>();
        }

        chunk
    }

    #[inline]
    fn coords_to_index(x: u32, y: u32, z: u32) -> usize {
        (y * (CHUNK_SIZE * CHUNK_SIZE) + z * CHUNK_SIZE + x) as usize
    }

    #[inline]
    pub fn get_block(&self, x: u32, y: u32, z: u32) -> BlockID {
        self.blocks[Chunk::coords_to_index(x, y, z)]
    }

    #[inline]
    pub fn set_block(&mut self, x: u32, y: u32, z: u32, block: BlockID) {
        self.blocks[Chunk::coords_to_index(x, y, z)] = block;
        self.dirty = true;
    }

    pub fn regenerate_vbo(&mut self, uv_map: &HashMap<BlockID, UVCoordinate>) {
        let mut idx = 0;
        self.vertices_drawn = 0;

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = self.get_block(x, y, z);

                    if block == BlockID::AIR {
                        continue;
                    }

                    let (uv_bl, ub_tr) = *uv_map.get(&block).unwrap();
                    let cube_array = unit_cube_array(
                        (x as f32, y as f32, z as f32),
                        uv_bl,
                        ub_tr,
                        [true, true, true, true, true, true],
                    );

                    gl_call!(gl::NamedBufferSubData(
                        self.vbo,
                        (idx * std::mem::size_of::<f32>()) as isize,
                        (cube_array.len() * std::mem::size_of::<f32>()) as isize,
                        cube_array.as_ptr() as *mut c_void,
                    ));

                    self.vertices_drawn += cube_array.len() as u32 / 5;
                    idx += cube_array.len();
                }
            }
        }

        self.dirty = false;
    }
}
