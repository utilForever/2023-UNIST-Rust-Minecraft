use crate::chunk::{BlockID, ChunkColumn};
use crate::chunk_manager::ChunkManager;
use crate::constants::{RENDER_DISTANCE, WORLD_GENERATION_THREAD_POOL_SIZE};
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
    chunk_column_pool: Arc<RwLock<Vec<Arc<ChunkColumn>>>>,
    loaded_columns: Vec<(i32, i32)>,
    removed_columns: Vec<(i32, i32)>,
    loaded_chunks: Vec<(i32, i32, i32)>,
    _chunks_to_load: VecDeque<(i32, i32, i32)>,
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
            chunk_column_pool: Arc::new(RwLock::new(Vec::new())),
            loaded_columns: Vec::new(),
            removed_columns: Vec::new(),
            loaded_chunks: Vec::new(),
            _chunks_to_load: VecDeque::new(),
            chunk_at_player: (-100, -100, -100),
            send_chunk_column: tx1,
            receive_chunk_column: rx1,
            send_chunks: tx2,
            receive_chunks: rx2,
            pool: rayon::ThreadPoolBuilder::new()
                .stack_size(4 * 1024 * 1024)
                .num_threads(*WORLD_GENERATION_THREAD_POOL_SIZE)
                .build()
                .unwrap(),
        }
    }

    // #[inline]
    // fn allocate_chunk_column(&mut self) -> Arc<ChunkColumn> {
    //     match self.chunk_column_pool.pop() {
    //         Some(column) => {
    //             for chunk in column.chunks.iter() {
    //                 chunk.reset();
    //             }
    //
    //             column
    //         }
    //         None => Arc::new(ChunkColumn::new()),
    //     }
    // }

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
                is_visited.set(coords_to_index(x + 1, z), true);
            }
            if !is_visited[coords_to_index(x - 1, z)] {
                queue.push_back((x - 1, z, dist - 1));
                is_visited.set(coords_to_index(x - 1, z), true);
            }
            if !is_visited[coords_to_index(x, z + 1)] {
                queue.push_back((x, z + 1, dist - 1));
                is_visited.set(coords_to_index(x, z + 1), true);
            }
            if !is_visited[coords_to_index(x, z - 1)] {
                queue.push_back((x, z - 1, dist - 1));
                is_visited.set(coords_to_index(x, z - 1), true);
            }
        }

        visited_chunks
    }

    fn flood_fill_3d(
        chunk_manager: &ChunkManager,
        x: i32,
        y: i32,
        z: i32,
        distance: i32,
    ) -> Vec<(i32, i32, i32)> {
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

                if let Some(chunk) = chunk_manager.get_chunk(x, y, z) {
                    if chunk.is_fully_opaque() {
                        continue;
                    }
                }
            }

            if dist <= 0 {
                continue;
            }

            if !is_visited[coords_to_index(x + 1, y, z)] {
                queue.push_back((x + 1, y, z, dist - 1));
                is_visited.set(coords_to_index(x + 1, y, z), true);
            }
            if !is_visited[coords_to_index(x - 1, y, z)] {
                queue.push_back((x - 1, y, z, dist - 1));
                is_visited.set(coords_to_index(x - 1, y, z), true);
            }
            if !is_visited[coords_to_index(x, y, z + 1)] {
                queue.push_back((x, y, z + 1, dist - 1));
                is_visited.set(coords_to_index(x, y, z + 1), true);
            }
            if !is_visited[coords_to_index(x, y, z - 1)] {
                queue.push_back((x, y, z - 1, dist - 1));
                is_visited.set(coords_to_index(x, y, z - 1), true);
            }
            if !is_visited[coords_to_index(x, y + 1, z)] {
                queue.push_back((x, y + 1, z, dist - 1));
                is_visited.set(coords_to_index(x, y + 1, z), true);
            }
            if !is_visited[coords_to_index(x, y - 1, z)] {
                queue.push_back((x, y - 1, z, dist - 1));
                is_visited.set(coords_to_index(x, y - 1, z), true);
            }
        }

        visited_chunks
    }
}

