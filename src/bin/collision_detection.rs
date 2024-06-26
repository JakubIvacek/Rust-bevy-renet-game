pub struct AABB {
    pub min: (f32, f32),
    pub max: (f32, f32),
}

// Function to detect collision and return depth x/y overlaping of 2 rectangles
pub fn test_aabb_overlap(a: AABB, b: AABB) -> Option<(f32, f32)>{
    let d1x = b.min.0 - a.max.0;
    let d1y = b.min.1 - a.max.1;
    let d2x = a.min.0 - b.max.0;
    let d2y = a.min.1 - b.max.1;


    // Depth of x-cor overlap
    let depth_x = if d1x <= 0.0 && d2x <= 0.0 {
        if d1x.abs() < d2x.abs() {
            d1x
        } else {
            -d2x
        }
    } else {
        0.0
    };

    // Depth of y-cor overlap
    let depth_y = if d1y <= 0.0 && d2y <= 0.0 {
        if d1y.abs() < d2y.abs() {
            d1y
        } else {
            -d2y
        }
    } else {
        0.0
    };

    // Return none if no collision return Some if collision
    if depth_x == 0.0 || depth_y == 0.0 {
        None
    } else {
        //potom v podstate skontrolujeme ktora depth je vacsia a podla toho vieme na ktorej
        // stene bude prva kolizia
        Some((depth_x, depth_y))
    }

}

#[allow(dead_code)]
fn main() {}