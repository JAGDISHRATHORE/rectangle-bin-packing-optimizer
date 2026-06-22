#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub bins_used: usize,
    pub wasted_area: u32,
}

use crate::greedy::PackingSolution;

pub fn evaluate(solution: &PackingSolution, box_size: u32) -> Score {
    let bins_used = solution.bins.len();

    let total_capacity = bins_used as u32 * box_size * box_size;

    let used_area: u32 = solution
        .bins
        .iter()
        .flat_map(|bin| bin.placements.iter())
        .map(|p| p.width * p.height)
        .sum();

    let wasted_area = total_capacity.saturating_sub(used_area);

    Score {
        bins_used,
        wasted_area,
    }
}