use crate::chunk::{BlockID, ChunkColumn};
use crate::chunk_manager::ChunkManager;
use crate::constants::{RENDER_DISTANCE, WORLD_GENERATION_THREAD_POOL_SIZE};
use crate::physics::Interpolator;
use crate::player::PlayerPhysicsState;
use crate::types::TexturePack;
use bit_vec::BitVec;
// use itertools::Itertools;
use noise::{NoiseFn, SuperSimplex};
use num_traits::abs;
use parking_lot::RwLock;
use specs::{Join, Read, ReadStorage, System, Write};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
// use std::time::Instant;
// use std::sync::atomic::AtomicBool;

pub struct ChunkLoading {
    noise_fn: SuperSimplex,
    chunk_column_pool: Arc<RwLock<Vec<Arc<ChunkColumn>>>>,
    loaded_columns: Vec<(i32, i32)>,
    _removed_columns: Vec<(i32, i32)>,
    loaded_chunks: Arc<RwLock<Vec<(i32, i32, i32)>>>,
    _chunks_to_load: VecDeque<(i32, i32, i32)>,
    chunk_at_player: (i32, i32, i32),

    _send_chunk_column: Sender<(i32, i32, Arc<ChunkColumn>)>,
    _receive_chunk_column: Receiver<(i32, i32, Arc<ChunkColumn>)>,

    send_chunks: Sender<(i32, i32, i32)>,
    receive_chunks: Receiver<(i32, i32, i32)>,
    expand_ring: Arc<RwLock<bool>>,
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
            _removed_columns: Vec::new(),
            loaded_chunks: Arc::new(RwLock::new(Vec::new())),
            _chunks_to_load: VecDeque::new(),
            chunk_at_player: (-100, -100, -100),
            _send_chunk_column: tx1,
            _receive_chunk_column: rx1,
            send_chunks: tx2,
            receive_chunks: rx2,
            expand_ring: Arc::new(RwLock::new(true)),
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

    fn flood_fill_columns(
        chunk_manager: &ChunkManager,
        x: i32,
        z: i32,
        distance: i32,
    ) -> Vec<(i32, i32)> {
        assert!(distance >= 2);

        let matrix_width = 2 * distance + 1;
        let mut is_visited = BitVec::from_elem((matrix_width * matrix_width) as usize, false);

        let center = (x, z);
        let matrix_index = move |x: i32, z: i32| {
            (matrix_width * (x - center.0 + distance) + (z - center.1 + distance)) as usize
        };
        let is_position_valid = |coord_x: i32, coord_z: i32| {
            abs(x - coord_x) <= distance && abs(z - coord_z) <= distance
        };

        let mut queue = VecDeque::new();
        let mut ring = Vec::new();
        let mut ring_number = 0;

        queue.push_back((x, z));
        ring.push((x, z));
        is_visited.set(matrix_index(x, z), true);

        while !queue.is_empty() {
            // Expand the ring
            for (coord_x, coord_z) in queue.drain(..) {
                for &(coord_x, coord_z) in &[
                    (coord_x + 1, coord_z),
                    (coord_x - 1, coord_z),
                    (coord_x, coord_z + 1),
                    (coord_x, coord_z - 1),
                ] {
                    if is_position_valid(coord_x, coord_z)
                        && !is_visited[matrix_index(coord_x, coord_z)]
                    {
                        ring.push((coord_x, coord_z));
                        is_visited.set(matrix_index(coord_x, coord_z), true);
                    }
                }
            }

            // We must expand at least 2 rings before returning sometinh
            ring_number += 1;

            if ring_number < 2 {
                queue.extend(ring.iter());
                continue;
            }

            let mut unloaded_columns = Vec::new();

            for column in ring.iter() {
                if !chunk_manager
                    .loaded_chunk_columns
                    .read()
                    .contains_key(column)
                {
                    unloaded_columns.push(*column);
                }
            }

            if !unloaded_columns.is_empty() {
                return unloaded_columns;
            } else {
                queue.extend(ring.iter());
                ring.clear();
            }
        }

        Vec::new()
    }

