use crate::rectangle::Rectangle;

#[derive(Debug, Clone)]
pub struct Instance {
    pub rectangles: Vec<Rectangle>,
    pub box_size: u32,
}