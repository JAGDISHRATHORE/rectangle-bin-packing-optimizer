use crate::framework::GreedyProblem;

pub fn run_generic_greedy<P>(problem: &P, strategy: &P::Strategy) -> P::Solution
where
    P: GreedyProblem,
{
    let mut items = problem.items();
    problem.sort_items(&mut items, strategy);
    problem.build_solution(&items)
}