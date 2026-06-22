use crate::rectangle::Rectangle;

#[derive(Debug, Clone)]
pub struct Placement {
    pub rect_id: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rotated: bool,
}

#[derive(Debug, Clone)]
pub struct Bin {
    pub id: usize,
    pub size: u32,
    pub placements: Vec<Placement>,
}

fn overlap(a: &Placement, b: &Placement) -> bool {
    !(a.x + a.width <= b.x
        || b.x + b.width <= a.x
        || a.y + a.height <= b.y
        || b.y + b.height <= a.y)
}

impl Bin {
    pub fn can_place_dimensions(
        &self,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
        rect_id: usize,
    ) -> bool {
        if x + width > self.size || y + height > self.size {
            return false;
        }

        let new_placement = Placement {
            rect_id,
            x,
            y,
            width,
            height,
            rotated: false,
        };

        for p in &self.placements {
            if overlap(p, &new_placement) {
                return false;
            }
        }

        true
    }

    pub fn can_place_dimensions_ignoring(
        &self,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
        rect_id: usize,
    ) -> bool {
        if x + width > self.size || y + height > self.size {
            return false;
        }

        let new_placement = Placement {
            rect_id,
            x,
            y,
            width,
            height,
            rotated: false,
        };

        for p in &self.placements {
            if p.rect_id == rect_id {
                continue;
            }
            if overlap(p, &new_placement) {
                return false;
            }
        }

        true
    }

    pub fn place_dimensions(
        &mut self,
        rect_id: usize,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
        rotated: bool,
    ) {
        self.placements.push(Placement {
            rect_id,
            x,
            y,
            width,
            height,
            rotated,
        });
    }

    pub fn try_place_first_fit(&mut self, rect: &Rectangle) -> bool {
        for x in 0..self.size {
            for y in 0..self.size {
                if self.can_place_dimensions(rect.width, rect.height, x, y, rect.id) {
                    self.place_dimensions(rect.id, rect.width, rect.height, x, y, false);
                    return true;
                }

                if rect.width != rect.height
                    && self.can_place_dimensions(rect.height, rect.width, x, y, rect.id)
                {
                    self.place_dimensions(rect.id, rect.height, rect.width, x, y, true);
                    return true;
                }
            }
        }
        false
    }

    pub fn remove_placement_by_rect_id(&mut self, rect_id: usize) -> Option<Placement> {
        if let Some(index) = self.placements.iter().position(|p| p.rect_id == rect_id) {
            Some(self.placements.remove(index))
        } else {
            None
        }
    }

    pub fn get_placement_by_rect_id(&self, rect_id: usize) -> Option<&Placement> {
        self.placements.iter().find(|p| p.rect_id == rect_id)
    }

    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }
}