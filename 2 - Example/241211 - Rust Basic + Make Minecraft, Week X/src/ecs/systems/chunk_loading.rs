use crate::chunk::{BlockID, ChunkColumn};
use crate::chunk_manager::ChunkManager;
use crate::physics::Interpolator;
use crate::player::PlayerPhysicsState;
use crate::types::TexturePack;
use bit_vec::BitVec;
use itertools::Itertools;
use noise::{NoiseFn, SuperSimplex};
use num_traits::abs;
use parking_lot::RwLock;
use specs::{Join, Read, ReadStorage, System, Write};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;

pub struct ChunkLoading {
    noise_fn: SuperSimplex,
    chunk_column_pool: Vec<Arc<ChunkColumn>>,
    loaded_columns: Vec<(i32, i32)>,
    loaded_chunks: Vec<(i32, i32, i32)>,
    chunks_to_load: VecDeque<(i32, i32, i32)>,
    chunk_at_player: (i32, i32, i32),

    send_chunk_column: Sender<(i32, i32, Arc<ChunkColumn>)>,
    receive_chunk_column: Receiver<(i32, i32, Arc<ChunkColumn>)>,

    send_chunks: Sender<(i32, i32, i32)>,
    receive_chunks: Receiver<(i32, i32, i32)>,
    pool: rayon::ThreadPool,
}

impl ChunkLoading {
    pub fn new() -> Self {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        Self {
            noise_fn: SuperSimplex::new(42),
            chunk_column_pool: Vec::new(),
            loaded_columns: Vec::new(),
            loaded_chunks: Vec::new(),
            chunks_to_load: VecDeque::new(),
            chunk_at_player: (-100, -100, -100),
            send_chunk_column: tx1,
            receive_chunk_column: rx1,
            send_chunks: tx2,
            receive_chunks: rx2,
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .unwrap(),
        }
    }

    fn allocate_chunk_column(&mut self) -> Arc<ChunkColumn> {
        match self.chunk_column_pool.pop() {
            Some(column) => {
                for chunk in column.chunks.iter() {
                    chunk.reset();
                }

                column
            }
            None => Arc::new(ChunkColumn::new()),
        }
    }

    fn flood_fill_2d(x: i32, z: i32, distance: i32) -> Vec<(i32, i32)> {
        assert!(distance >= 0);

        let matrix_width = 2 * distance + 1;
        let mut is_visited = BitVec::from_elem((matrix_width * matrix_width) as usize, false);

        let center = (x, z);
        let coords_to_index = move |x: i32, z: i32| {
            (matrix_width * (x - center.0 + distance) + (z - center.1 + distance)) as usize
        };

        let mut visited_chunks = Vec::new();
        let mut queue = VecDeque::new();

        is_visited.set(coords_to_index(x, z), true);
        queue.push_back((x, z, distance));

        while !queue.is_empty() {
            let (x, z, dist) = queue.pop_front().unwrap();

            visited_chunks.push((x, z));

            if dist <= 0 {
                continue;
            }

            if !is_visited[coords_to_index(x + 1, z)] {
                queue.push_back((x + 1, z, dist - 1));
            }
            if !is_visited[coords_to_index(x - 1, z)] {
                queue.push_back((x - 1, z, dist - 1));
            }
            if !is_visited[coords_to_index(x, z + 1)] {
                queue.push_back((x, z + 1, dist - 1));
            }
            if !is_visited[coords_to_index(x, z - 1)] {
                queue.push_back((x, z - 1, dist - 1));
            }
        }

        visited_chunks
    }

