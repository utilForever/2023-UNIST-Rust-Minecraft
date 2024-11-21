use crate::chunk::{BlockID, BlockIterator, ChunkColumn};
use crate::chunk_manager::ChunkManager;
use crate::physics::Interpolator;
use crate::player::PlayerPhysicsState;
use crate::types::TexturePack;
use noise::{NoiseFn, SuperSimplex};
use specs::{Join, Read, ReadStorage, System, Write};
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

pub struct ChunkLoading {
    ss: SuperSimplex,
    loaded_columns: HashSet<(i32, i32)>,
    loaded_chunks: HashSet<(i32, i32, i32)>,
    chunks_to_load: VecDeque<(i32, i32, i32)>,
    chunk_at_player: (i32, i32, i32),
}

impl ChunkLoading {
    pub fn new() -> Self {
        Self {
            ss: SuperSimplex::new(42),
            loaded_columns: HashSet::new(),
            loaded_chunks: HashSet::new(),
            chunks_to_load: VecDeque::new(),
            chunk_at_player: (-1, -1, -1),
        }
    }

    fn flood_fill_2d(x: i32, z: i32, distance: i32) -> HashSet<(i32, i32)> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((x, z, distance));

        while !queue.is_empty() {
            let (x, z, dist) = queue.pop_front().unwrap();

            visited.insert((x, z));

            if dist <= 0 {
                continue;
            }

            if !visited.contains(&(x + 1, z)) {
                queue.push_back((x + 1, z, dist - 1));
            }
            if !visited.contains(&(x - 1, z)) {
                queue.push_back((x - 1, z, dist - 1));
            }
            if !visited.contains(&(x, z + 1)) {
                queue.push_back((x, z + 1, dist - 1));
            }
            if !visited.contains(&(x, z - 1)) {
                queue.push_back((x, z - 1, dist - 1));
            }
        }

        visited
    }

    fn flood_fill_3d(x: i32, y: i32, z: i32, distance: i32) -> HashSet<(i32, i32, i32)> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((x, y, z, distance));

        while !queue.is_empty() {
            let (x, y, z, dist) = queue.pop_front().unwrap();

            if y < 0 || y > 15 {
                continue;
            }

            visited.insert((x, y, z));

            if dist <= 0 {
                continue;
            }

            if !visited.contains(&(x + 1, y, z)) {
                queue.push_back((x + 1, y, z, dist - 1));
            }
            if !visited.contains(&(x - 1, y, z)) {
                queue.push_back((x - 1, y, z, dist - 1));
            }
            if !visited.contains(&(x, y, z + 1)) {
                queue.push_back((x, y, z + 1, dist - 1));
            }
            if !visited.contains(&(x, y, z - 1)) {
                queue.push_back((x, y, z - 1, dist - 1));
            }
            if y != 15 && !visited.contains(&(x, y + 1, z)) {
                queue.push_back((x, y + 1, z, dist - 1));
            }
            if y != 0 && !visited.contains(&(x, y - 1, z)) {
                queue.push_back((x, y - 1, z, dist - 1));
            }
        }

        visited
    }
}

const RENDER_DISTANCE: i32 = 5;

