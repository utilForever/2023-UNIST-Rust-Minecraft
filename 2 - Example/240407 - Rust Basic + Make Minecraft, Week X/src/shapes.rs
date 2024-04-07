use crate::types::UVFaces;

#[rustfmt::skip]
pub unsafe fn write_unit_cube_to_ptr(
    ptr: *mut f32,
    position: (f32, f32, f32),
    (front_uv, back_uv, top_uv, bottom_uv, left_uv, right_uv): UVFaces,
    [right, left, top, bottom, front, back]: [bool; 6],
) -> u32 {
    let (x, y, z) = position;

    let vertex_size = 5;
    let vertices_per_face = 6;
    let face_size = vertex_size * vertices_per_face;

    let mut idx = 0;
    let mut copied_vertices = 0;

    if front {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 1.0 + z, front_uv.0, front_uv.1,
            1.0 + x, 0.0 + y, 1.0 + z, front_uv.2, front_uv.1,
            1.0 + x, 1.0 + y, 1.0 + z, front_uv.2, front_uv.3,
            1.0 + x, 1.0 + y, 1.0 + z, front_uv.2, front_uv.3,
            0.0 + x, 1.0 + y, 1.0 + z, front_uv.0, front_uv.3,
            0.0 + x, 0.0 + y, 1.0 + z, front_uv.0, front_uv.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if back {
        ptr.offset(idx).copy_from_nonoverlapping([
            1.0 + x, 0.0 + y, 0.0 + z, back_uv.0, back_uv.1,
            0.0 + x, 0.0 + y, 0.0 + z, back_uv.2, back_uv.1,
            0.0 + x, 1.0 + y, 0.0 + z, back_uv.2, back_uv.3,
            0.0 + x, 1.0 + y, 0.0 + z, back_uv.2, back_uv.3,
            1.0 + x, 1.0 + y, 0.0 + z, back_uv.0, back_uv.3,
            1.0 + x, 0.0 + y, 0.0 + z, back_uv.0, back_uv.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if left {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 0.0 + z, left_uv.0, left_uv.1,
            0.0 + x, 0.0 + y, 1.0 + z, left_uv.2, left_uv.1,
            0.0 + x, 1.0 + y, 1.0 + z, left_uv.2, left_uv.3,
            0.0 + x, 1.0 + y, 1.0 + z, left_uv.2, left_uv.3,
            0.0 + x, 1.0 + y, 0.0 + z, left_uv.0, left_uv.3,
            0.0 + x, 0.0 + y, 0.0 + z, left_uv.0, left_uv.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if right {
        ptr.offset(idx).copy_from_nonoverlapping([
            1.0 + x, 0.0 + y, 1.0 + z, right_uv.0, right_uv.1,
            1.0 + x, 0.0 + y, 0.0 + z, right_uv.2, right_uv.1,
            1.0 + x, 1.0 + y, 0.0 + z, right_uv.2, right_uv.3,
            1.0 + x, 1.0 + y, 0.0 + z, right_uv.2, right_uv.3,
            1.0 + x, 1.0 + y, 1.0 + z, right_uv.0, right_uv.3,
            1.0 + x, 0.0 + y, 1.0 + z, right_uv.0, right_uv.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if top {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 1.0 + y, 1.0 + z, top_uv.0, top_uv.1,
            1.0 + x, 1.0 + y, 1.0 + z, top_uv.2, top_uv.1,
            1.0 + x, 1.0 + y, 0.0 + z, top_uv.2, top_uv.3,
            1.0 + x, 1.0 + y, 0.0 + z, top_uv.2, top_uv.3,
            0.0 + x, 1.0 + y, 0.0 + z, top_uv.0, top_uv.3,
            0.0 + x, 1.0 + y, 1.0 + z, top_uv.0, top_uv.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if bottom {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 0.0 + z, bottom_uv.0, bottom_uv.1,
            1.0 + x, 0.0 + y, 0.0 + z, bottom_uv.2, bottom_uv.1,
            1.0 + x, 0.0 + y, 1.0 + z, bottom_uv.2, bottom_uv.3,
            1.0 + x, 0.0 + y, 1.0 + z, bottom_uv.2, bottom_uv.3,
            0.0 + x, 0.0 + y, 1.0 + z, bottom_uv.0, bottom_uv.3,
            0.0 + x, 0.0 + y, 0.0 + z, bottom_uv.0, bottom_uv.1,
        ].as_ptr(), face_size);

        // idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    copied_vertices as u32
}
