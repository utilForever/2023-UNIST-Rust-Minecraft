use crate::chunk::BlockID;
use crate::constants::{GUI_SCALING, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::shader::ShaderProgram;
use crate::shapes::centered_unit_cube;
use crate::types::TexturePack;
use nalgebra::Matrix4;
use nalgebra_glm::{pi, vec3, Mat4};
use std::ptr::null;

#[derive(Copy, Clone)]
pub struct ItemStack {
    pub item: BlockID,
    pub amount: u32,
    pub(crate) item_render: ItemRender,
}

impl ItemStack {
    pub fn new(block: BlockID, amount: u32) -> Self {
        Self {
            item: block,
            amount,
            item_render: ItemRender::new(),
        }
    }

    pub fn update_if_dirty(&mut self, texture_pack: &TexturePack) {
        self.item_render
            .update_vbo_if_dirty(self.item, &texture_pack);
    }
}

#[derive(Copy, Clone)]
pub struct ItemRender {
    vao: u32,
    vbo: u32,
    // This is dirty when the VBO needs to be updated (at creation and when changing the block)
    pub(crate) dirty: bool,
    projection_matrix: Mat4,
}

impl ItemRender {
    pub fn new() -> Self {
        let mut vao = 0;
        gl_call!(gl::CreateVertexArrays(1, &mut vao));

        // Position
        gl_call!(gl::EnableVertexArrayAttrib(vao, 0));
        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            0
        ));
        gl_call!(gl::VertexArrayAttribBinding(vao, 0, 0));

        // Texture coords
        gl_call!(gl::EnableVertexArrayAttrib(vao, 1));
        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<f32>() as u32
        ));
        gl_call!(gl::VertexArrayAttribBinding(vao, 1, 0));

        // Normals
        gl_call!(gl::EnableVertexArrayAttrib(vao, 2));
        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            2,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * std::mem::size_of::<f32>() as u32
        ));
        gl_call!(gl::VertexArrayAttribBinding(vao, 2, 0));

        let mut vbo = 0;
        gl_call!(gl::CreateBuffers(1, &mut vbo));

        gl_call!(gl::NamedBufferData(
            vbo,
            (9 * 6 * 6 * std::mem::size_of::<f32>()) as isize,
            null(),
            gl::DYNAMIC_DRAW
        ));
        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            0,
            vbo,
            0,
            (9 * std::mem::size_of::<f32>()) as i32
        ));

        let projection_matrix = nalgebra_glm::ortho(
            0.0,
            WINDOW_WIDTH as f32,
            0.0,
            WINDOW_HEIGHT as f32,
            -1000.0,
            1000.0,
        );

        Self {
            vao,
            vbo,
            dirty: true,
            projection_matrix,
        }
    }

    pub fn update_vbo_if_dirty(&mut self, item: BlockID, texture_pack: &TexturePack) {
        if self.dirty {
            self.update_vbo(item, &texture_pack);
            self.dirty = false;
        }
    }

    pub fn update_vbo(&mut self, item: BlockID, texture_pack: &TexturePack) {
        let vbo_data = centered_unit_cube(
            -0.5,
            -0.5,
            -0.5,
            texture_pack.get(&item).unwrap().get_uv_of_every_face(),
        );

        gl_call!(gl::NamedBufferSubData(
            self.vbo,
            0,
            (vbo_data.len() * std::mem::size_of::<f32>()) as isize,
            vbo_data.as_ptr() as *const _,
        ));
    }

    pub fn draw(&self, x: f32, y: f32, shader: &mut ShaderProgram) {
        let model_matrix = {
            let translate_matrix = Matrix4::new_translation(&vec3(x, y, 1.0));
            let rotate_matrix = {
                let rotate_y = Matrix4::from_euler_angles(0.0, pi::<f32>() / 4.0, 0.0); // 45 degrees
                let rotate_x = Matrix4::from_euler_angles(pi::<f32>() / 6.0, 0.0, 0.0); // 30 degrees

                rotate_y * rotate_x
            };
            let scale_matrix =
                Matrix4::new_nonuniform_scaling(&(GUI_SCALING * vec3(10.0, 10.0, 10.0)));

            translate_matrix * rotate_matrix * scale_matrix
        };

        shader.use_program();
        unsafe {
            shader.set_uniform_matrix4fv("model", model_matrix.as_ptr());
            shader.set_uniform_matrix4fv("projection", self.projection_matrix.as_ptr());
        }
        shader.set_uniform1i("tex", 0);

        gl_call!(gl::BindVertexArray(self.vao));
        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 36));
    }
}
