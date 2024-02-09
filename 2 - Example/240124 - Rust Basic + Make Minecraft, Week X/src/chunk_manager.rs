use crate::chunk::{BlockID, Chunk};
use crate::shader::ShaderProgram;
use crate::util::UVCoordinate;
use nalgebra::Matrix4;
use nalgebra_glm::vec3;
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Default)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl ChunkManager {
    pub fn preload_some_chunks(&mut self) {
        for y in -1..=1 {
            for z in -1..=1 {
                for x in -1..=1 {
                    self.loaded_chunks.insert(
                        (x, y, z),
                        Chunk::full_of_block(if (x + y + z) % 2 == 0 {
                            BlockID::COBBLESTONE
                        } else {
                            BlockID::DIRT
                        }),
                    );
                }
            }
        }
    }

    fn get_chunk_and_block_coords(x: i32, y: i32, z: i32) -> (i32, i32, i32, u32, u32, u32) {
        let chunk_x = if x < 0 { (x + 1) / 16 - 1 } else { x / 16 };
        let chunk_y = if y < 0 { (y + 1) / 16 - 1 } else { y / 16 };
        let chunk_z = if z < 0 { (z + 1) / 16 - 1 } else { z / 16 };

        let block_x = x.rem_euclid(16) as u32;
        let block_y = y.rem_euclid(16) as u32;
        let block_z = z.rem_euclid(16) as u32;

        (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z)
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<BlockID> {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_and_block_coords(x, y, z);

        self.loaded_chunks
            .get((chunk_x, chunk_y, chunk_z).borrow())
            .map(|chunk| chunk.get_block(block_x, block_y, block_z))
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, block: BlockID) {
        let (chunk_x, chunk_y, chunk_z, block_x, block_y, block_z) =
            ChunkManager::get_chunk_and_block_coords(x, y, z);

        if let Some(chunk) = self
            .loaded_chunks
            .get_mut((chunk_x, chunk_y, chunk_z).borrow())
        {
            chunk.set_block(block_x, block_y, block_z, block);
        }
    }

    pub fn rebuild_dirty_chunks(&mut self, uv_map: &HashMap<BlockID, UVCoordinate>) {
        for chunk in self.loaded_chunks.values_mut() {
            if chunk.dirty {
                chunk.regenerate_vbo(uv_map);
            }
        }
    }

    pub fn render_loaded_chunks(&mut self, program: &mut ShaderProgram) {
        for ((x, y, z), chunk) in &self.loaded_chunks {
            let model_matrix = {
                let translate_matrix =
                    Matrix4::new_translation(&vec3(*x as f32, *y as f32, *z as f32).scale(16.0));
                let rotate_matrix = Matrix4::from_euler_angles(0.0f32, 0.0, 0.0);
                let scale_matrix = Matrix4::new_nonuniform_scaling(&vec3(1.0f32, 1.0f32, 1.0f32));

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
