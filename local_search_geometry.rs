use rand::Rng;

use crate::bin::Bin;
use crate::evaluator::Score;
use crate::framework::{Neighborhood, OptimizationProblem};
use crate::generic_local_search::{run_generic_local_search, GenericLocalSearchResult};
use crate::greedy::PackingSolution;
use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct GeometryMove {
    pub rect_id: usize, 
    pub from_bin: usize,
    pub to_bin: usize,
    pub new_x: u32,
    pub new_y: u32,
    pub new_width: u32,
    pub new_height: u32,
    pub rotated: bool,
}

#[derive(Debug)]
pub struct GeometrySearchResult {
    pub solution: PackingSolution,
    pub score: Score,
    pub iterations: usize,
    pub history: Vec<PackingSolution>,
}

#[derive(Debug, Clone)]
pub struct GeometryProblem {
    pub instance: Instance,
}

impl OptimizationProblem for GeometryProblem {
    type Solution = PackingSolution;
    type Score = Score;

    fn evaluate(&self, solution: &Self::Solution) -> Self::Score {
        crate::evaluator::evaluate(solution, self.instance.box_size)
    }
}

pub struct GeometryNeighborhood {
    pub max_neighbors: usize,
}

impl Neighborhood<GeometryProblem> for GeometryNeighborhood {
    fn neighbors(
        &self,
        _problem: &GeometryProblem,
        solution: &PackingSolution,
    ) -> Vec<PackingSolution> {
        let mut rng = rand::thread_rng();
        let mut neighbors = Vec::new();

        if solution.bins.is_empty() {
            return neighbors;
        }

        for _ in 0..self.max_neighbors {
            let from_bin_index = rng.gen_range(0..solution.bins.len());
            let from_bin = &solution.bins[from_bin_index];

            if from_bin.placements.is_empty() {
                continue;
            }

            let placement_index = rng.gen_range(0..from_bin.placements.len());
            let placement = &from_bin.placements[placement_index];

            let to_bin_index = rng.gen_range(0..solution.bins.len());
            let to_bin = &solution.bins[to_bin_index];

            let positions = candidate_positions(to_bin);
            if positions.is_empty() {
                continue;
            }

            let (x, y) = positions[rng.gen_range(0..positions.len())];

            let mv = GeometryMove {
                rect_id: placement.rect_id,
                from_bin: from_bin_index,
                to_bin: to_bin_index,
                new_x: x,
                new_y: y,
                new_width: placement.width,
                new_height: placement.height,
                rotated: placement.rotated,
            };

            if let Some(candidate) = apply_move(solution, &mv) {
                neighbors.push(candidate);
            }

            if placement.width != placement.height {
                let mv_rot = GeometryMove {
                    rect_id: placement.rect_id,
                    from_bin: from_bin_index,
                    to_bin: to_bin_index,
                    new_x: x,
                    new_y: y,
                    new_width: placement.height,
                    new_height: placement.width,
                    rotated: !placement.rotated,
                };

                if let Some(candidate) = apply_move(solution, &mv_rot) {
                    neighbors.push(candidate);
                }
            }
        }

        neighbors
    }
}

fn renumber_bins(solution: &mut PackingSolution) {
    for (i, bin) in solution.bins.iter_mut().enumerate() {
        bin.id = i;
    }
}

fn candidate_positions(bin: &Bin) -> Vec<(u32, u32)> {
    let mut positions = vec![(0, 0)];

    for p in &bin.placements {
        positions.push((p.x + p.width, p.y));
        positions.push((p.x, p.y + p.height));
    }

    positions.sort_unstable();
    positions.dedup();
    positions
}

fn apply_move(solution: &PackingSolution, mv: &GeometryMove) -> Option<PackingSolution> {
    let mut new_solution = solution.clone();

    if mv.from_bin >= new_solution.bins.len() || mv.to_bin >= new_solution.bins.len() {
        return None;
    }

    let removed = {
        let from_bin = &mut new_solution.bins[mv.from_bin];
        from_bin.remove_placement_by_rect_id(mv.rect_id)?
    };

    let can_place = {
        let to_bin = &new_solution.bins[mv.to_bin];
        to_bin.can_place_dimensions_ignoring(
            mv.new_width,
            mv.new_height,
            mv.new_x,
            mv.new_y,
            mv.rect_id,
        )
    };

    if !can_place {
        return None;
    }

    {
        let to_bin = &mut new_solution.bins[mv.to_bin];
        to_bin.place_dimensions(
            removed.rect_id,
            mv.new_width,
            mv.new_height,
            mv.new_x,
            mv.new_y,
            mv.rotated,
        );
    }

    new_solution.bins.retain(|bin| !bin.is_empty());
    renumber_bins(&mut new_solution);

    Some(new_solution)
}

pub fn run_geometry_local_search(
    instance: &Instance,
    start_solution: PackingSolution,
    max_iterations: usize,
) -> GeometrySearchResult {
    let problem = GeometryProblem {
        instance: instance.clone(),
    };

    let neighborhood = GeometryNeighborhood { max_neighbors: 20 };

    let result: GenericLocalSearchResult<PackingSolution, Score> =
        run_generic_local_search(&problem, &neighborhood, start_solution, max_iterations);

    GeometrySearchResult {
        solution: result.solution,
        score: result.score,
        iterations: result.iterations,
        history: result.history,
    }
}