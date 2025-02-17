pub fn compute_ao_of_block(does_occlude: &dyn Fn(i32, i32, i32) -> bool) -> [[u8; 4]; 6] {
    let mut ao_block = [[0; 4]; 6];

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                let is_corner = x != 0 && y != 0 && z != 0;

                if does_occlude(x, y, z) {
                    let idx = ao_index((x, y, z));

                    for i in idx..idx + if is_corner { 3 } else { 4 } {
                        let (face, vertex) = AO_AFFECTED_VERTICES[i];
                        ao_block[face as usize][vertex as usize] += 1;
                    }
                }
            }
        }
    }

    ao_block
}

#[inline]
pub fn ao_index((x, y, z): (i32, i32, i32)) -> usize {
    let x = x + 1;
    let y = y + 1;
    let z = z + 1;

    assert!(x >= 0);
    assert!(y >= 0);
    assert!(z >= 0);

    ((4 * 9 * x) + (4 * 3 * y) + (4 * z)) as usize
}

lazy_static! {
    static ref AO_AFFECTED_VERTICES: [(u8, u8); 27 * 4] = {
        let mut affected_vertices: [(u8, u8); 27 * 4] = [(0, 0); 27 * 4];
        let mut add = |idx, key, value| affected_vertices[ao_index(key) + idx] = value;

        const RIGHT: u8 = 0;
        const LEFT: u8 = 1;
        const TOP: u8 = 2;
        const BOTTOM: u8 = 3;
        const FRONT: u8 = 4;
        const BACK: u8 = 5;

        // Corners bottom
        add(0, (-1, -1, -1), (LEFT, 0));
        add(1, (-1, -1, -1), (BOTTOM, 0));
        add(2, (-1, -1, -1), (BACK, 1));

        add(0, (1, -1, -1), (RIGHT, 1));
        add(1, (1, -1, -1), (BOTTOM, 1));
        add(2, (1, -1, -1), (BACK, 0));

        add(0, (1, -1, 1), (RIGHT, 0));
        add(1, (1, -1, 1), (BOTTOM, 2));
        add(2, (1, -1, 1), (FRONT, 1));

        add(0, (-1, -1, 1), (LEFT, 1));
        add(1, (-1, -1, 1), (BOTTOM, 3));
        add(2, (-1, -1, 1), (FRONT, 0));

        // Corners top
        add(0, (-1, 1, -1), (LEFT, 3));
        add(1, (-1, 1, -1), (TOP, 3));
        add(2, (-1, 1, -1), (BACK, 2));

        add(0, (1, 1, -1), (RIGHT, 2));
        add(1, (1, 1, -1), (TOP, 2));
        add(2, (1, 1, -1), (BACK, 3));

        add(0, (1, 1, 1), (RIGHT, 3));
        add(1, (1, 1, 1), (TOP, 1));
        add(2, (1, 1, 1), (FRONT, 2));

        add(0, (-1, 1, 1), (LEFT, 2));
        add(1, (-1, 1, 1), (TOP, 0));
        add(2, (-1, 1, 1), (FRONT, 3));

        // X Edges
        add(0, (0, -1, -1), (BOTTOM, 0));
        add(1, (0, -1, -1), (BOTTOM, 1));
        add(2, (0, -1, -1), (BACK, 0));
        add(3, (0, -1, -1), (BACK, 1));

        add(0, (0, 1, -1), (TOP, 2));
        add(1, (0, 1, -1), (TOP, 3));
        add(2, (0, 1, -1), (BACK, 2));
        add(3, (0, 1, -1), (BACK, 3));

        add(0, (0, 1, 1), (TOP, 0));
        add(1, (0, 1, 1), (TOP, 1));
        add(2, (0, 1, 1), (FRONT, 2));
        add(3, (0, 1, 1), (FRONT, 3));

        add(0, (0, -1, 1), (BOTTOM, 2));
        add(1, (0, -1, 1), (BOTTOM, 3));
        add(2, (0, -1, 1), (FRONT, 0));
        add(3, (0, -1, 1), (FRONT, 1));

        // Y Edges
        add(0, (-1, 0, -1), (LEFT, 0));
        add(1, (-1, 0, -1), (LEFT, 3));
        add(2, (-1, 0, -1), (BACK, 1));
        add(3, (-1, 0, -1), (BACK, 2));

        add(0, (1, 0, -1), (RIGHT, 1));
        add(1, (1, 0, -1), (RIGHT, 2));
        add(2, (1, 0, -1), (BACK, 0));
        add(3, (1, 0, -1), (BACK, 3));

        add(0, (1, 0, 1), (RIGHT, 0));
        add(1, (1, 0, 1), (RIGHT, 3));
        add(2, (1, 0, 1), (FRONT, 1));
        add(3, (1, 0, 1), (FRONT, 2));

        add(0, (-1, 0, 1), (LEFT, 1));
        add(1, (-1, 0, 1), (LEFT, 2));
        add(2, (-1, 0, 1), (FRONT, 0));
        add(3, (-1, 0, 1), (FRONT, 3));

        // Z Edges
        add(0, (-1, -1, 0), (LEFT, 0));
        add(1, (-1, -1, 0), (LEFT, 1));
        add(2, (-1, -1, 0), (BOTTOM, 0));
        add(3, (-1, -1, 0), (BOTTOM, 3));

        add(0, (1, -1, 0), (RIGHT, 0));
        add(1, (1, -1, 0), (RIGHT, 1));
        add(2, (1, -1, 0), (BOTTOM, 1));
        add(3, (1, -1, 0), (BOTTOM, 2));

        add(0, (1, 1, 0), (RIGHT, 2));
        add(1, (1, 1, 0), (RIGHT, 3));
        add(2, (1, 1, 0), (TOP, 1));
        add(3, (1, 1, 0), (TOP, 2));

        add(0, (-1, 1, 0), (LEFT, 2));
        add(1, (-1, 1, 0), (LEFT, 3));
        add(2, (-1, 1, 0), (TOP, 0));
        add(3, (-1, 1, 0), (TOP, 3));

        // Sides
        add(0, (-1, 0, 0), (LEFT, 0));
        add(1, (-1, 0, 0), (LEFT, 1));
        add(2, (-1, 0, 0), (LEFT, 2));
        add(3, (-1, 0, 0), (LEFT, 3));

        add(0, (1, 0, 0), (RIGHT, 0));
        add(1, (1, 0, 0), (RIGHT, 1));
        add(2, (1, 0, 0), (RIGHT, 2));
        add(3, (1, 0, 0), (RIGHT, 3));

        add(0, (0, -1, 0), (BOTTOM, 0));
        add(1, (0, -1, 0), (BOTTOM, 1));
        add(2, (0, -1, 0), (BOTTOM, 2));
        add(3, (0, -1, 0), (BOTTOM, 3));

        add(0, (0, 1, 0), (TOP, 0));
        add(1, (0, 1, 0), (TOP, 1));
        add(2, (0, 1, 0), (TOP, 2));
        add(3, (0, 1, 0), (TOP, 3));

        add(0, (0, 0, -1), (BACK, 0));
        add(1, (0, 0, -1), (BACK, 1));
        add(2, (0, 0, -1), (BACK, 2));
        add(3, (0, 0, -1), (BACK, 3));

        add(0, (0, 0, 1), (FRONT, 0));
        add(1, (0, 0, 1), (FRONT, 1));
        add(2, (0, 0, 1), (FRONT, 2));
        add(3, (0, 0, 1), (FRONT, 3));

        affected_vertices
    };
}
