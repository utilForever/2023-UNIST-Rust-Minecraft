use crate::ambient_occlusion::compute_ao_of_block;
use crate::chunk::{BlockID, Chunk, ChunkColumn};
use crate::shader::ShaderProgram;
use crate::types::TexturePack;
use nalgebra::Matrix4;
use nalgebra_glm::vec3;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_VOLUME: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Default)]
pub struct ChunkManager {
    loaded_chunk_columns: HashMap<(i32, i32), Arc<RwLock<Box<ChunkColumn>>>>,
    pub block_changelist: HashSet<(i32, i32, i32)>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            loaded_chunk_columns: HashMap::new(),
            block_changelist: HashSet::new(),
        }
    }

    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<MappedRwLockReadGuard<Chunk>> {
        if y < 0 || y > 15 {
            return None;
        }

        self.loaded_chunk_columns.get(&(x, z)).map(|column| {
            parking_lot::RwLockReadGuard::map(column.read(), |column| &column.chunks[y as usize])
        })
    }

    pub fn get_chunk_mut(&self, x: i32, y: i32, z: i32) -> Option<MappedRwLockWriteGuard<Chunk>> {
        if y < 0 || y > 15 {
            return None;
        }

        self.loaded_chunk_columns.get(&(x, z)).map(|column| {
            parking_lot::RwLockWriteGuard::map(column.write(), |column| {
                &mut column.chunks[y as usize]
            })
        })
    }

    pub fn add_chunk_column(
        &mut self,
        xz: (i32, i32),
        chunk_column: Arc<RwLock<Box<ChunkColumn>>>,
    ) {
        if !self.loaded_chunk_columns.contains_key(&xz) {
            self.loaded_chunk_columns.insert(xz, chunk_column);
            // self.fresh_chunk.insert(xz);
        }
    }

    pub fn remove_chunk_column(
        &mut self,
        xz: &(i32, i32),
    ) -> Option<Arc<RwLock<Box<ChunkColumn>>>> {
        self.loaded_chunk_columns.remove(&xz)
    }

    pub fn generate_progressive_terrain(&mut self) {
        // TODO: Implement a better terrain generation algorithm
    }

    // pub fn generate_terrain(&mut self) {
    //     let ss = SuperSimplex::new(1296);
    //     let render_distance = 5;
    //
    //     for y in -render_distance..=render_distance {
    //         for z in -render_distance..=render_distance {
    //             for x in -render_distance..=render_distance {
    //                 self.add_chunk((x, y, z), Chunk::new());
    //             }
    //         }
    //     }
    //
    //     for x in -16 * render_distance..=16 * render_distance {
    //         for z in -16 * render_distance..=16 * render_distance {
    //             let (xf, zf) = (x as f64 / 64.0, z as f64 / 64.0);
    //             let y = ss.get([xf, zf]);
    //             let y = (16.0 * (y + 1.0)) as i32;
    //
    //             self.set_block(x, y, z, BlockID::GrassBlock);
    //             self.set_block(x, y - 1, z, BlockID::Dirt);
    //             self.set_block(x, y - 2, z, BlockID::Dirt);
    //             self.set_block(x, y - 3, z, BlockID::Cobblestone);
    //
    //             if random::<u32>() % 100 == 0 {
    //                 let h = 5;
    //
    //                 for i in y + 1..y + 1 + h {
    //                     self.set_block(x, i, z, BlockID::OakLog);
    //                 }
    //
    //                 for yy in y + h - 2..=y + h - 1 {
    //                     for xx in x - 2..=x + 2 {
    //                         for zz in z - 2..=z + 2 {
    //                             if xx != x || zz != z {
    //                                 self.set_block(xx, yy, zz, BlockID::OakLeaves);
    //                             }
    //                         }
    //                     }
    //                 }
    //
    //                 for xx in x - 1..=x + 1 {
    //                     for zz in z - 1..=z + 1 {
    //                         if xx != x || zz != z {
    //                             self.set_block(xx, y + h, zz, BlockID::OakLeaves);
    //                         }
    //                     }
    //                 }
    //
    //                 self.set_block(x, y + h + 1, z, BlockID::OakLeaves);
    //                 self.set_block(x + 1, y + h + 1, z, BlockID::OakLeaves);
    //                 self.set_block(x - 1, y + h + 1, z, BlockID::OakLeaves);
    //                 self.set_block(x, y + h + 1, z + 1, BlockID::OakLeaves);
    //                 self.set_block(x, y + h + 1, z - 1, BlockID::OakLeaves);
    //             }
    //         }
    //     }
    // }

    pub fn preload_some_chunks(&mut self) {
        for z in 0..2 {
            for x in 0..2 {
                self.add_chunk_column((x, z), Arc::new(RwLock::new(Box::new(ChunkColumn::new()))));
            }
        }
    }

    pub fn single_chunk(&mut self) {
        self.add_chunk_column(
            (0, 0),
            Arc::new(RwLock::new(Box::new(ChunkColumn::full_of_block(
                BlockID::Cobblestone,
            )))),
        );
    }

    // Transform global block coordinates into chunk local coordinates
    #[inline]
    pub fn get_chunk_coords(x: i32, y: i32, z: i32) -> (i32, i32, i32, u32, u32, u32) {
        let chunk_x = if x < 0 { (x + 1) / 16 - 1 } else { x / 16 };
        let chunk_y = if y < 0 { (y + 1) / 16 - 1 } else { y / 16 };
        let chunk_z = if z < 0 { (z + 1) / 16 - 1 } else { z / 16 };

        let block_x = x.rem_euclid(16) as u32;
        let block_y = y.rem_euclid(16) as u32;
        let block_z = z.rem_euclid(16) as u32;

        (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z)
    }

    // Transform chunk local coordinates into global coordinates
    pub fn get_global_coords(
        (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z): (i32, i32, i32, u32, u32, u32),
    ) -> (i32, i32, i32) {
        let x = 16 * chunk_x + block_x as i32;
        let y = 16 * chunk_y + block_y as i32;
        let z = 16 * chunk_z + block_z as i32;

        (x, y, z)
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockID> {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_coords(x, y, z);

        match self.get_chunk(chunk_x, chunk_y, chunk_z) {
            Some(chunk) => Some(chunk.get_block(block_x, block_y, block_z)),
            None => None,
        }
    }

    // Replaces the block at (x, y, z) with `block`
    // This function should be used for terrain generation because it does not modify the changelist
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockID) -> bool {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_coords(x, y, z);

        match self.get_chunk_mut(chunk_x, chunk_y, chunk_z) {
            Some(mut chunk) => {
                Some(chunk.set_block(block_x, block_y, block_z, block));
                true
            }
            None => false,
        }
    }

    // Like `set_block` but it modifies the changelist
    // Should be used when an entity (player, mob etc.) interacts with the world
    pub fn put_block(&mut self, x: i32, y: i32, z: i32, block: BlockID) {
        if self.set_block(x, y, z, block) {
            self.block_changelist.insert((x, y, z));
        }
    }

    pub fn is_solid_block_at(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(x, y, z)
            .filter(|&block| block != BlockID::Air)
            .is_some()
    }

    pub fn update_block(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
    ) {
        let array_index =
            (block_y * CHUNK_SIZE * CHUNK_SIZE + block_z * CHUNK_SIZE + block_x) as usize;
        let (world_x, world_y, world_z) =
            ChunkManager::get_global_coords((chunk_x, chunk_y, chunk_z, block_x, block_y, block_z));
        let active_faces_of_block = {
            let chunk = self.get_chunk(chunk_x, chunk_y, chunk_z).unwrap();

            if chunk.get_block(block_x, block_y, block_z) == BlockID::Air {
                return;
            }

            self.get_active_faces_of_block(world_x, world_y, world_z)
        };

        {
            let mut chunk = self.get_chunk_mut(chunk_x, chunk_y, chunk_z).unwrap();
            chunk
                .active_faces
                .set(6 * array_index, active_faces_of_block[0]);
            chunk
                .active_faces
                .set(6 * array_index + 1, active_faces_of_block[1]);
            chunk
                .active_faces
                .set(6 * array_index + 2, active_faces_of_block[2]);
            chunk
                .active_faces
                .set(6 * array_index + 3, active_faces_of_block[3]);
            chunk
                .active_faces
                .set(6 * array_index + 4, active_faces_of_block[4]);
            chunk
                .active_faces
                .set(6 * array_index + 5, active_faces_of_block[5]);
        }

        // Ambient Occlusion
        let block_ao = compute_ao_of_block(&|rx: i32, ry: i32, rz: i32| {
            self.get_block(world_x + rx, world_y + ry, world_z + rz)
                .filter(|b| !b.is_transparent_no_leaves())
                .is_some()
        });

        let mut chunk = self.get_chunk_mut(chunk_x, chunk_y, chunk_z).unwrap();
        chunk.ao_vertices[array_index] = block_ao;
    }

    pub fn rebuild_dirty_chunks(&mut self, uv_map: &TexturePack) {
        let mut changelist_per_chunk: HashMap<(i32, i32, i32), Vec<(u32, u32, u32)>> =
            HashMap::new();

        for &change in self.block_changelist.iter() {
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
                            ChunkManager::get_chunk_coords(
                                change.0 + x,
                                change.1 + y,
                                change.2 + z,
                            );
                        changelist_per_chunk
                            .entry((chunk_x, chunk_y, chunk_z))
                            .or_default()
                            .push((block_x, block_y, block_z));
                    }
                }
            }
        }

        self.block_changelist.clear();

        // for &(chunk_x, chunk_y, chunk_z) in self.fresh_chunk.clone().iter() {
        //     for (block_x, block_y, block_z) in BlockIterator::new() {
        //         self.update_block(chunk_x, chunk_y, chunk_z, block_x, block_y, block_z);
        //     }
        //
        //     self.update_chunk(chunk_x, chunk_y, chunk_z, &uv_map);
        // }
        //
        // self.fresh_chunk.clear();

        for (&(chunk_x, chunk_y, chunk_z), dirty_blocks) in changelist_per_chunk.iter() {
            if let None = self.get_chunk(chunk_x, chunk_y, chunk_z) {
                continue;
            }

            for &(block_x, block_y, block_z) in dirty_blocks {
                self.update_block(chunk_x, chunk_y, chunk_z, block_x, block_y, block_z);
            }

            self.get_chunk_mut(chunk_x, chunk_y, chunk_z)
                .unwrap()
                .upload_to_gpu(&uv_map);
        }
    }

    // An active face is a block face next to a transparent block that needs to be rendered
    pub fn get_active_faces_of_block(&self, x: i32, y: i32, z: i32) -> [bool; 6] {
        let right = self
            .get_block(x + 1, y, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let left = self
            .get_block(x - 1, y, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let top = self
            .get_block(x, y + 1, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let bottom = self
            .get_block(x, y - 1, z)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let front = self
            .get_block(x, y, z + 1)
            .filter(|&b| !b.is_transparent())
            .is_none();
        let back = self
            .get_block(x, y, z - 1)
            .filter(|&b| !b.is_transparent())
            .is_none();

        [right, left, top, bottom, front, back]
    }

    pub fn render_loaded_chunks(&mut self, program: &mut ShaderProgram) {
        for ((x, z), chunk_column) in self.loaded_chunk_columns.iter() {
            for (ref y, chunk) in chunk_column.read().chunks.iter().enumerate() {
                // Skip rendering the chunk if there is nothing to draw
                if !chunk.is_rendered || chunk.vertices_drawn == 0 {
                    continue;
                }

                let model_matrix = {
                    let translate_matrix = Matrix4::new_translation(
                        &vec3(*x as f32, *y as f32, *z as f32).scale(16.0),
                    );
                    let rotate_matrix = Matrix4::from_euler_angles(0.0f32, 0.0, 0.0);
                    let scale_matrix =
                        Matrix4::new_nonuniform_scaling(&vec3(1.0f32, 1.0f32, 1.0f32));

                    translate_matrix * rotate_matrix * scale_matrix
                };

                gl_call!(gl::BindVertexArray(chunk.vao));
                unsafe {
                    program.set_uniform_matrix4fv("model", model_matrix.as_ptr());
                }
                gl_call!(gl::DrawArrays(
                    gl::TRIANGLES,
                    0,
                    chunk.vertices_drawn as i32
                ));
            }
        }
    }
}
