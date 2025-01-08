use crate::constants::{FAR_PLANE, NEAR_PLANE, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::ecs::components::MainHandItemChanged;
use crate::inventory::Inventory;
use crate::main_hand::MainHand;
use crate::physics::Interpolator;
use crate::player::{PlayerPhysicsState, PlayerState};
use crate::timer::Timer;
use crate::types::{Shaders, TexturePack};
use crate::util::Forward;
use nalgebra::{Matrix4, Vector3};
use nalgebra_glm::vec3;
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};
use std::time::Instant;

pub struct UpdateMainHand;

impl<'a> System<'a> for UpdateMainHand {
    type SystemData = (
        WriteStorage<'a, MainHandItemChanged>,
        ReadStorage<'a, Inventory>,
        WriteStorage<'a, MainHand>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut main_hand_item_changed, inventory, mut main_hand) = data;

        for (_, inventory, main_hand) in
            (&main_hand_item_changed, &inventory, &mut main_hand).join()
        {
            main_hand.switch_item_to(inventory.get_selected_item());
        }

        main_hand_item_changed.clear();
    }
}

pub struct RenderMainHand {
    pub y_velocity: f32,
    pub y_offset: Interpolator<f32>,
}

impl RenderMainHand {
    pub fn new() -> Self {
        Self {
            y_velocity: 0.0,
            y_offset: Interpolator::new(1.0 / 30.0, 0.0),
        }
    }
}

impl<'a> System<'a> for RenderMainHand {
    type SystemData = (
        WriteStorage<'a, MainHand>,
        ReadStorage<'a, PlayerState>,
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
        Read<'a, TexturePack>,
        Read<'a, Timer>,
        Write<'a, Shaders>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut main_hand,
            player_state,
            player_physics_state,
            texture_pack,
            global_timer,
            mut shaders,
        ) = data;

        for (player_state, player_physics_state, main_hand) in
            (&player_state, &player_physics_state, &mut main_hand).join()
        {
            if main_hand.begin_switch {
                main_hand.begin_switch = false;

                if self.y_velocity.is_sign_positive() {
                    self.y_velocity = -8.0;
                }
            }

            self.y_offset
                .interpolate_hand(global_timer.time(), self.y_velocity);

            let y_offset_latest = self.y_offset.get_latest_state_mut();

            if *y_offset_latest < -1.2 {
                *y_offset_latest = -1.2;
                self.y_velocity *= -1.0;

                main_hand.set_showing_item(main_hand.switching_to);
            }

            if *y_offset_latest > 0.0 {
                *y_offset_latest = 0.0;
                self.y_velocity = 0.0;
            }

            if main_hand.showing_item.is_none() {
                return;
            }

            let view_matrix = {
                let player_physics_state = player_physics_state.get_interpolated_state();
                let camera_position = player_physics_state.position
                    + vec3(
                        0.0,
                        *player_state.camera_height.get_interpolated_state(),
                        0.0,
                    );
                let looking_dir = player_state.rotation.forward();

                nalgebra_glm::look_at(
                    &camera_position,
                    &(camera_position + looking_dir),
                    &Vector3::y(),
                )
            };

            main_hand.update_if_dirty(&texture_pack);

            let player_pos = player_physics_state.get_interpolated_state().position;
            let camera_height = *player_state.camera_height.get_interpolated_state();
            let camera_pos = player_pos + vec3(0.0, camera_height, 0.0);

            let forward = &player_state.rotation.forward().normalize();
            let right = forward.cross(&Vector3::y()).normalize();
            let up = right.cross(&forward).normalize();

            let model_matrix = {
                let translate_matrix1 = Matrix4::new_translation(
                    &(vec3(camera_pos.x, camera_pos.y, camera_pos.z)
                        + up * -1.2
                        + up * *self.y_offset.get_interpolated_state()),
                );
                let translate_matrix2 = Matrix4::new_translation(&(vec3(2.0, 0.0, 0.0)));

                let rotate_matrix =
                    nalgebra_glm::rotation(-player_state.rotation.y, &vec3(0.0, 1.0, 0.0));
                let rotate_matrix =
                    nalgebra_glm::rotation(player_state.rotation.x, &right) * rotate_matrix;
                let rotate_matrix =
                    nalgebra_glm::rotation(-35.0f32.to_radians(), &up) * rotate_matrix;

                translate_matrix1 * rotate_matrix * translate_matrix2
            };

            let projection_matrix = {
                let fov = 70.0f32.to_radians();
                nalgebra_glm::perspective(
                    WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                    fov,
                    NEAR_PLANE,
                    FAR_PLANE,
                )
            };

            let hand_shader = shaders.get_mut("hand_shader").unwrap();
            hand_shader.use_program();
            unsafe {
                hand_shader.set_uniform_matrix4fv("model", model_matrix.as_ptr());
                hand_shader.set_uniform_matrix4fv("view", view_matrix.as_ptr());
                hand_shader.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
                hand_shader.set_uniform1i("array_texture", 0);
            }

            gl_call!(gl::BindVertexArray(main_hand.render.vao));

            gl_call!(gl::Disable(gl::DEPTH_TEST));
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 36));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
        }
    }
}

impl Interpolator<f32> {
    pub fn interpolate_hand(&mut self, time: Instant, add: f32) {
        self.step(time, &mut |offset, _t, dt| offset + add * dt);
    }
}
