use crate::block_texture_faces::BlockFaces;
use crate::chunk::BlockID;
use crate::constants::ITEM_ARRAY_TEXTURE_LAYERS;
use crate::types::{TextureLayer, TexturePack};
use image::{ColorType, DynamicImage};
use std::collections::HashMap;
use std::os::raw::c_void;

pub fn generate_array_texture() -> (u32, TexturePack) {
    let face_images = create_face_images_map();
    let array_texture = create_array_texture(ITEM_ARRAY_TEXTURE_LAYERS as i32);
    let face_uvs = create_face_uvs_map(array_texture, face_images);

    (array_texture, face_uvs)
}

fn create_face_images_map() -> HashMap<BlockID, BlockFaces<&'static str>> {
    let mut face_images: HashMap<BlockID, BlockFaces<&str>> = HashMap::new();

    face_images.insert(BlockID::Dirt, BlockFaces::All("textures/blocks/dirt.png"));
    face_images.insert(
        BlockID::GrassBlock,
        BlockFaces::Sides {
            sides: "textures/blocks/grass_block_side.png",
            top: "textures/blocks/grass_block_top.png",
            bottom: "textures/blocks/dirt.png",
        },
    );
    face_images.insert(BlockID::Stone, BlockFaces::All("textures/blocks/stone.png"));
    face_images.insert(
        BlockID::Cobblestone,
        BlockFaces::All("textures/blocks/cobblestone.png"),
    );
    face_images.insert(
        BlockID::Bedrock,
        BlockFaces::All("textures/blocks/bedrock.png"),
    );
    face_images.insert(
        BlockID::Obsidian,
        BlockFaces::All("textures/blocks/obsidian.png"),
    );
    face_images.insert(
        BlockID::OakLog,
        BlockFaces::Sides {
            sides: "textures/blocks/oak_log.png",
            top: "textures/blocks/oak_log_top.png",
            bottom: "textures/blocks/oak_log_top.png",
        },
    );
    face_images.insert(
        BlockID::OakLeaves,
        BlockFaces::All("textures/blocks/oak_leaves.png"),
    );
    face_images.insert(
        BlockID::OakPlanks,
        BlockFaces::All("textures/blocks/oak_planks.png"),
    );
    face_images.insert(BlockID::Glass, BlockFaces::All("textures/blocks/glass.png"));
    face_images.insert(BlockID::Debug, BlockFaces::All("textures/blocks/debug.png"));
    face_images.insert(
        BlockID::Debug2,
        BlockFaces::All("textures/blocks/debug2.png"),
    );

    face_images
}

fn create_array_texture(layers: i32) -> u32 {
    let mut item_array_texture = 0;

    gl_call!(gl::CreateTextures(
        gl::TEXTURE_2D_ARRAY,
        1,
        &mut item_array_texture
    ));
    gl_call!(gl::TextureParameteri(
        item_array_texture,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST_MIPMAP_NEAREST as i32
    ));
    gl_call!(gl::TextureParameteri(
        item_array_texture,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));
    gl_call!(gl::TextureStorage3D(
        item_array_texture,
        1,
        gl::RGBA8,
        16,
        16,
        layers,
    ));

    item_array_texture
}

fn create_face_uvs_map(
    array_texture: u32,
    face_images: HashMap<BlockID, BlockFaces<&str>>,
) -> TexturePack {
    // Load all the images and fill the UV map for all the blocks
    let mut layer = 0;

    // Put an image into the array texture at layer "layer"
    let mut put_image_into_array_texture = |image: &DynamicImage| {
        let layer_blit = layer;
        blit_image_to_texture(image, array_texture, layer_blit as i32);

        // Advance to the next available layer in the texture
        layer += 1;

        // Return layer where we put the texture
        layer_blit
    };

    let mut face_uvs = HashMap::<BlockID, BlockFaces<TextureLayer>>::new();

    for (block, faces) in face_images {
        match faces {
            BlockFaces::All(all) => {
                face_uvs.insert(
                    block,
                    BlockFaces::All(put_image_into_array_texture(&mut read_image(all))),
                );
            }
            BlockFaces::Sides { sides, top, bottom } => {
                face_uvs.insert(
                    block,
                    BlockFaces::Sides {
                        sides: put_image_into_array_texture(&mut read_image(sides)),
                        top: put_image_into_array_texture(&mut read_image(top)),
                        bottom: put_image_into_array_texture(&mut read_image(bottom)),
                    },
                );
            }
            BlockFaces::Each {
                top,
                bottom,
                front,
                back,
                left,
                right,
            } => {
                face_uvs.insert(
                    block,
                    BlockFaces::Each {
                        top: put_image_into_array_texture(&mut read_image(top)),
                        bottom: put_image_into_array_texture(&mut read_image(bottom)),
                        front: put_image_into_array_texture(&mut read_image(front)),
                        back: put_image_into_array_texture(&mut read_image(back)),
                        left: put_image_into_array_texture(&mut read_image(left)),
                        right: put_image_into_array_texture(&mut read_image(right)),
                    },
                );
            }
        }
    }

    face_uvs
}

fn read_image(image_path: &str) -> DynamicImage {
    let img = image::open(image_path);
    let img = match img {
        Ok(img) => img.flipv(),
        Err(err) => panic!("Filename: {image_path}, error: {}", err.to_string()),
    };

    match img.color() {
        ColorType::Rgba8 => {}
        _ => panic!("Texture format not supported"),
    };

    img
}

fn blit_image_to_texture(src: &DynamicImage, texture: u32, layer: i32) {
    gl_call!(gl::TextureSubImage3D(
        texture,
        0,
        0,
        0,
        layer,
        src.width() as i32,
        src.height() as i32,
        1,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        src.as_bytes().as_ptr() as *mut c_void
    ));
}
