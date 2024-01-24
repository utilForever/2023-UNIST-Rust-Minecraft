#[rustfmt::skip]
pub fn unit_cube_array(
    position: (f32, f32, f32),
    uv_bl: (f32, f32),
    uv_tr: (f32, f32),
    sides: [bool; 6],
) -> Vec<f32> {
    let (x, y, z) = position;
    let (front, back, left, right, up, down) =
        (sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
    let mut array = Vec::new();

    if front {
        array.extend_from_slice(&[
            0.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
        ]);
    }

    if back {
        array.extend_from_slice(&[
            1.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
            0.0 + x, 0.0 + y, 0.0 + z, uv_tr.0, uv_bl.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_bl.0, uv_tr.1,
            1.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
        ]);
    }

    if left {
        array.extend_from_slice(&[
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
            0.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_bl.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
        ]);
    }

    if right {
        array.extend_from_slice(&[
            1.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 0.0 + y, 0.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_tr.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
        ]);
    }

    if up {
        array.extend_from_slice(&[
            0.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 1.0 + y, 1.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 1.0 + y, 0.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 1.0 + y, 0.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 1.0 + y, 1.0 + z, uv_bl.0, uv_bl.1,
        ]);
    }

    if down {
        array.extend_from_slice(&[
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
            1.0 + x, 0.0 + y, 0.0 + z, uv_tr.0, uv_bl.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            1.0 + x, 0.0 + y, 1.0 + z, uv_tr.0, uv_tr.1,
            0.0 + x, 0.0 + y, 1.0 + z, uv_bl.0, uv_tr.1,
            0.0 + x, 0.0 + y, 0.0 + z, uv_bl.0, uv_bl.1,
        ]);
    }

    array
}
