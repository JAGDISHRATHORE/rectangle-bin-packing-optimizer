use crate::evaluator::{evaluate, Score};
use crate::framework::{GreedyProblem, OptimizationProblem};
use crate::greedy::{pack_rectangles, GreedyStrategy, PackingSolution};
use crate::instance::Instance;
use crate::rectangle::Rectangle;

#[derive(Debug, Clone)]
pub struct PackingProblem {
    pub instance: Instance,
}

impl OptimizationProblem for PackingProblem {
    type Solution = PackingSolution;
    type Score = Score;

    fn evaluate(&self, solution: &Self::Solution) -> Self::Score {
        evaluate(solution, self.instance.box_size)
    }
}

impl GreedyProblem for PackingProblem {
    type Item = Rectangle;
    type Strategy = GreedyStrategy;
    type Solution = PackingSolution;

    fn items(&self) -> Vec<Self::Item> {
        self.instance.rectangles.clone()
    }

    fn sort_items(&self, items: &mut Vec<Self::Item>, strategy: &Self::Strategy) {
        match strategy {
            GreedyStrategy::AreaDescending => {
                items.sort_by(|a, b| b.area().cmp(&a.area()));
            }
            GreedyStrategy::MaxSideDescending => {
                items.sort_by(|a, b| {
                    let a_max = a.width.max(a.height);
                    let b_max = b.width.max(b.height);
                    b_max.cmp(&a_max).then_with(|| b.area().cmp(&a.area()))
                });
            }
        }
    }

    fn build_solution(&self, items: &[Self::Item]) -> Self::Solution {
        pack_rectangles(&self.instance, items)
    }
}