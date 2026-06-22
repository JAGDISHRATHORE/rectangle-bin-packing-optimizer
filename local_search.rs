use rand::Rng;

use crate::evaluator::Score;
use crate::framework::{Neighborhood, OptimizationProblem};
use crate::generic_local_search::{run_generic_local_search, GenericLocalSearchResult};
use crate::greedy::{pack_rectangles, PackingSolution};
use crate::instance::Instance;
use crate::rectangle::Rectangle;

#[derive(Debug, Clone, Copy)]
pub enum LocalSearchStart {
    BadAscendingArea,
    GreedyAreaDescending,
}

#[derive(Debug, Clone)]
pub struct PermutationProblem {
    pub instance: Instance,
}

impl OptimizationProblem for PermutationProblem {
    type Solution = Vec<Rectangle>;
    type Score = Score;

    fn evaluate(&self, solution: &Self::Solution) -> Self::Score {
        let packing = pack_rectangles(&self.instance, solution);
        crate::evaluator::evaluate(&packing, self.instance.box_size)
    }
}

pub struct PermutationNeighborhood {
    pub max_neighbors: usize,
}

impl Neighborhood<PermutationProblem> for PermutationNeighborhood {
    fn neighbors(
        &self,
        _problem: &PermutationProblem,
        solution: &Vec<Rectangle>,
    ) -> Vec<Vec<Rectangle>> {
        let mut rng = rand::thread_rng();
        let mut neighbors = Vec::new();
        let n = solution.len();

        if n < 2 {
            return neighbors;
        }

        for _ in 0..self.max_neighbors {
            let mut candidate = solution.clone();

            if rng.gen_bool(0.5) {
                let i = rng.gen_range(0..n);
                let mut j = rng.gen_range(0..n);
                while j == i {
                    j = rng.gen_range(0..n);
                }
                candidate.swap(i, j);
            } else {
                let i = rng.gen_range(0..n);
                let mut j = rng.gen_range(0..n);
                while j == i {
                    j = rng.gen_range(0..n);
                }
                let rect = candidate.remove(i);
                candidate.insert(j, rect);
            }

            neighbors.push(candidate);
        }

        neighbors
    }
}

#[derive(Debug)]
pub struct LocalSearchResult {
    pub solution: PackingSolution,
    pub score: Score,
    pub iterations: usize,
    pub history: Vec<PackingSolution>,
}

fn build_bad_start_order(instance: &Instance) -> Vec<Rectangle> {
    let mut rectangles = instance.rectangles.clone();
    rectangles.sort_by(|a, b| a.area().cmp(&b.area()));
    rectangles
}

fn build_greedy_area_order(instance: &Instance) -> Vec<Rectangle> {
    let mut rectangles = instance.rectangles.clone();
    rectangles.sort_by(|a, b| b.area().cmp(&a.area()));
    rectangles
}

fn build_start_order(instance: &Instance, start: LocalSearchStart) -> Vec<Rectangle> {
    match start {
        LocalSearchStart::BadAscendingArea => build_bad_start_order(instance),
        LocalSearchStart::GreedyAreaDescending => build_greedy_area_order(instance),
    }
}

pub fn run_local_search_swap_tuned(
    instance: &Instance,
    max_iterations: usize,
    start: LocalSearchStart,
    max_neighbors: usize,
) -> LocalSearchResult {
    let problem = PermutationProblem {
        instance: instance.clone(),
    };

    let start_order = build_start_order(instance, start);
    let neighborhood = PermutationNeighborhood { max_neighbors };

    let result: GenericLocalSearchResult<Vec<Rectangle>, Score> =
        run_generic_local_search(&problem, &neighborhood, start_order, max_iterations);

    let final_solution = pack_rectangles(instance, &result.solution);

    let history = result
        .history
        .iter()
        .map(|order| pack_rectangles(instance, order))
        .collect();

    LocalSearchResult {
        solution: final_solution,
        score: result.score,
        iterations: result.iterations,
        history,
    }
}

pub fn run_local_search_swap(
    instance: &Instance,
    max_iterations: usize,
    start: LocalSearchStart,
) -> LocalSearchResult {
    run_local_search_swap_tuned(instance, max_iterations, start, 30)
}