impl<'a> System<'a> for ChunkLoading {
    type SystemData = (
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
        Write<'a, ChunkManager>,
        Read<'a, TexturePack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_physics_state, mut chunk_manager, texture_pack) = data;

        for player_physics_state in player_physics_state.join() {
            let state = player_physics_state.get_latest_state();
            let chunk_x = state.position.x as i32 / 16;
            let chunk_y = state.position.y as i32 / 16;
            let chunk_z = state.position.z as i32 / 16;
            let chunk_xyz = (chunk_x, chunk_y, chunk_z);

            if chunk_xyz != self.chunk_at_player {
                let now = Instant::now();
                self.chunk_at_player = chunk_xyz;

                let visited = Self::flood_fill_2d(chunk_x, chunk_z, RENDER_DISTANCE + 2);
                println!("Flood fill: {:#?}", Instant::now().duration_since(now));

                let old_columns = self.loaded_columns.difference(&visited);

                for chunk in old_columns {
                    chunk_manager.remove_chunk(chunk);
                }

                let new_columns = visited.difference(&self.loaded_columns);
                error!("new_columns: {:?}", new_columns);

                for &(x, z) in new_columns {
                    let mut column = Box::new(ChunkColumn::new());

                    for block_x in 0..16 {
                        for block_z in 0..16 {
                            let x = 16 * x;
                            let z = 16 * z;

                            // Scale the input for the noise function
                            let (func_x, func_z) = (
                                (x + block_x as i32) as f64 / 64.0,
                                (z + block_z as i32) as f64 / 64.0,
                            );
                            let y = self.ss.get([func_x, func_z]);
                            let y = (16.0 * (y + 10.0)) as u32;

                            // Ground layers
                            column.set_block(block_x, y, block_z, BlockID::GrassBlock);
                            column.set_block(block_x, y - 1, block_z, BlockID::Dirt);
                            column.set_block(block_x, y - 2, block_z, BlockID::Dirt);
                            column.set_block(block_x, y - 3, block_z, BlockID::Dirt);
                            column.set_block(block_x, y - 4, block_z, BlockID::Dirt);
                            column.set_block(block_x, y - 5, block_z, BlockID::Dirt);

                            // Trees
                            // if random::<u32>() % 100 < 1 {
                            //     let height = 5;
                            //
                            //     for i in y + 1..y + 1 + height {
                            //         chunk_manager.set_block(x, i, z, BlockID::OakLog);
                            //     }
                            //
                            //     for yy in y + height - 2..=y + height - 1 {
                            //         for xx in x - 2..=x + 2 {
                            //             for zz in z - 2..=z + 2 {
                            //                 if xx != x || zz != z {
                            //                     chunk_manager.set_block(
                            //                         xx,
                            //                         yy,
                            //                         zz,
                            //                         BlockID::OakLeaves,
                            //                     );
                            //                 }
                            //             }
                            //         }
                            //     }
                            //
                            //     for xx in x - 1..=x + 1 {
                            //         for zz in z - 1..=z + 1 {
                            //             if xx != x || zz != z {
                            //                 chunk_manager.set_block(
                            //                     xx,
                            //                     y + height,
                            //                     zz,
                            //                     BlockID::OakLeaves,
                            //                 );
                            //             }
                            //         }
                            //     }
                            //
                            //     chunk_manager.set_block(x, y + height + 1, z, BlockID::OakLeaves);
                            //     chunk_manager.set_block(
                            //         x + 1,
                            //         y + height + 1,
                            //         z,
                            //         BlockID::OakLeaves,
                            //     );
                            //     chunk_manager.set_block(
                            //         x - 1,
                            //         y + height + 1,
                            //         z,
                            //         BlockID::OakLeaves,
                            //     );
                            //     chunk_manager.set_block(
                            //         x,
                            //         y + height + 1,
                            //         z + 1,
                            //         BlockID::OakLeaves,
                            //     );
                            //     chunk_manager.set_block(
                            //         x,
                            //         y + height + 1,
                            //         z - 1,
                            //         BlockID::OakLeaves,
                            //     );
                            // }
                        }
                    }

                    chunk_manager.add_chunk_column((x, z), column);
                }

                println!("Set block: {:#?}", Instant::now().duration_since(now));

                self.loaded_columns = visited;

                let visited = Self::flood_fill_3d(chunk_x, chunk_y, chunk_z, RENDER_DISTANCE);
                let new_chunks = visited.difference(&self.loaded_chunks);

                self.chunks_to_load.extend(new_chunks);
                self.loaded_chunks = visited;

                println!("Set blocks: {:#?}", Instant::now().duration_since(now));
            }
        }

        if let Some((chunk_x, chunk_y, chunk_z)) = self.chunks_to_load.pop_front() {
            let now = Instant::now();

            for (block_x, block_y, block_z) in BlockIterator::new() {
                chunk_manager.update_block(chunk_x, chunk_y, chunk_z, block_x, block_y, block_z);
            }

            println!("Update blocks {:#?}", Instant::now().duration_since(now));

            let now = Instant::now();
            chunk_manager.upload_chunk_to_gpu(chunk_x, chunk_y, chunk_z, &texture_pack);
            println!("Upload {:#?}", Instant::now().duration_since(now));
        }
    }
}
