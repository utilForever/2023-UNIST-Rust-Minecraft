use nalgebra_glm::{floor, IVec3, Vec3};
use num_traits::float::FloatCore;

pub fn raycast(
    is_solid_block_at: &dyn Fn(i32, i32, i32) -> bool,
    origin: &Vec3,
    direction: &Vec3,
    distance: f32,
) -> Option<((i32, i32, i32), IVec3)> {
    let mut t = 0.0f32;
    let mut i = floor(origin).map(|x| x as i32);
    let step = direction.map(|x| if x > 0f32 { 1 } else { -1 });
    let t_delta = direction.map(|x| (1.0 / x).abs());
    let dist = origin.zip_zip_map(&i, &step, |p, i, s| {
        if s > 0 {
            i as f32 + 1.0 - p
        } else {
            p - i as f32
        }
    });
    let mut t_max = t_delta.zip_map(&dist, |t, d| {
        if t.is_finite() {
            t * d
        } else {
            f32::infinity()
        }
    });

    let mut _hit_pos = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_norm = IVec3::new(0, 0, 0);

    let mut stepped_index = -1;
    while t <= distance {
        // exit check
        if is_solid_block_at(i.x, i.y, i.z) {
            _hit_pos = origin.zip_map(direction, |p, d| p + t * d);
            if stepped_index == 0 {
                hit_norm[0] = -step.x;
            }
            if stepped_index == 1 {
                hit_norm[1] = -step.y;
            }
            if stepped_index == 2 {
                hit_norm[2] = -step.z;
            }
            return Some(((i.x, i.y, i.z), hit_norm));
        }

        // advance t to next nearest voxel boundary
        let min = t_max.x.min(t_max.y.min(t_max.z));

        if min == t_max.x {
            i.x += step.x;
            t = t_max.x;
            t_max.x += t_delta.x;
            stepped_index = 0;
        } else if min == t_max.y {
            i.y += step.y;
            t = t_max.y;
            t_max.y += t_delta.y;
            stepped_index = 1;
        } else {
            i.z += step.z;
            t = t_max.z;
            t_max.z += t_delta.z;
            stepped_index = 2;
        }
    }

    // no voxel hit found
    _hit_pos = origin.zip_map(direction, |p, d| p + t * d);

    None
}
