use crate::types::UVFaces;

#[rustfmt::skip]
pub fn quad(uv: (f32, f32, f32, f32)) -> Vec<f32> {
    (&[
        -0.5, -0.5, 0.0, uv.0, uv.3,
         0.5, -0.5, 0.0, uv.2, uv.3,
         0.5,  0.5, 0.0, uv.2, uv.1,
         0.5,  0.5, 0.0, uv.2, uv.1,
        -0.5,  0.5, 0.0, uv.0, uv.1,
        -0.5, -0.5, 0.0, uv.0, uv.3,
    ]).to_vec()
}

#[rustfmt::skip]
pub fn quad_array_texture() -> Vec<f32> {
    (&[
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
         0.5,  0.5, 0.0,
         0.5,  0.5, 0.0,
        -0.5,  0.5, 0.0,
        -0.5, -0.5, 0.0,
    ]).to_vec()
}

#[rustfmt::skip]
pub unsafe fn write_unit_cube_to_ptr(
    ptr: *mut f32,
    position: (f32, f32, f32),
    (front_layer, back_layer, top_layer, bottom_layer, left_layer, right_layer): UVFaces,
    [right, left, top, bottom, front, back]: [bool; 6],
    ao: [[u8; 4]; 6],
) -> u32 {
    let (x, y, z) = position;

    let vertex_size = 10;
    let vertices_per_face = 6;
    let face_size = vertex_size * vertices_per_face;

    let mut idx = 0;
    let mut copied_vertices = 0;

    let uv = (0.0, 0.0, 1.0, 1.0);

    if front {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, front_layer as f32, 0.0, 0.0, 1.0, ao[4][0] as f32,
            1.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.1, front_layer as f32, 0.0, 0.0, 1.0, ao[4][1] as f32,
            1.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, front_layer as f32, 0.0, 0.0, 1.0, ao[4][2] as f32,
            1.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, front_layer as f32, 0.0, 0.0, 1.0, ao[4][2] as f32,
            0.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.3, front_layer as f32, 0.0, 0.0, 1.0, ao[4][3] as f32,
            0.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, front_layer as f32, 0.0, 0.0, 1.0, ao[4][0] as f32,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if back {
        ptr.offset(idx).copy_from_nonoverlapping([
            1.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, back_layer as f32, 0.0, 0.0, -1.0, ao[5][0] as f32,
            0.0 + x, 0.0 + y, 0.0 + z, uv.2, uv.1, back_layer as f32, 0.0, 0.0, -1.0, ao[5][1] as f32,
            0.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, back_layer as f32, 0.0, 0.0, -1.0, ao[5][2] as f32,
            0.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, back_layer as f32, 0.0, 0.0, -1.0, ao[5][2] as f32,
            1.0 + x, 1.0 + y, 0.0 + z, uv.0, uv.3, back_layer as f32, 0.0, 0.0, -1.0, ao[5][3] as f32,
            1.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, back_layer as f32, 0.0, 0.0, -1.0, ao[5][0] as f32,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if left {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, left_layer as f32, -1.0, 0.0, 0.0, ao[1][0] as f32,
            0.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.1, left_layer as f32, -1.0, 0.0, 0.0, ao[1][1] as f32,
            0.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, left_layer as f32, -1.0, 0.0, 0.0, ao[1][2] as f32,
            0.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, left_layer as f32, -1.0, 0.0, 0.0, ao[1][2] as f32,
            0.0 + x, 1.0 + y, 0.0 + z, uv.0, uv.3, left_layer as f32, -1.0, 0.0, 0.0, ao[1][3] as f32,
            0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, left_layer as f32, -1.0, 0.0, 0.0, ao[1][0] as f32,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if right {
        ptr.offset(idx).copy_from_nonoverlapping([
            1.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, right_layer as f32, 1.0, 0.0, 0.0, ao[0][0] as f32,
            1.0 + x, 0.0 + y, 0.0 + z, uv.2, uv.1, right_layer as f32, 1.0, 0.0, 0.0, ao[0][1] as f32,
            1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, right_layer as f32, 1.0, 0.0, 0.0, ao[0][2] as f32,
            1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, right_layer as f32, 1.0, 0.0, 0.0, ao[0][2] as f32,
            1.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.3, right_layer as f32, 1.0, 0.0, 0.0, ao[0][3] as f32,
            1.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, right_layer as f32, 1.0, 0.0, 0.0, ao[0][0] as f32,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if top {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.1, top_layer as f32, 0.0, 1.0, 0.0, ao[2][0] as f32,
            1.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.1, top_layer as f32, 0.0, 1.0, 0.0, ao[2][1] as f32,
            1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, top_layer as f32, 0.0, 1.0, 0.0, ao[2][2] as f32,
            1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, top_layer as f32, 0.0, 1.0, 0.0, ao[2][2] as f32,
            0.0 + x, 1.0 + y, 0.0 + z, uv.0, uv.3, top_layer as f32, 0.0, 1.0, 0.0, ao[2][3] as f32,
            0.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.1, top_layer as f32, 0.0, 1.0, 0.0, ao[2][0] as f32,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if bottom {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, bottom_layer as f32, 0.0, -1.0, 0.0, ao[3][0] as f32,
            1.0 + x, 0.0 + y, 0.0 + z, uv.2, uv.1, bottom_layer as f32, 0.0, -1.0, 0.0, ao[3][1] as f32,
            1.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.3, bottom_layer as f32, 0.0, -1.0, 0.0, ao[3][2] as f32,
            1.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.3, bottom_layer as f32, 0.0, -1.0, 0.0, ao[3][2] as f32,
            0.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.3, bottom_layer as f32, 0.0, -1.0, 0.0, ao[3][3] as f32,
            0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, bottom_layer as f32, 0.0, -1.0, 0.0, ao[3][0] as f32,
        ].as_ptr(), face_size);

        // idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    copied_vertices as u32
}

// Reference: https://stackoverflow.com/questions/25195363/draw-cube-vertices-with-fewest-number-of-steps
#[rustfmt::skip]
pub fn block_outline() -> &'static [f32; 72] {
    // Groups of parallel lines for each dimension
    &[
        0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 1.0, 1.0, 0.0,
        0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        0.0, 0.0, 1.0, 1.0, 0.0, 1.0,

        0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        1.0, 0.0, 0.0, 1.0, 1.0, 0.0,
        1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0,

        0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 0.0, 1.0, 1.0, 1.0,
        0.0, 1.0, 0.0, 0.0, 1.0, 1.0,
    ]
}

#[rustfmt::skip]
pub fn centered_unit_cube(x: f32, y: f32, z: f32, (front_layer, back_layer, top_layer, bottom_layer, left_layer, right_layer): UVFaces) -> Vec<f32> {
    // Position: 3 floats
    // UV coords: 3 floats
    // Normal: 3 floats
    let uv = (0.0, 0.0, 1.0, 1.0);

    [
        0.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, front_layer as f32, 0.0, 0.0, 1.0,
        1.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.1, front_layer as f32, 0.0, 0.0, 1.0,
        1.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, front_layer as f32, 0.0, 0.0, 1.0,
        1.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, front_layer as f32, 0.0, 0.0, 1.0,
        0.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.3, front_layer as f32, 0.0, 0.0, 1.0,
        0.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, front_layer as f32, 0.0, 0.0, 1.0,

        1.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, back_layer as f32, 0.0, 0.0, -1.0,
        0.0 + x, 0.0 + y, 0.0 + z, uv.2, uv.1, back_layer as f32, 0.0, 0.0, -1.0,
        0.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, back_layer as f32, 0.0, 0.0, -1.0,
        0.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, back_layer as f32, 0.0, 0.0, -1.0,
        1.0 + x, 1.0 + y, 0.0 + z, uv.0, uv.3, back_layer as f32, 0.0, 0.0, -1.0,
        1.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, back_layer as f32, 0.0, 0.0, -1.0,

        0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, left_layer as f32, -1.0, 0.0, 0.0,
        0.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.1, left_layer as f32, -1.0, 0.0, 0.0,
        0.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, left_layer as f32, -1.0, 0.0, 0.0,
        0.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.3, left_layer as f32, -1.0, 0.0, 0.0,
        0.0 + x, 1.0 + y, 0.0 + z, uv.0, uv.3, left_layer as f32, -1.0, 0.0, 0.0,
        0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, left_layer as f32, -1.0, 0.0, 0.0,

        1.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, right_layer as f32, 1.0, 0.0, 0.0,
        1.0 + x, 0.0 + y, 0.0 + z, uv.2, uv.1, right_layer as f32, 1.0, 0.0, 0.0,
        1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, right_layer as f32, 1.0, 0.0, 0.0,
        1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, right_layer as f32, 1.0, 0.0, 0.0,
        1.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.3, right_layer as f32, 1.0, 0.0, 0.0,
        1.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.1, right_layer as f32, 1.0, 0.0, 0.0,

        0.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.1, top_layer as f32, 0.0, 1.0, 0.0,
        1.0 + x, 1.0 + y, 1.0 + z, uv.2, uv.1, top_layer as f32, 0.0, 1.0, 0.0,
        1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, top_layer as f32, 0.0, 1.0, 0.0,
        1.0 + x, 1.0 + y, 0.0 + z, uv.2, uv.3, top_layer as f32, 0.0, 1.0, 0.0,
        0.0 + x, 1.0 + y, 0.0 + z, uv.0, uv.3, top_layer as f32, 0.0, 1.0, 0.0,
        0.0 + x, 1.0 + y, 1.0 + z, uv.0, uv.1, top_layer as f32, 0.0, 1.0, 0.0,

        0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, bottom_layer as f32, 0.0, -1.0, 0.0,
        1.0 + x, 0.0 + y, 0.0 + z, uv.2, uv.1, bottom_layer as f32, 0.0, -1.0, 0.0,
        1.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.3, bottom_layer as f32, 0.0, -1.0, 0.0,
        1.0 + x, 0.0 + y, 1.0 + z, uv.2, uv.3, bottom_layer as f32, 0.0, -1.0, 0.0,
        0.0 + x, 0.0 + y, 1.0 + z, uv.0, uv.3, bottom_layer as f32, 0.0, -1.0, 0.0,
        0.0 + x, 0.0 + y, 0.0 + z, uv.0, uv.1, bottom_layer as f32, 0.0, -1.0, 0.0,
    ].to_vec()
}
