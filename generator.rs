use rand::Rng;

use crate::instance::Instance;
use crate::rectangle::Rectangle;

pub fn generate_instance(
    count: usize,
    min_w: u32,
    max_w: u32,
    min_h: u32,
    max_h: u32,
    box_size: u32,
) -> Instance {
    let mut rng = rand::thread_rng();
    let mut rectangles = Vec::new();

    for id in 0..count {
        let width = rng.gen_range(min_w..=max_w);
        let height = rng.gen_range(min_h..=max_h);

        rectangles.push(Rectangle { id, width, height });
    }

    Instance { box_size, rectangles }
}