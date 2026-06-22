use crate::bin::{Bin, Placement};
use crate::rectangle::Rectangle;

#[derive(Debug, Clone)]
struct SkylineNode {
    x: u32,
    y: u32,
    width: u32,
}

#[derive(Debug, Clone)]
pub struct SkylineBin {
    size: u32,
    nodes: Vec<SkylineNode>,
    placements: Vec<Placement>,
}

impl SkylineBin {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            nodes: vec![SkylineNode {
                x: 0,
                y: 0,
                width: size,
            }],
            placements: Vec::new(),
        }
    }

    fn fit_at(&self, index: usize, width: u32, height: u32) -> Option<(u32, u32)> {
        let x = self.nodes[index].x;
        if x + width > self.size {
            return None;
        }

        let mut width_left = width;
        let mut i = index;
        let mut y = 0;

        while width_left > 0 {
            if i >= self.nodes.len() {
                return None;
            }

            y = y.max(self.nodes[i].y);

            if y + height > self.size {
                return None;
            }

            if self.nodes[i].width >= width_left {
                break;
            }

            width_left -= self.nodes[i].width;
            i += 1;
        }

        Some((x, y))
    }

    fn find_position(&self, rect: &Rectangle) -> Option<(usize, u32, u32, u32, u32, bool)> {
        let mut best: Option<(usize, u32, u32, u32, u32, bool)> = None;

        for i in 0..self.nodes.len() {
            if let Some((x, y)) = self.fit_at(i, rect.width, rect.height) {
                let candidate = (i, x, y, rect.width, rect.height, false);
                if is_better(candidate, best) {
                    best = Some(candidate);
                }
            }

            if rect.width != rect.height {
                if let Some((x, y)) = self.fit_at(i, rect.height, rect.width) {
                    let candidate = (i, x, y, rect.height, rect.width, true);
                    if is_better(candidate, best) {
                        best = Some(candidate);
                    }
                }
            }
        }

        best
    }

    fn add_level(
        &mut self,
        index: usize,
        rect_id: usize,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
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

        self.nodes.insert(
            index,
            SkylineNode {
                x,
                y: y + height,
                width,
            },
        );

        let mut i = index + 1;
        while i < self.nodes.len() {
            let prev_end = self.nodes[i - 1].x + self.nodes[i - 1].width;

            if self.nodes[i].x < prev_end {
                let shrink = prev_end - self.nodes[i].x;

                if self.nodes[i].width <= shrink {
                    self.nodes.remove(i);
                } else {
                    self.nodes[i].x += shrink;
                    self.nodes[i].width -= shrink;
                    break;
                }
            } else {
                break;
            }
        }

        self.merge_equal_height_nodes();
    }

    fn merge_equal_height_nodes(&mut self) {
        let mut i = 0;
        while i + 1 < self.nodes.len() {
            if self.nodes[i].y == self.nodes[i + 1].y {
                let next_width = self.nodes[i + 1].width;
                self.nodes[i].width += next_width;
                self.nodes.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }

    pub fn place(&mut self, rect: &Rectangle) -> bool {
        if let Some((index, x, y, width, height, rotated)) = self.find_position(rect) {
            self.add_level(index, rect.id, x, y, width, height, rotated);
            true
        } else {
            false
        }
    }

    pub fn into_bin(self, id: usize) -> Bin {
        Bin {
            id,
            size: self.size,
            placements: self.placements,
        }
    }
}

fn is_better(
    candidate: (usize, u32, u32, u32, u32, bool),
    current_best: Option<(usize, u32, u32, u32, u32, bool)>,
) -> bool {
    match current_best {
        None => true,
        Some(best) => {
            let (_, cand_x, cand_y, _, _, _) = candidate;
            let (_, best_x, best_y, _, _, _) = best;

            cand_y < best_y || (cand_y == best_y && cand_x < best_x)
        }
    }
}

