use crate::chunk_manager::ChunkManager;
use crate::constants::{BACKGROUND_COLOR, BLOCK_OUTLINE_WIDTH};
use crate::gui::{
    create_block_outline_vao, create_crosshair_vao, create_hotbar_selection_vao, create_hotbar_vao,
    draw_crosshair,
};
use crate::inventory::Inventory;
use crate::player::PlayerState;
use crate::timer::Timer;
use crate::types::{ParticleSystems, Shaders, TexturePack};
use nalgebra::Matrix4;
use nalgebra_glm::vec3;
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct RenderChunks;

impl<'a> System<'a> for RenderChunks {
    type SystemData = (
        Read<'a, TexturePack>,
        ReadStorage<'a, PlayerState>,
        Write<'a, ChunkManager>,
        Write<'a, Shaders>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (texture_pack, player_state, mut chunk_manager, mut shaders) = data;

        chunk_manager.rebuild_dirty_chunks(&texture_pack);

        let mut voxel_shader = shaders.get_mut("voxel_shader").unwrap();
        voxel_shader.use_program();
        voxel_shader.set_uniform1i("array_texture", 0);

        let (r, g, b, a) = BACKGROUND_COLOR;
        gl_call!(gl::ClearColor(r, g, b, a));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

        for player_state in (&player_state).join() {
            unsafe {
                voxel_shader.set_uniform_matrix4fv("view", player_state.view_matrix.as_ptr());
                voxel_shader
                    .set_uniform_matrix4fv("projection", player_state.projection_matrix.as_ptr());
            }
            chunk_manager.render_loaded_chunks(&mut voxel_shader);
        }

        // chunk_manager.generate_progressive_terrain();
    }
}

pub struct RenderParticles;

impl<'a> System<'a> for RenderParticles {
    type SystemData = (
        Read<'a, Timer>,
        ReadStorage<'a, PlayerState>,
        Write<'a, ChunkManager>,
        Write<'a, Shaders>,
        Write<'a, ParticleSystems>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (global_timer, player_state, chunk_manager, mut shaders, mut particle_systems) = data;

        gl_call!(gl::Disable(gl::CULL_FACE));

        let mut particle_shader = shaders.get_mut("particle_shader").unwrap();
        particle_shader.use_program();
        particle_shader.set_uniform1i("array_texture", 0);

        for player_state in (&player_state).join() {
            for particle_system in particle_systems.values_mut() {
                particle_system.update_all_particles(global_timer.time(), &chunk_manager);
                particle_system.render_all_particles(
                    &mut particle_shader,
                    &player_state.view_matrix,
                    &player_state.projection_matrix,
                );
            }
        }

        gl_call!(gl::Enable(gl::CULL_FACE));
    }
}

pub struct RenderBlockOutline {
    vao: u32,
}

impl RenderBlockOutline {
    pub fn new() -> Self {
        Self {
            vao: create_block_outline_vao(),
        }
    }
}

impl<'a> System<'a> for RenderBlockOutline {
    type SystemData = (ReadStorage<'a, PlayerState>, Write<'a, Shaders>);

    fn run(&mut self, data: Self::SystemData) {
        let (player_state, mut shaders) = data;

        for player_state in (&player_state).join() {
            if let Some(((x, y, z), _)) = player_state.targeted_block {
                let (x, y, z) = (x as f32, y as f32, z as f32);
                let model_matrix = Matrix4::new_translation(&vec3(x, y, z));

                let outline_shader = shaders.get_mut("outline_shader").unwrap();
                outline_shader.use_program();
                unsafe {
                    outline_shader.set_uniform_matrix4fv("model", model_matrix.as_ptr());
                    outline_shader.set_uniform_matrix4fv("view", player_state.view_matrix.as_ptr());
                    outline_shader.set_uniform_matrix4fv(
                        "projection",
                        player_state.projection_matrix.as_ptr(),
                    );
                }

                gl_call!(gl::LineWidth(BLOCK_OUTLINE_WIDTH));
                gl_call!(gl::BindVertexArray(self.vao));
                gl_call!(gl::DrawArrays(gl::LINES, 0, 24));
            }
        }
    }
}

pub(crate) struct RenderGUI {
    crosshair_vao: u32,
    hotbar_vao: u32,
    hotbar_selection_vao: u32,
}

impl RenderGUI {
    pub fn new() -> Self {
        Self {
            crosshair_vao: create_crosshair_vao(),
            hotbar_vao: create_hotbar_vao(),
            hotbar_selection_vao: create_hotbar_selection_vao(),
        }
    }
}

impl<'a> System<'a> for RenderGUI {
    type SystemData = (
        Read<'a, TexturePack>,
        Write<'a, Shaders>,
        WriteStorage<'a, Inventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (texture_pack, mut shaders, mut inventory) = data;

        for inventory in (&mut inventory).join() {
            let mut gui_shader = shaders.get_mut("gui_shader").unwrap();
            draw_crosshair(self.crosshair_vao, &mut gui_shader);
            gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

            gl_call!(gl::Disable(gl::DEPTH_TEST));
            inventory.update_dirty_items(&texture_pack);
            inventory.draw_hotbar(self.hotbar_vao, &mut gui_shader);
            inventory.draw_hotbar_selection_box(self.hotbar_selection_vao, &mut gui_shader);

            let mut item_shader = shaders.get_mut("item_shader").unwrap();
            inventory.draw_hotbar_items(&mut item_shader);
            gl_call!(gl::Enable(gl::DEPTH_TEST));
        }
    }
}
