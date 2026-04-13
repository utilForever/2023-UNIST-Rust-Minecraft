use crate::chunk::BlockID;
use crate::shapes::centered_unit_cube;
use crate::types::TexturePack;
use specs::{Component, DenseVecStorage};

#[derive(Component)]
pub struct MainHand {
    pub begin_switch: bool,
    pub showing_item: Option<BlockID>,
    pub render: MainHandRender,
    pub switching_to: Option<BlockID>,
}

impl MainHand {
    pub fn new() -> Self {
        Self {
            begin_switch: false,
            showing_item: None,
            render: MainHandRender::new(),
            switching_to: None,
        }
    }

    pub fn switch_item_to(&mut self, item: Option<BlockID>) {
        self.switching_to = item;
        self.begin_switch = true;
    }

    pub fn set_showing_item(&mut self, item: Option<BlockID>) {
        self.showing_item = item;
        self.render.dirty = true;
    }

    pub fn update_if_dirty(&mut self, texture_pack: &TexturePack) {
        if let Some(item) = self.showing_item {
            self.render.update_vbo_if_dirty(item, &texture_pack);
        }
    }
}

pub struct MainHandRender {
    pub vao: u32,
    pub vbo: u32,
    pub dirty: bool,
}

impl MainHandRender {
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

        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            0,
            vbo,
            0,
            (9 * std::mem::size_of::<f32>()) as i32
        ));

        Self {
            vao,
            vbo,
            dirty: true,
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

        gl_call!(gl::NamedBufferData(
            self.vbo,
            (vbo_data.len() * std::mem::size_of::<f32>()) as isize,
            vbo_data.as_ptr() as *const _,
            gl::DYNAMIC_DRAW
        ));
    }
}
