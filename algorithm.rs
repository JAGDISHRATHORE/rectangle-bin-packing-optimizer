use crate::evaluator::Score;
use crate::greedy::PackingSolution;
use crate::instance::Instance;

pub trait PackingAlgorithm {
    fn name(&self) -> &str;

    fn run(&self, instance: &Instance) -> PackingSolution;

    fn evaluate(&self, solution: &PackingSolution, box_size: u32) -> Score {
        crate::evaluator::evaluate(solution, box_size)
    }
}