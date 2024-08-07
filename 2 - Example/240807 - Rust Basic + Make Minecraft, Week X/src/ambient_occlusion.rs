use std::collections::HashMap;

pub fn compute_ao_of_block(does_occlude: &dyn Fn(i32, i32, i32) -> bool) -> [[u8; 4]; 6] {
    let mut ao_block = [[0; 4]; 6];

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                if does_occlude(x, y, z) {
                    if let Some(affected) = AO_AFFECTED_VERTICES.get(&(x, y, z)) {
                        for &(face, vertex) in affected {
                            ao_block[face as usize][vertex as usize] += 1;
                        }
                    }
                }
            }
        }
    }

    ao_block
}

lazy_static! {
    static ref AO_AFFECTED_VERTICES: HashMap<(i32, i32, i32), Vec<(u8, u8)>> = {
        let mut affected_vertices: HashMap<(i32, i32, i32), Vec<(u8, u8)>> = HashMap::new();
        let mut add = |key, value| affected_vertices.entry(key).or_default().push(value);

        const RIGHT: u8 = 0;
        const LEFT: u8 = 1;
        const TOP: u8 = 2;
        const BOTTOM: u8 = 3;
        const FRONT: u8 = 4;
        const BACK: u8 = 5;

        // Corners bottom
        add((-1, -1, -1), (LEFT, 0));
        add((-1, -1, -1), (BOTTOM, 0));
        add((-1, -1, -1), (BACK, 1));

        add((1, -1, -1), (RIGHT, 1));
        add((1, -1, -1), (BOTTOM, 1));
        add((1, -1, -1), (BACK, 0));

        add((1, -1, 1), (RIGHT, 0));
        add((1, -1, 1), (BOTTOM, 2));
        add((1, -1, 1), (FRONT, 1));

        add((-1, -1, 1), (LEFT, 1));
        add((-1, -1, 1), (BOTTOM, 3));
        add((-1, -1, 1), (FRONT, 0));

        // Corners top
        add((-1, 1, -1), (LEFT, 3));
        add((-1, 1, -1), (TOP, 3));
        add((-1, 1, -1), (BACK, 2));

        add((1, 1, -1), (RIGHT, 2));
        add((1, 1, -1), (TOP, 2));
        add((1, 1, -1), (BACK, 3));

        add((1, 1, 1), (RIGHT, 3));
        add((1, 1, 1), (TOP, 1));
        add((1, 1, 1), (FRONT, 2));

        add((-1, 1, 1), (LEFT, 2));
        add((-1, 1, 1), (TOP, 0));
        add((-1, 1, 1), (FRONT, 3));

        // X Edges
        add((0, -1, -1), (BOTTOM, 0));
        add((0, -1, -1), (BOTTOM, 1));
        add((0, -1, -1), (BACK, 0));
        add((0, -1, -1), (BACK, 1));

        add((0, 1, -1), (TOP, 2));
        add((0, 1, -1), (TOP, 3));
        add((0, 1, -1), (BACK, 2));
        add((0, 1, -1), (BACK, 3));

        add((0, 1, 1), (TOP, 0));
        add((0, 1, 1), (TOP, 1));
        add((0, 1, 1), (FRONT, 2));
        add((0, 1, 1), (FRONT, 3));

        add((0, -1, 1), (BOTTOM, 2));
        add((0, -1, 1), (BOTTOM, 3));
        add((0, -1, 1), (FRONT, 0));
        add((0, -1, 1), (FRONT, 1));

        // Y Edges
        add((-1, 0, -1), (LEFT, 0));
        add((-1, 0, -1), (LEFT, 3));
        add((-1, 0, -1), (BACK, 1));
        add((-1, 0, -1), (BACK, 2));

        add((1, 0, -1), (RIGHT, 1));
        add((1, 0, -1), (RIGHT, 2));
        add((1, 0, -1), (BACK, 0));
        add((1, 0, -1), (BACK, 3));

        add((1, 0, 1), (RIGHT, 0));
        add((1, 0, 1), (RIGHT, 3));
        add((1, 0, 1), (FRONT, 1));
        add((1, 0, 1), (FRONT, 2));

        add((-1, 0, 1), (LEFT, 1));
        add((-1, 0, 1), (LEFT, 2));
        add((-1, 0, 1), (FRONT, 0));
        add((-1, 0, 1), (FRONT, 3));

        // Z Edges
        add((-1, -1, 0), (LEFT, 0));
        add((-1, -1, 0), (LEFT, 1));
        add((-1, -1, 0), (BOTTOM, 0));
        add((-1, -1, 0), (BOTTOM, 3));

        add((1, -1, 0), (RIGHT, 0));
        add((1, -1, 0), (RIGHT, 1));
        add((1, -1, 0), (BOTTOM, 1));
        add((1, -1, 0), (BOTTOM, 2));

        add((1, 1, 0), (RIGHT, 2));
        add((1, 1, 0), (RIGHT, 3));
        add((1, 1, 0), (TOP, 1));
        add((1, 1, 0), (TOP, 2));

        add((-1, 1, 0), (LEFT, 2));
        add((-1, 1, 0), (LEFT, 3));
        add((-1, 1, 0), (TOP, 0));
        add((-1, 1, 0), (TOP, 3));

        // Sides
        add((-1, 0, 0), (LEFT, 0));
        add((-1, 0, 0), (LEFT, 1));
        add((-1, 0, 0), (LEFT, 2));
        add((-1, 0, 0), (LEFT, 3));

        add((1, 0, 0), (RIGHT, 0));
        add((1, 0, 0), (RIGHT, 1));
        add((1, 0, 0), (RIGHT, 2));
        add((1, 0, 0), (RIGHT, 3));

        add((0, -1, 0), (BOTTOM, 0));
        add((0, -1, 0), (BOTTOM, 1));
        add((0, -1, 0), (BOTTOM, 2));
        add((0, -1, 0), (BOTTOM, 3));

        add((0, 1, 0), (TOP, 0));
        add((0, 1, 0), (TOP, 1));
        add((0, 1, 0), (TOP, 2));
        add((0, 1, 0), (TOP, 3));

        add((0, 0, -1), (BACK, 0));
        add((0, 0, -1), (BACK, 1));
        add((0, 0, -1), (BACK, 2));
        add((0, 0, -1), (BACK, 3));

        add((0, 0, 1), (FRONT, 0));
        add((0, 0, 1), (FRONT, 1));
        add((0, 0, 1), (FRONT, 2));
        add((0, 0, 1), (FRONT, 3));

        affected_vertices
    };
}
