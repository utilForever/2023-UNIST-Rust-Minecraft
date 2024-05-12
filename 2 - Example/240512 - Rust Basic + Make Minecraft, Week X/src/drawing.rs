use crate::constants::{GUI_SCALING, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::inventory::Inventory;
use crate::shader::ShaderProgram;
use nalgebra::Matrix4;
use nalgebra_glm::vec3;

impl Inventory {
    pub fn draw_hotbar(&self, vao: u32, shader: &mut ShaderProgram) {
        let model_matrix = {
            let translate_matrix =
                Matrix4::new_translation(&vec3(WINDOW_WIDTH as f32 / 2.0, 11.0 * GUI_SCALING, 0.0));
            let scale_matrix = Matrix4::new_nonuniform_scaling(&vec3(
                182.0 * GUI_SCALING,
                22.0 * GUI_SCALING,
                1.0,
            ));

            translate_matrix * scale_matrix
        };
        let projection_matrix = nalgebra_glm::ortho(
            0.0,
            WINDOW_WIDTH as f32,
            0.0,
            WINDOW_HEIGHT as f32,
            -5.0,
            5.0,
        );

        shader.use_program();
        unsafe {
            shader.set_uniform_matrix4fv("model", model_matrix.as_ptr());
            shader.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
        }
        shader.set_uniform1i("tex", 2);

        gl_call!(gl::BindVertexArray(vao));
        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
    }

    pub fn draw_hotbar_selection_box(&self, vao: u32, shader: &mut ShaderProgram) {
        let inter_slot_spacing = 20.0;
        let hotbar_left_margin = WINDOW_WIDTH as f32 / 2.0 - 4.0 * inter_slot_spacing * GUI_SCALING;
        let selection_box_x_pos = hotbar_left_margin
            + inter_slot_spacing * self.selected_hotbar_slot as f32 * GUI_SCALING;

        let model_matrix = {
            let translate_matrix =
                Matrix4::new_translation(&vec3(selection_box_x_pos, 11.0 * GUI_SCALING, 0.0));
            let scale_matrix =
                Matrix4::new_nonuniform_scaling(&vec3(24.0 * GUI_SCALING, 24.0 * GUI_SCALING, 1.0));

            translate_matrix * scale_matrix
        };
        let projection_matrix = nalgebra_glm::ortho(
            0.0,
            WINDOW_WIDTH as f32,
            0.0,
            WINDOW_HEIGHT as f32,
            -5.0,
            5.0,
        );

        shader.use_program();
        unsafe {
            shader.set_uniform_matrix4fv("model", model_matrix.as_ptr());
            shader.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
        }
        shader.set_uniform1i("tex", 2);

        gl_call!(gl::BindVertexArray(vao));
        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
    }

    pub fn draw_hotbar_items(&self, shader: &mut ShaderProgram) {
        let inter_slot_spacing = 20.0;
        let hotbar_left_margin = WINDOW_WIDTH as f32 / 2.0 - 4.0 * inter_slot_spacing * GUI_SCALING;

        let mut x = 0;
        let y = 11;

        for slot in self.slots.iter() {
            if let Some(slot) = slot {
                let item_x_pos = hotbar_left_margin + (x as f32) * inter_slot_spacing * GUI_SCALING;
                slot.item_render
                    .draw(item_x_pos, y as f32 * GUI_SCALING, shader);
            }

            x += 1;
        }
    }
}
