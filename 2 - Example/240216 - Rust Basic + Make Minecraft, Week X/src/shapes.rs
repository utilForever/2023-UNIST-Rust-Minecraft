use crate::chunk_manager::Sides;

#[rustfmt::skip]
pub unsafe fn write_unit_cube_to_ptr(
    ptr: *mut f32,
    position: (f32, f32, f32),
    uv_bl: (f32, f32),
    uv_tr: (f32, f32),
    sides: Sides,
) -> u32 {
    let (x, y, z) = position;
    let (right, left, top, bottom, front, back) = sides;

    let vertex_size = 5;
    let vertices_per_face = 6;
    let face_size = vertex_size * vertices_per_face;

    let mut idx = 0;
    let mut copied_vertices = 0;

    if front {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if back {
        ptr.offset(idx).copy_from_nonoverlapping([
            1.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
            0.0 + x, 0.0 + y, 0.0 + z, uv_tr.0, uv_bl.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_bl.0, uv_tr.1,
            1.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if left {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
            0.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_bl.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if right {
        ptr.offset(idx).copy_from_nonoverlapping([
            1.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 0.0 + y, 0.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_tr.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if top {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
        ].as_ptr(), face_size);

        idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    if bottom {
        ptr.offset(idx).copy_from_nonoverlapping([
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 0.0 + y, 0.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
        ].as_ptr(), face_size);

        // idx += face_size as isize;
        copied_vertices += vertices_per_face;
    }

    copied_vertices as u32
}
