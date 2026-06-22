pub trait OptimizationProblem {
    type Solution: Clone;
    type Score: Ord + Clone;

    fn evaluate(&self, solution: &Self::Solution) -> Self::Score;
}

pub trait GreedyProblem {
    type Item: Clone;
    type Strategy;
    type Solution: Clone;

    fn items(&self) -> Vec<Self::Item>;
    fn sort_items(&self, items: &mut Vec<Self::Item>, strategy: &Self::Strategy);
    fn build_solution(&self, items: &[Self::Item]) -> Self::Solution;
}

pub trait Neighborhood<P: OptimizationProblem> {
    fn neighbors(&self, problem: &P, solution: &P::Solution) -> Vec<P::Solution>;
}