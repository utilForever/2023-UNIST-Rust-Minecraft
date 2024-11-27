#[macro_use]
extern crate lazy_static;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate specs;

#[macro_use]
pub mod debugging;
pub mod aabb;
pub mod ambient_occlusion;
pub mod block_texture_faces;
pub mod chunk;
pub mod chunk_manager;
pub mod constants;
pub mod ecs;
pub mod gui;
pub mod input;
pub mod inventory;
pub mod main_hand;
pub mod particle_system;
pub mod physics;
pub mod player;
pub mod raycast;
pub mod renderer;
pub mod shader;
pub mod shapes;
pub mod texture;
pub mod texture_pack;
pub mod timer;
pub mod types;
pub mod util;
pub mod window;

use crate::chunk_manager::ChunkManager;
use crate::debugging::*;
use crate::physics::Interpolator;
use crate::shader::ShaderProgram;
use std::collections::HashMap;
// use glfw::ffi::glfwSwapInterval;
use crate::constants::*;
use crate::ecs::components::*;
use crate::ecs::systems::chunk_loading::ChunkLoading;
use crate::ecs::systems::*;
use crate::gui::{create_gui_icons_texture, create_widgets_texture};
use crate::input::InputCache;
use crate::inventory::Inventory;
use crate::main_hand::MainHand;
use crate::particle_system::ParticleSystem;
use crate::player::{PlayerPhysicsState, PlayerState};
use crate::texture_pack::generate_array_texture;
use crate::timer::Timer;
use crate::types::Shaders;
use crate::window::create_window;
use ecs::systems::fps_counter::FpsCounter;
use nalgebra_glm::vec3;
use specs::{Builder, DispatcherBuilder, World, WorldExt};
use std::os::raw::c_void;

fn main() {
    let mut log_builder = pretty_env_logger::formatted_builder();
    log_builder.parse_filters(LOG_LEVEL.as_str()).init();

    let mut world = World::new();
    world.register::<PlayerState>();
    world.register::<Interpolator<PlayerPhysicsState>>();
    world.register::<Inventory>();
    world.register::<MainHand>();
    world.register::<MainHandItemChanged>();

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local({
            let (glfw, window, events) = create_window(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_NAME);

            gl_call!(gl::Enable(gl::DEBUG_OUTPUT));
            gl_call!(gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS));
            gl_call!(gl::DebugMessageCallback(
                Some(debug_message_callback),
                std::ptr::null::<c_void>(),
            ));
            gl_call!(gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                std::ptr::null::<u32>(),
                gl::TRUE
            ));

            gl_call!(gl::Enable(gl::CULL_FACE));
            gl_call!(gl::CullFace(gl::BACK));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
            gl_call!(gl::Enable(gl::BLEND));
            gl_call!(gl::Viewport(
                0,
                0,
                WINDOW_WIDTH as i32,
                WINDOW_HEIGHT as i32
            ));

            ReadWindowEvents {
                glfw,
                window,
                events,
            }
        })
        .with_thread_local(InventoryHandleInput)
        .with_thread_local(HandlePlayerInput)
        .with_thread_local(UpdatePlayerState)
        .with_thread_local(PlaceAndBreakBlocks)
        .with_thread_local(UpdatePlayerPhysics)
        .with_thread_local(UpdateMainHand)
        .with_thread_local(ChunkLoading::new())
        .with_thread_local(RenderChunks)
        .with_thread_local(RenderParticles)
        .with_thread_local(RenderBlockOutline::new())
        .with_thread_local(RenderMainHand::new())
        .with_thread_local(RenderGUI::new())
        .with_thread_local(AdvanceGlobalTime)
        .with_thread_local(FpsCounter::new())
        .build();

    world.insert(InputCache::default());
    world.insert(Timer::default());
    world.insert({
        let (item_array_texture, texture_pack) = generate_array_texture();
        gl_call!(gl::BindTextureUnit(0, item_array_texture));

        texture_pack
    });
    world.insert({
        let mut particle_systems = HashMap::new();
        particle_systems.insert("block_particles", ParticleSystem::new(500));

        particle_systems
    });
    world.insert({
        let mut shaders_resource = Shaders::new();
        shaders_resource.insert(
            "voxel_shader",
            ShaderProgram::compile("src/shaders/voxel.vert", "src/shaders/voxel.frag"),
        );
        shaders_resource.insert(
            "gui_shader",
            ShaderProgram::compile("src/shaders/gui.vert", "src/shaders/gui.frag"),
        );
        shaders_resource.insert(
            "outline_shader",
            ShaderProgram::compile("src/shaders/outline.vert", "src/shaders/outline.frag"),
        );
        shaders_resource.insert(
            "item_shader",
            ShaderProgram::compile("src/shaders/item.vert", "src/shaders/item.frag"),
        );
        shaders_resource.insert(
            "particle_shader",
            ShaderProgram::compile("src/shaders/particle.vert", "src/shaders/particle.frag"),
        );
        shaders_resource.insert(
            "hand_shader",
            ShaderProgram::compile("src/shaders/hand.vert", "src/shaders/hand.frag"),
        );

        shaders_resource
    });
    world.insert(ChunkManager::new());

    {
        let gui_icons_texture = create_gui_icons_texture();
        gl_call!(gl::ActiveTexture(gl::TEXTURE1));
        gl_call!(gl::BindTexture(gl::TEXTURE_2D, gui_icons_texture));

        let gui_widgets_texture = create_widgets_texture();
        gl_call!(gl::ActiveTexture(gl::TEXTURE2));
        gl_call!(gl::BindTexture(gl::TEXTURE_2D, gui_widgets_texture));
    }

    let _player = world
        .create_entity()
        .with(PlayerState::new())
        .with(Interpolator::new(
            1.0 / PHYSICS_TICKRATE,
            PlayerPhysicsState::new_at_position(vec3(8.0, 180.0, 8.0)),
        ))
        .with(Inventory::new())
        .with(MainHand::new())
        .with(MainHandItemChanged)
        .build();

    // Loop until the user closes the window
    loop {
        dispatcher.dispatch(&world);
    }
}