impl<'a> System<'a> for ChunkLoading {
    type SystemData = (
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
        Write<'a, Arc<ChunkManager>>,
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

                // Unload old chunks
                let old_chunks = self.loaded_chunks.iter().filter(|(x, y, z)| {
                    abs(x - self.chunk_at_player.0)
                        + abs(y - self.chunk_at_player.1)
                        + abs(z - self.chunk_at_player.2)
                        > RENDER_DISTANCE
                });

                for &(x, y, z) in old_chunks {
                    if let Some(chunk) = chunk_manager.get_chunk(x, y, z) {
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
                    .cloned()
                    .collect_vec();

                let now = Instant::now();

                for column in old_columns {
                    self.removed_columns.push(column);
                }

                println!(
                    "Removing old columns\t{:#?}",
                    Instant::now().duration_since(now)
                );

                let noise_fn = self.noise_fn.clone();
                let send_column = self.send_chunk_column.clone();
                let send_chunks = self.send_chunks.clone();
                let column_pool = Arc::clone(&self.chunk_column_pool);
                let chunk_manager = Arc::clone(&chunk_manager);

                let now = Instant::now();
                let visited_columns = Self::flood_fill_2d(chunk_x, chunk_z, RENDER_DISTANCE + 2);

                println!(
                    "Flood fill columns\t{:#?} {} {:?}",
                    Instant::now().duration_since(now),
                    visited_columns.len(),
                    (chunk_x, chunk_z)
                );

                let new_columns = visited_columns.iter().filter(|(x, z)| {
                    abs(x - previous_chunk_at_player.0) + abs(z - previous_chunk_at_player.2)
                        > RENDER_DISTANCE + 2
                });
                let now = Instant::now();
                let mut vec = Vec::new();

                for &(x, z) in new_columns {
                    vec.push((x, z, {
                        let mut guard = column_pool.write();

                        match guard.pop() {
                            Some(column) => {
                                for chunk in column.chunks.iter() {
                                    chunk.reset();
                                }

                                column
                            }
                            None => Arc::new(ChunkColumn::new()),
                        }
                    }));
                }

                println!("Terrain gen\t{:#?}", Instant::now().duration_since(now));

                let now = Instant::now();
                let visited_chunks =
                    Self::flood_fill_3d(&chunk_manager, chunk_x, chunk_y, chunk_z, RENDER_DISTANCE);

                println!(
                    "Flood fill chunks\t{:#?} {}",
                    Instant::now().duration_since(now),
                    visited_chunks.len()
                );

                self.loaded_columns = visited_columns;
                self.loaded_chunks = visited_chunks.clone();

                self.pool.spawn(move || {
                    // Terrain gen
                    rayon::scope(move |s| {
                        for (x, z, column) in vec {
                            let send_column = send_column.clone();

                            s.spawn(move |_| {
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
                                    }
                                }

                                _ = send_column.send((x, z, column));
                            });
                        }
                    });

                    // Chunk face culling & AO
                    rayon::scope(move |s| {
                        let new_chunks = visited_chunks.iter().filter(|c| {
                            abs(c.0 - previous_chunk_at_player.0)
                                + abs(c.1 - previous_chunk_at_player.1)
                                + abs(c.2 - previous_chunk_at_player.2)
                                > RENDER_DISTANCE
                        });

                        for &(chunk_x, chunk_y, chunk_z) in new_chunks {
                            let chunk_manager = Arc::clone(&chunk_manager);
                            let send_chunks = send_chunks.clone();

                            s.spawn(move |_| {
                                if let Some(chunk) =
                                    chunk_manager.get_chunk(chunk_x, chunk_y, chunk_z)
                                {
                                    if chunk.is_empty() {
                                        return;
                                    }

                                    chunk_manager.update_all_blocks(chunk_x, chunk_y, chunk_z);
                                    _ = send_chunks.send((chunk_x, chunk_y, chunk_z));
                                }
                            });
                        }
                    });
                });
            }
        }

        // Fix dashmap synchronization behaviour
        let mut to_remove = Vec::new();

        self.removed_columns.retain(|column| {
            if let Some(column) = chunk_manager.remove_chunk_column(&column) {
                to_remove.push(column);
                false
            } else {
                true
            }
        });
        self.chunk_column_pool.write().extend(to_remove.into_iter());

        for (x, z, column) in self.receive_chunk_column.try_iter() {
            chunk_manager.add_chunk_column((x, z), column);
        }

        if let Ok((chunk_x, chunk_y, chunk_z)) = self.receive_chunks.try_recv() {
            if let Some(chunk) = chunk_manager.get_chunk(chunk_x, chunk_y, chunk_z) {
                chunk.upload_to_gpu(&texture_pack);
                *chunk.is_rendered.write() = true;
            }
        }
    }
}