    fn flood_fill_3d(x: i32, y: i32, z: i32, distance: i32) -> Vec<(i32, i32, i32)> {
        assert!(distance >= 0);

        let matrix_width = 2 * distance + 1;
        let mut is_visited =
            BitVec::from_elem((matrix_width * matrix_width * matrix_width) as usize, false);

        let center = (x, y, z);
        let coords_to_index = move |x: i32, y: i32, z: i32| {
            (matrix_width * matrix_width * (x - center.0 + distance)
                + matrix_width * (y - center.1 + distance)
                + (z - center.2 + distance)) as usize
        };

        let mut visited_chunks = Vec::new();
        let mut queue = VecDeque::new();

        is_visited.set(coords_to_index(x, y, z), true);
        queue.reserve(100);
        queue.push_back((x, y, z, distance));

        while !queue.is_empty() {
            let (x, y, z, dist) = queue.pop_front().unwrap();

            if y >= 0 && y < 16 {
                visited_chunks.push((x, y, z));
            }

            if dist <= 0 {
                continue;
            }

            if !is_visited[coords_to_index(x + 1, y, z)] {
                queue.push_back((x + 1, y, z, dist - 1));
            }
            if !is_visited[coords_to_index(x - 1, y, z)] {
                queue.push_back((x - 1, y, z, dist - 1));
            }
            if !is_visited[coords_to_index(x, y, z + 1)] {
                queue.push_back((x, y, z + 1, dist - 1));
            }
            if !is_visited[coords_to_index(x, y, z - 1)] {
                queue.push_back((x, y, z - 1, dist - 1));
            }
            if !is_visited[coords_to_index(x, y + 1, z)] {
                queue.push_back((x, y + 1, z, dist - 1));
            }
            if !is_visited[coords_to_index(x, y - 1, z)] {
                queue.push_back((x, y - 1, z, dist - 1));
            }
        }

        visited_chunks
    }
}

const RENDER_DISTANCE: i32 = 5;