    fn flood_fill_chunks(
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
        let is_position_valid = |coord_x: i32, coord_y: i32, coord_z: i32| {
            abs(x - coord_x) <= distance
                && abs(y - coord_y) <= distance
                && abs(z - coord_z) <= distance
                && coord_y >= 0
                && coord_y < 16
        };

        let mut queue = VecDeque::new();
        let mut ring = Vec::new();

        queue.push_back((x, y, z));
        ring.push((x, y, z));
        is_visited.set(coords_to_index(x, y, z), true);

        // Load the first tile
        if let Some(chunk) = chunk_manager.get_chunk(x, y, z) {
            if !*chunk.is_rendered.read() {
                println!("FIRST TILE");
                return ring;
            }
        }

        while !queue.is_empty() {
            for (x, y, z) in queue.drain(..) {
                for &(x, y, z) in &[
                    (x + 1, y, z),
                    (x - 1, y, z),
                    (x, y, z + 1),
                    (x, y, z - 1),
                    (x, y + 1, z),
                    (x, y - 1, z),
                ] {
                    if is_position_valid(x, y, z) && !is_visited[coords_to_index(x, y, z)] {
                        ring.push((x, y, z));
                        is_visited.set(coords_to_index(x, y, z), true);
                    }
                }
            }

            let mut unloaded_chunks = Vec::new();

            for &(x, y, z) in ring.iter() {
                if !*chunk_manager.get_chunk(x, y, z).unwrap().is_rendered.read() {
                    unloaded_chunks.push((x, y, z));
                }
            }

            if !unloaded_chunks.is_empty() {
                return unloaded_chunks;
            } else {
                queue.extend(ring.iter());
                ring.clear();
            }
        }

        Vec::new()
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
            // if chunk_xyz != self.chunk_at_player {

            // Make this a constant
            let chunk_uploads_per_frame = 2;

            for (chunk_x, chunk_y, chunk_z) in
                self.receive_chunks.try_iter().take(chunk_uploads_per_frame)
            {
                if let Some(chunk) = chunk_manager.get_chunk(chunk_x, chunk_y, chunk_z) {
                    chunk.upload_to_gpu(&texture_pack);
                    *chunk.is_rendered.write() = true;
                }
            }

            if *self.expand_ring.read() {
                *self.expand_ring.write() = false;

                // let previous_chunk_at_player = self.chunk_at_player;
                self.chunk_at_player = chunk_xyz;

                // Unload old chunks
                // let loaded_chunks = self.loaded_chunks.read().clone();
                // let old_chunks = loaded_chunks.iter().filter(|(x, y, z)| {
                //     abs(x - self.chunk_at_player.0) > RENDER_DISTANCE
                //         || abs(y - self.chunk_at_player.1) > RENDER_DISTANCE
                //         || abs(z - self.chunk_at_player.2) > RENDER_DISTANCE
                // });
                //
                // for &(x, y, z) in old_chunks {
                //     if let Some(chunk) = chunk_manager.get_chunk(x, y, z) {
                //         chunk.unload_from_gpu();
                //         *chunk.is_rendered.write() = false;
                //     }
                // }
                //
                // // Remove old chunk columns
                // let old_columns = self
                //     .loaded_columns
                //     .iter()
                //     .filter(|(x, z)| {
                //         abs(x - self.chunk_at_player.0) > RENDER_DISTANCE + 2
                //             || abs(z - self.chunk_at_player.2) > RENDER_DISTANCE + 2
                //     })
                //     .cloned()
                //     .collect_vec();
                //
                // let now = Instant::now();
                //
                // for column in old_columns {
                //     self.removed_columns.push(column);
                // }
                //
                // println!(
                //     "Removing old columns\t{:#?}",
                //     Instant::now().duration_since(now)
                // );

                // let now = Instant::now();
                let visited_columns =
                    Self::flood_fill_columns(&chunk_manager, chunk_x, chunk_z, RENDER_DISTANCE + 2);

                // println!(
                //     "Flood fill columns\t{:#?} {} {:?}",
                //     Instant::now().duration_since(now),
                //     visited_columns.len(),
                //     (chunk_x, chunk_z)
                // );

                let column_pool = Arc::clone(&self.chunk_column_pool);
                // let new_columns = visited_columns.iter().filter(|(x, z)| {
                //     abs(x - previous_chunk_at_player.0) > RENDER_DISTANCE + 2
                //         || abs(z - previous_chunk_at_player.2) > RENDER_DISTANCE + 2
                // });
                let new_columns = &visited_columns;

                // let now = Instant::now();
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

                // println!("Terrain gen\t{:#?}", Instant::now().duration_since(now));

                self.loaded_columns = visited_columns;

                let noise_fn = self.noise_fn.clone();
                // let send_column = self.send_chunk_column.clone();
                let send_chunks = self.send_chunks.clone();
                let cm = Arc::clone(&chunk_manager);
                let loaded_chunks = Arc::clone(&self.loaded_chunks);
                let expand_ring = Arc::clone(&self.expand_ring);

                self.pool.spawn(move || {
                    // Terrain gen
                    let chunk_manager = Arc::clone(&cm);

                    rayon::scope(move |s| {
                        for (x, z, column) in vec {
                            let chunk_manager = Arc::clone(&chunk_manager);
                            // let send_column = send_column.clone();

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

                                // _ = send_column.send((x, z, column));
                                chunk_manager.add_chunk_column((x, z), column);
                            });
                        }
                    });

                    let chunk_manager = Arc::clone(&cm);

                    // Chunk face culling & AO
                    rayon::scope(move |s| {
                        // let now = Instant::now();
                        let visited_chunks = Self::flood_fill_chunks(
                            &chunk_manager,
                            chunk_x,
                            chunk_y,
                            chunk_z,
                            RENDER_DISTANCE,
                        );

                        // println!(
                        //     "Flood fill chunks\t{:#?} {}",
                        //     Instant::now().duration_since(now),
                        //     visited_chunks.len()
                        // );

                        loaded_chunks.write().extend(visited_chunks.iter());

                        // let new_chunks = visited_chunks.iter().filter(|c| {
                        //     abs(c.0 - previous_chunk_at_player.0) > RENDER_DISTANCE
                        //         || abs(c.1 - previous_chunk_at_player.1) > RENDER_DISTANCE
                        //         || abs(c.2 - previous_chunk_at_player.2) > RENDER_DISTANCE
                        // });
                        let new_chunks = &visited_chunks;

                        for &(chunk_x, chunk_y, chunk_z) in new_chunks {
                            let chunk_manager = Arc::clone(&chunk_manager);
                            let send_chunks = send_chunks.clone();

                            s.spawn(move |_| {
                                if let Some(chunk) =
                                    chunk_manager.get_chunk(chunk_x, chunk_y, chunk_z)
                                {
                                    *chunk.is_rendered.write() = true;

                                    if chunk.is_empty() {
                                        return;
                                    }

                                    chunk_manager.update_all_blocks(chunk_x, chunk_y, chunk_z);
                                    _ = send_chunks.send((chunk_x, chunk_y, chunk_z));
                                }
                            });
                        }
                    });

                    *expand_ring.write() = true;
                });
            }
        }

        // Fix dashmap synchronization behaviour
        // let mut to_remove = Vec::new();
        //
        // self.removed_columns.retain(|column| {
        //     if let Some(column) = chunk_manager.remove_chunk_column(&column) {
        //         to_remove.push(column);
        //         false
        //     } else {
        //         true
        //     }
        // });
        // self.chunk_column_pool.write().extend(to_remove.into_iter());

        // for (x, z, column) in self.receive_chunk_column.try_iter() {
        //     chunk_manager.add_chunk_column((x, z), column);
        // }
    }
}
