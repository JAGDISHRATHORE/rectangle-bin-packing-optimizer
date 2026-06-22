#[derive(Debug, Clone)]
pub struct Rectangle {
    pub id: usize,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}