impl<'a> System<'a> for ChunkLoading {
    type SystemData = (
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
        Write<'a, Arc<RwLock<ChunkManager>>>,
        Read<'a, TexturePack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_physics_state, chunk_manager, texture_pack) = data;

        for player_physics_state in player_physics_state.join() {
            let state = player_physics_state.get_latest_state();
            let (chunk_x, chunk_y, chunk_z, _, _, _) = ChunkManager::get_chunk_coords(
                state.position.x as i32,
                state.position.y as i32,
                state.position.z as i32,
            );
            let chunk_xyz = (chunk_x, chunk_y, chunk_z);

            // Execute this system every time a player travels to another chunk
            if chunk_xyz != self.chunk_at_player {
                let previous_chunk_at_player = self.chunk_at_player;
                self.chunk_at_player = chunk_xyz;

                // Flood fill for columns and chunks
                let now = Instant::now();
                let visited_columns = Self::flood_fill_2d(chunk_x, chunk_z, RENDER_DISTANCE + 2);
                println!(
                    "Flood fill columns\t{:#?}",
                    Instant::now().duration_since(now)
                );

                let now = Instant::now();
                let visited_chunks =
                    Self::flood_fill_3d(chunk_x, chunk_y, chunk_z, RENDER_DISTANCE);
                println!(
                    "Flood fill chunks\t{:#?}",
                    Instant::now().duration_since(now)
                );

                // Unload old chunks
                let old_chunks = self.loaded_chunks.iter().filter(|(x, y, z)| {
                    abs(x - self.chunk_at_player.0)
                        + abs(y - self.chunk_at_player.1)
                        + abs(z - self.chunk_at_player.2)
                        > RENDER_DISTANCE
                });

                for &(x, y, z) in old_chunks {
                    if let Some(column) = chunk_manager.read().get_column(x, z) {
                        let chunk = column.get_chunk(y);

                        chunk.unload_from_gpu();
                        *chunk.is_rendered.write() = false;
                    }
                }

                // Remove old chunk columns
                let old_columns = self
                    .loaded_columns
                    .iter()
                    .filter(|(x, z)| {
                        abs(x - self.chunk_at_player.0) + abs(z - self.chunk_at_player.2)
                            > RENDER_DISTANCE + 2
                    })
                    .collect_vec();

                let now = Instant::now();

                for column in old_columns {
                    if let Some(column) = chunk_manager.read().remove_chunk_column(column) {
                        self.chunk_column_pool.push(column);
                    }
                }

                println!(
                    "Removing old columns\t{:#?}",
                    Instant::now().duration_since(now)
                );

                // Generate terrain
                let now = Instant::now();
                let new_chunks = visited_columns.iter().filter(|(x, z)| {
                    abs(x - previous_chunk_at_player.0) + abs(z - previous_chunk_at_player.2)
                        > RENDER_DISTANCE + 2
                });

                let mut vec = Vec::new();

                for &(x, z) in new_chunks {
                    vec.push((x, z, self.allocate_chunk_column()));
                }

                println!("Terrain gen\t{:#?}", Instant::now().duration_since(now));

                // Insert new chunk columns
                let noise_fn = self.noise_fn.clone();
                let tx = self.send_chunk_column.clone();

                self.pool.spawn_fifo(move || {
                    for (x, z, column) in vec {
                        for block_x in 0..16 {
                            for block_z in 0..16 {
                                let x = 16 * x;
                                let z = 16 * z;

                                // Scale the input for the noise function
                                let (func_x, func_z) = (
                                    (x + block_x as i32) as f64 / 64.0,
                                    (z + block_z as i32) as f64 / 64.0,
                                );
                                let y = noise_fn.get([func_x, func_z]);
                                let y = (16.0 * (y + 10.0)) as u32;

                                // Ground layers
                                column.set_block(block_x, y, block_z, BlockID::GrassBlock);
                                column.set_block(block_x, y - 1, block_z, BlockID::Dirt);
                                column.set_block(block_x, y - 2, block_z, BlockID::Dirt);
                                column.set_block(block_x, y - 3, block_z, BlockID::Dirt);
                                column.set_block(block_x, y - 4, block_z, BlockID::Dirt);
                                column.set_block(block_x, y - 5, block_z, BlockID::Dirt);

                                for y in 1..y - 5 {
                                    column.set_block(block_x, y, block_z, BlockID::Stone);
                                }

                                column.set_block(block_x, 0, block_z, BlockID::Bedrock);

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

                        _ = tx.send((x, z, column));
                    }
                });

                let now = Instant::now();

                for (x, z, column) in self.receive_chunk_column.try_iter() {
                    chunk_manager.read().add_chunk_column((x, z), column);
                }

                println!("Adding column {:?}", Instant::now().duration_since(now));

                // Add new chunks to the loading queue
                let new_chunks = visited_chunks.iter().filter(|c| {
                    abs(c.0 - previous_chunk_at_player.0)
                        + abs(c.1 - previous_chunk_at_player.1)
                        + abs(c.2 - previous_chunk_at_player.2)
                        > RENDER_DISTANCE
                });
                self.chunks_to_load.extend(new_chunks);

                self.loaded_columns = visited_columns;
                self.loaded_chunks = visited_chunks;
            }
        }

        for &(chunk_x, chunk_y, chunk_z) in self.chunks_to_load.iter() {
            let chunk_manager = Arc::clone(&chunk_manager);
            let tx = self.send_chunks.clone();

            self.pool.spawn_fifo(move || {
                let chunk_manager = chunk_manager.read();

                if let Some(column) = chunk_manager.get_column(chunk_x, chunk_z) {
                    let chunk = column.get_chunk(chunk_y);

                    if *chunk.number_of_blocks.read() == 0 {
                        return;
                    }

                    chunk_manager.update_all_blocks(chunk_x, chunk_y, chunk_z);
                    _ = tx.send((chunk_x, chunk_y, chunk_z));
                }
            });
        }

        self.chunks_to_load.clear();

        if let Ok((chunk_x, chunk_y, chunk_z)) = self.receive_chunks.try_recv() {
            let now = Instant::now();
            let chunk_manager = chunk_manager.read();

            if let Some(column) = chunk_manager.get_column(chunk_x, chunk_z) {
                let chunk = column.get_chunk(chunk_y);

                chunk.upload_to_gpu(&texture_pack);
                *chunk.is_rendered.write() = true;
            }

            println!(
                "Uploading to GPU\t{:#?}",
                Instant::now().duration_since(now)
            );
        }
    }
}
