use crate::ambient_occlusion::compute_ao_of_block;
use crate::chunk::{BlockID, BlockIterator, Chunk, ChunkColumn};
use crate::shader::ShaderProgram;
use crate::types::TexturePack;
use nalgebra::Matrix4;
use nalgebra_glm::vec3;
use owning_ref::OwningRef;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_VOLUME: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Default)]
pub struct ChunkManager {
    pub loaded_chunk_columns: RwLock<HashMap<(i32, i32), Arc<ChunkColumn>>>,
    block_changelist: RwLock<HashSet<(BlockID, i32, i32, i32)>>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            loaded_chunk_columns: RwLock::new(HashMap::new()),
            block_changelist: RwLock::new(HashSet::new()),
        }
    }

    #[inline]
    pub fn get_column(&self, x: i32, z: i32) -> Option<Arc<ChunkColumn>> {
        self.loaded_chunk_columns
            .read()
            .get(&(x, z))
            .map(|col| Arc::clone(col))
    }

    #[inline]
    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<OwningRef<Arc<ChunkColumn>, Chunk>> {
        if y < 0 || y >= 16 {
            return None;
        }

        self.loaded_chunk_columns
            .read()
            .get(&(x, z))
            .map(|column| OwningRef::new(Arc::clone(column)).map(|column| column.get_chunk(y)))
    }

    #[inline]
    pub fn add_chunk_column(&self, xz: (i32, i32), chunk_column: Arc<ChunkColumn>) {
        let mut guard = self.loaded_chunk_columns.write();

        if !guard.contains_key(&xz) {
            guard.insert(xz, chunk_column);
            // self.fresh_chunk.insert(xz);
        }
    }

    #[inline]
    pub fn remove_chunk_column(&self, xz: &(i32, i32)) -> Option<Arc<ChunkColumn>> {
        self.loaded_chunk_columns.write().remove(&xz)
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
                self.add_chunk_column((x, z), Arc::new(ChunkColumn::new()));
            }
        }
    }

    pub fn single_chunk(&mut self) {
        self.add_chunk_column(
            (0, 0),
            Arc::new(ChunkColumn::full_of_block(BlockID::Cobblestone)),
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

        self.get_chunk(chunk_x, chunk_y, chunk_z)
            .map(|chunk| chunk.get_block(block_x, block_y, block_z))
    }

    // Replaces the block at (x, y, z) with `block`
    // This function should be used for terrain generation because it does not modify the changelist
    pub fn set_block(&self, x: i32, y: i32, z: i32, block: BlockID) -> bool {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_coords(x, y, z);

        match self.get_chunk(chunk_x, chunk_y, chunk_z) {
            Some(chunk) => {
                chunk.set_block(block_x, block_y, block_z, block);
                true
            }
            None => false,
        }
    }

    // Like `set_block` but it modifies the changelist
    // Should be used when an entity (player, mob etc.) interacts with the world
    pub fn put_block(&self, x: i32, y: i32, z: i32, block: BlockID) {
        if self.set_block(x, y, z, block) {
            self.block_changelist.write().insert((block, x, y, z));
        }
    }

    pub fn is_solid_block_at(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(x, y, z)
            .filter(|&block| block != BlockID::Air)
            .is_some()
    }

    pub fn update_all_blocks(&self, chunk_x: i32, chunk_y: i32, chunk_z: i32) {
        let this_column = match self.get_column(chunk_x, chunk_z) {
            Some(column) => column,
            None => {
                error!(
                    "Cannot update chunk {:?} because its column doesn't exist",
                    (chunk_x, chunk_y, chunk_z)
                );
                return;
            }
        };
        let this_chunk = this_column.get_chunk(chunk_y);

        if this_chunk.is_empty() {
            return;
        }

        let mut neighbourhood = [None, None, None, None, None, None, None, None, None];

        for x in -1..=1 {
            for z in -1..=1 {
                neighbourhood[3 * (x + 1) as usize + (z + 1) as usize] = if x == 0 && z == 0 {
                    None
                } else {
                    self.get_column(chunk_x + x, chunk_z + z)
                };
            }
        }

        #[inline]
        fn block_at(
            column: &ChunkColumn,
            neighbourhood: &[Option<Arc<ChunkColumn>>; 9],
            chunk_x: i32,
            chunk_z: i32,
            world_x: i32,
            world_y: i32,
            world_z: i32,
        ) -> BlockID {
            let to_index = |x: i32, z: i32| -> usize {
                3 * (x - chunk_x + 1) as usize + (z - chunk_z + 1) as usize
            };
            let (chunk_xn, chunk_yn, chunk_zn, block_x, block_y, block_z) =
                ChunkManager::get_chunk_coords(world_x, world_y, world_z);

            if chunk_yn < 0 || chunk_yn >= 16 {
                return BlockID::Air;
            }

            if chunk_x == chunk_xn && chunk_z == chunk_zn {
                column
                    .get_chunk(chunk_yn)
                    .get_block(block_x, block_y, block_z)
            } else {
                if let Some(neighbour_column) = neighbourhood[to_index(chunk_xn, chunk_zn)].as_ref()
                {
                    neighbour_column
                        .get_chunk(chunk_yn)
                        .get_block(block_x, block_y, block_z)
                } else {
                    BlockID::Air
                }
            }
        }

        #[inline]
        fn compute_active_faces(
            column: &ChunkColumn,
            neighbourhood: &[Option<Arc<ChunkColumn>>; 9],
            chunk_x: i32,
            chunk_z: i32,
            x: i32,
            y: i32,
            z: i32,
        ) -> [bool; 6] {
            let right =
                block_at(&column, &neighbourhood, chunk_x, chunk_z, x + 1, y, z).is_transparent();
            let left =
                block_at(&column, &neighbourhood, chunk_x, chunk_z, x - 1, y, z).is_transparent();
            let top =
                block_at(&column, &neighbourhood, chunk_x, chunk_z, x, y + 1, z).is_transparent();
            let bottom =
                block_at(&column, &neighbourhood, chunk_x, chunk_z, x, y - 1, z).is_transparent();
            let front =
                block_at(&column, &neighbourhood, chunk_x, chunk_z, x, y, z + 1).is_transparent();
            let back =
                block_at(&column, &neighbourhood, chunk_x, chunk_z, x, y, z - 1).is_transparent();

            [right, left, top, bottom, front, back]
        }

        let mut active_faces = this_chunk.active_faces.write();
        let mut ao_vertices = this_chunk.ao_vertices.write();

        for (block_x, block_y, block_z) in BlockIterator::new() {
            if this_chunk.get_block(block_x, block_y, block_z) == BlockID::Air {
                continue;
            }

            let (world_x, world_y, world_z) = ChunkManager::get_global_coords((
                chunk_x, chunk_y, chunk_z, block_x, block_y, block_z,
            ));
            let af = compute_active_faces(
                &this_column,
                &neighbourhood,
                chunk_x,
                chunk_z,
                world_x,
                world_y,
                world_z,
            );
            let array_index =
                (block_y * CHUNK_SIZE * CHUNK_SIZE + block_z * CHUNK_SIZE + block_x) as usize;

            active_faces.set(6 * array_index, af[0]);
            active_faces.set(6 * array_index + 1, af[1]);
            active_faces.set(6 * array_index + 2, af[2]);
            active_faces.set(6 * array_index + 3, af[3]);
            active_faces.set(6 * array_index + 4, af[4]);
            active_faces.set(6 * array_index + 5, af[5]);

            // Ambient Occlusion
            let block_ao = compute_ao_of_block(&|rx: i32, ry: i32, rz: i32| {
                !block_at(
                    &this_column,
                    &neighbourhood,
                    chunk_x,
                    chunk_z,
                    world_x + rx,
                    world_y + ry,
                    world_z + rz,
                )
                .is_transparent_no_leaves()
            });

            ao_vertices[array_index] = block_ao;
        }
    }

    pub fn update_block(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
    ) {
        let chunk = self.get_chunk(chunk_x, chunk_y, chunk_z).unwrap();

        if chunk.get_block(block_x, block_y, block_z) == BlockID::Air {
            return;
        }

        let (world_x, world_y, world_z) =
            ChunkManager::get_global_coords((chunk_x, chunk_y, chunk_z, block_x, block_y, block_z));
        let array_index =
            (block_y * CHUNK_SIZE * CHUNK_SIZE + block_z * CHUNK_SIZE + block_x) as usize;
        let active_faces_of_block = self.get_active_faces_of_block(world_x, world_y, world_z);

        {
            let mut active_faces = chunk.active_faces.write();

            active_faces.set(6 * array_index, active_faces_of_block[0]);
            active_faces.set(6 * array_index + 1, active_faces_of_block[1]);
            active_faces.set(6 * array_index + 2, active_faces_of_block[2]);
            active_faces.set(6 * array_index + 3, active_faces_of_block[3]);
            active_faces.set(6 * array_index + 4, active_faces_of_block[4]);
            active_faces.set(6 * array_index + 5, active_faces_of_block[5]);
        }

        // Ambient Occlusion
        let block_ao = compute_ao_of_block(&|rx: i32, ry: i32, rz: i32| {
            self.get_block(world_x + rx, world_y + ry, world_z + rz)
                .filter(|b| !b.is_transparent_no_leaves())
                .is_some()
        });

        self.get_chunk(chunk_x, chunk_y, chunk_z)
            .unwrap()
            .ao_vertices
            .write()[array_index] = block_ao;
    }

    pub fn rebuild_dirty_chunks(&self, uv_map: &TexturePack) {
        let mut changelist_per_chunk: HashMap<(i32, i32, i32), Vec<(u32, u32, u32)>> =
            HashMap::new();

        for &change in self.block_changelist.read().iter() {
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
                            ChunkManager::get_chunk_coords(
                                change.1 + x,
                                change.2 + y,
                                change.3 + z,
                            );
                        changelist_per_chunk
                            .entry((chunk_x, chunk_y, chunk_z))
                            .or_default()
                            .push((block_x, block_y, block_z));
                    }
                }
            }
        }

        self.block_changelist.write().clear();

        for (&(chunk_x, chunk_y, chunk_z), dirty_blocks) in changelist_per_chunk.iter() {
            match self.get_chunk(chunk_x, chunk_y, chunk_z) {
                Some(chunk) => {
                    for &(block_x, block_y, block_z) in dirty_blocks.iter() {
                        self.update_block(chunk_x, chunk_y, chunk_z, block_x, block_y, block_z);
                    }

                    chunk.upload_to_gpu(&uv_map);
                }
                None => continue,
            }
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

    pub fn render_loaded_chunks(&self, program: &mut ShaderProgram) {
        for ((x, z), chunk_column) in self.loaded_chunk_columns.read().iter() {
            for (ref y, chunk) in chunk_column.chunks.iter().enumerate() {
                // Skip rendering the chunk if there is nothing to draw
                if !*chunk.is_uploaded_to_gpu.read() || chunk.is_empty() {
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

                gl_call!(gl::BindVertexArray(*chunk.vao.read()));
                unsafe {
                    program.set_uniform_matrix4fv("model", model_matrix.as_ptr());
                }
                gl_call!(gl::DrawArrays(
                    gl::TRIANGLES,
                    0,
                    *chunk.vertices_drawn.read() as i32
                ));
            }
        }
    }
}
