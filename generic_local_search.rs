use crate::framework::{Neighborhood, OptimizationProblem};

#[derive(Debug, Clone)]
pub struct GenericLocalSearchResult<S, Sc> {
    pub solution: S,
    pub score: Sc,
    pub iterations: usize,
    pub history: Vec<S>,
}

pub fn run_generic_local_search<P, N>(
    problem: &P,
    neighborhood: &N,
    start_solution: P::Solution,
    max_iterations: usize,
) -> GenericLocalSearchResult<P::Solution, P::Score>
where
    P: OptimizationProblem,
    N: Neighborhood<P>,
{
    let mut current = start_solution.clone();
    let mut current_score = problem.evaluate(&current);
    let mut iterations = 0;
    let mut history = vec![start_solution];

    loop {
        if iterations >= max_iterations {
            break;
        }

        let mut improved = false;

        for candidate in neighborhood.neighbors(problem, &current) {
            let candidate_score = problem.evaluate(&candidate);

            if candidate_score < current_score {
                current = candidate;
                current_score = candidate_score.clone();
                history.push(current.clone());
                improved = true;
                break;
            }
        }

        iterations += 1;

        if !improved {
            break;
        }
    }

    GenericLocalSearchResult {
        solution: current,
        score: current_score,
        iterations,
        history,
    }
}