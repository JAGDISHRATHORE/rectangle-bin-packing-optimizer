use rand::Rng;
use rand::seq::SliceRandom;

use crate::bin::Placement;
use crate::evaluator::Score;
use crate::greedy::{run_greedy, GreedyStrategy, PackingSolution};
use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct OverlapResult {
    pub solution: PackingSolution,
    pub score: Score,
    pub iterations: usize,
    pub history: Vec<PackingSolution>,
}

fn pair_overlap_area(a: &Placement, b: &Placement) -> u32 {
    let x_left = a.x.max(b.x);
    let y_top = a.y.max(b.y);
    let x_right = (a.x + a.width).min(b.x + b.width);
    let y_bottom = (a.y + a.height).min(b.y + b.height);

    if x_right <= x_left || y_bottom <= y_top {
        0
    } else {
        (x_right - x_left) * (y_bottom - y_top)
    }
}

fn total_overlap_area(solution: &PackingSolution) -> u32 {
    let mut total = 0;

    for bin in &solution.bins {
        for i in 0..bin.placements.len() {
            for j in (i + 1)..bin.placements.len() {
                total += pair_overlap_area(&bin.placements[i], &bin.placements[j]);
            }
        }
    }

    total
}

fn overlaps_exist(solution: &PackingSolution) -> bool {
    total_overlap_area(solution) > 0
}

fn candidate_positions(solution: &PackingSolution, bin_index: usize, box_size: u32) -> Vec<(u32, u32)> {
    let mut positions = vec![(0, 0)];

    if let Some(bin) = solution.bins.get(bin_index) {
        for p in &bin.placements {
            positions.push((p.x + p.width, p.y));
            positions.push((p.x, p.y + p.height));
            positions.push((p.x, p.y));
        }
    }

    positions.push((0, 0));
    positions.push((box_size / 2, 0));
    positions.push((0, box_size / 2));

    positions.sort_unstable();
    positions.dedup();
    positions
}

fn remove_empty_bins(solution: &mut PackingSolution) {
    solution.bins.retain(|bin| !bin.placements.is_empty());
    for (i, bin) in solution.bins.iter_mut().enumerate() {
        bin.id = i;
    }
}

fn compute_overlap_penalty_for_bin(
    solution: &PackingSolution,
    bin_index: usize,
    rect_id: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> u32 {
    let mut penalty = 0;

    if let Some(bin) = solution.bins.get(bin_index) {
        let candidate = Placement {
            rect_id,
            x,
            y,
            width,
            height,
            rotated: false,
        };

        for p in &bin.placements {
            if p.rect_id == rect_id {
                continue;
            }
            penalty += pair_overlap_area(&candidate, p);
        }
    }

    penalty
}

fn apply_inter_bin_move_with_allowed_overlap(
    solution: &PackingSolution,
    from_bin: usize,
    to_bin: usize,
    rect_id: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    rotated: bool,
    allowed_ratio: f32,
    box_size: u32,
) -> Option<PackingSolution> {
    if width > box_size || height > box_size {
        return None;
    }
    if x + width > box_size || y + height > box_size {
        return None;
    }

    let mut new_solution = solution.clone();

    if from_bin >= new_solution.bins.len() || to_bin >= new_solution.bins.len() {
        return None;
    }

    let removed = {
        let from = &mut new_solution.bins[from_bin];
        from.remove_placement_by_rect_id(rect_id)?
    };

    let overlap_area =
        compute_overlap_penalty_for_bin(&new_solution, to_bin, rect_id, x, y, width, height);

    let rect_area = width * height;
    let max_allowed_overlap = (rect_area as f32 * allowed_ratio) as u32;

    if overlap_area > max_allowed_overlap {
        return None;
    }

    {
        let to = &mut new_solution.bins[to_bin];
        to.place_dimensions(removed.rect_id, width, height, x, y, rotated);
    }

    remove_empty_bins(&mut new_solution);
    Some(new_solution)
}

fn apply_swap_with_allowed_overlap(
    solution: &PackingSolution,
    bin_a: usize,
    rect_a_id: usize,
    bin_b: usize,
    rect_b_id: usize,
    allowed_ratio: f32,
    box_size: u32,
) -> Option<PackingSolution> {
    if bin_a == bin_b {
        return None;
    }
    if bin_a >= solution.bins.len() || bin_b >= solution.bins.len() {
        return None;
    }

    let a = solution.bins[bin_a]
        .placements
        .iter()
        .find(|p| p.rect_id == rect_a_id)?
        .clone();

    let b = solution.bins[bin_b]
        .placements
        .iter()
        .find(|p| p.rect_id == rect_b_id)?
        .clone();

    if a.width > box_size || a.height > box_size || b.width > box_size || b.height > box_size {
        return None;
    }
    if b.x + a.width > box_size || b.y + a.height > box_size {
        return None;
    }
    if a.x + b.width > box_size || a.y + b.height > box_size {
        return None;
    }

    let mut new_solution = solution.clone();

    {
        let from = &mut new_solution.bins[bin_a];
        from.remove_placement_by_rect_id(rect_a_id)?;
    }
    {
        let from = &mut new_solution.bins[bin_b];
        from.remove_placement_by_rect_id(rect_b_id)?;
    }

    let overlap_a =
        compute_overlap_penalty_for_bin(&new_solution, bin_a, rect_b_id, a.x, a.y, b.width, b.height);
    let overlap_b =
        compute_overlap_penalty_for_bin(&new_solution, bin_b, rect_a_id, b.x, b.y, a.width, a.height);

    let max_a = ((b.width * b.height) as f32 * allowed_ratio) as u32;
    let max_b = ((a.width * a.height) as f32 * allowed_ratio) as u32;

    if overlap_a > max_a || overlap_b > max_b {
        return None;
    }

    {
        let to = &mut new_solution.bins[bin_a];
        to.place_dimensions(rect_b_id, b.width, b.height, a.x, a.y, b.rotated);
    }
    {
        let to = &mut new_solution.bins[bin_b];
        to.place_dimensions(rect_a_id, a.width, a.height, b.x, b.y, a.rotated);
    }

    remove_empty_bins(&mut new_solution);
    Some(new_solution)
}

fn compactness_penalty(solution: &PackingSolution, box_size: u32) -> u32 {
    let mut penalty = 0u32;

    for bin in &solution.bins {
        for p in &bin.placements {
            penalty = penalty
                .saturating_add(p.x)
                .saturating_add(p.y);
        }
        if !bin.placements.is_empty() {
            penalty = penalty.saturating_add(box_size / 2);
        }
    }

    penalty
}

fn score_with_overlap_penalty(
    solution: &PackingSolution,
    box_size: u32,
    overlap_weight: u32,
) -> (usize, u32, u32, u32) {
    let score = crate::evaluator::evaluate(solution, box_size);
    let overlap = total_overlap_area(solution);
    let compactness = compactness_penalty(solution, box_size);
    let penalized_waste = score
        .wasted_area
        .saturating_add(overlap.saturating_mul(overlap_weight))
        .saturating_add(compactness);

    (score.bins_used, penalized_waste, overlap, compactness)
}

fn tuple_to_scalar(value: (usize, u32, u32, u32)) -> f64 {
    let (bins, penalized, overlap, compactness) = value;
    bins as f64 * 1_000_000.0
        + penalized as f64 * 10.0
        + overlap as f64
        + compactness as f64 * 0.01
}

fn repair_to_valid(instance: &Instance, solution: &PackingSolution) -> PackingSolution {
    if !overlaps_exist(solution) {
        return solution.clone();
    }

    let mut rects = Vec::new();
    for bin in &solution.bins {
        for p in &bin.placements {
            rects.push(crate::rectangle::Rectangle {
                id: p.rect_id,
                width: if p.rotated { p.height } else { p.width },
                height: if p.rotated { p.width } else { p.height },
            });
        }
    }

    rects.sort_by(|a, b| b.area().cmp(&a.area()));
    crate::greedy::pack_rectangles(instance, &rects)
}

pub fn run_overlap_local_search_with_start(
    instance: &Instance,
    start_solution: PackingSolution,
    iterations: usize,
) -> OverlapResult {
    let mut rng = rand::thread_rng();

    let mut current = start_solution;
    let mut performed_iterations = 0;

    let mut best_valid = if overlaps_exist(&current) {
        run_greedy(instance, GreedyStrategy::AreaDescending)
    } else {
        current.clone()
    };
    let mut best_valid_score = crate::evaluator::evaluate(&best_valid, instance.box_size);

 
    let mut history = vec![best_valid.clone()];

    for iter in 0..iterations {
        if current.bins.is_empty() {
            break;
        }

        let progress = iter as f32 / iterations.max(1) as f32;
        let allowed_ratio = 0.55 * (1.0 - progress);
        let overlap_weight = 4 + (progress * 30.0) as u32;
        let temperature = ((1.0 - progress).max(0.05) * 250.0) as f64;

        let tries_per_iteration = if instance.rectangles.len() <= 25 { 140 } else { 40 };

        let current_score_tuple =
            score_with_overlap_penalty(&current, instance.box_size, overlap_weight);

        let mut best_candidate: Option<(PackingSolution, (usize, u32, u32, u32))> = None;

        for _ in 0..tries_per_iteration {
            if current.bins.is_empty() {
                break;
            }

            let do_swap = current.bins.len() >= 2 && rng.gen_bool(0.25);

            let candidate = if do_swap {
                let bin_a = rng.gen_range(0..current.bins.len());
                let bin_b = rng.gen_range(0..current.bins.len());

                if bin_a == bin_b
                    || current.bins[bin_a].placements.is_empty()
                    || current.bins[bin_b].placements.is_empty()
                {
                    None
                } else {
                    let pa = &current.bins[bin_a].placements
                        [rng.gen_range(0..current.bins[bin_a].placements.len())];
                    let pb = &current.bins[bin_b].placements
                        [rng.gen_range(0..current.bins[bin_b].placements.len())];

                    apply_swap_with_allowed_overlap(
                        &current,
                        bin_a,
                        pa.rect_id,
                        bin_b,
                        pb.rect_id,
                        allowed_ratio,
                        instance.box_size,
                    )
                }
            } else {
                let from_bin = rng.gen_range(0..current.bins.len());
                if current.bins[from_bin].placements.is_empty() {
                    None
                } else {
                    let placement_index =
                        rng.gen_range(0..current.bins[from_bin].placements.len());
                    let placement = &current.bins[from_bin].placements[placement_index];

                    let rect_id = placement.rect_id;
                    let base_width = placement.width;
                    let base_height = placement.height;
                    let base_rotated = placement.rotated;

                    let to_bin = rng.gen_range(0..current.bins.len());
                    let mut positions =
                        candidate_positions(&current, to_bin, instance.box_size);

                    // Randomize a bit to make exploration broader.
                    positions.shuffle(&mut rng);

                    let mut found: Option<PackingSolution> = None;

                    for (x, y) in positions.into_iter().take(12) {
                        if let Some(candidate_solution) =
                            apply_inter_bin_move_with_allowed_overlap(
                                &current,
                                from_bin,
                                to_bin,
                                rect_id,
                                x,
                                y,
                                base_width,
                                base_height,
                                base_rotated,
                                allowed_ratio,
                                instance.box_size,
                            )
                        {
                            found = Some(candidate_solution);
                            break;
                        }

                        if base_width != base_height {
                            if let Some(candidate_solution) =
                                apply_inter_bin_move_with_allowed_overlap(
                                    &current,
                                    from_bin,
                                    to_bin,
                                    rect_id,
                                    x,
                                    y,
                                    base_height,
                                    base_width,
                                    !base_rotated,
                                    allowed_ratio,
                                    instance.box_size,
                                )
                            {
                                found = Some(candidate_solution);
                                break;
                            }
                        }
                    }

                    found
                }
            };

            if let Some(candidate_solution) = candidate {
                let candidate_score_tuple =
                    score_with_overlap_penalty(&candidate_solution, instance.box_size, overlap_weight);

                let replace = match &best_candidate {
                    None => true,
                    Some((_, best_tuple)) => candidate_score_tuple < *best_tuple,
                };

                if replace {
                    best_candidate = Some((candidate_solution, candidate_score_tuple));
                }
            }
        }

        if let Some((candidate_solution, candidate_score_tuple)) = best_candidate {
            let current_scalar = tuple_to_scalar(current_score_tuple);
            let candidate_scalar = tuple_to_scalar(candidate_score_tuple);

            let accept = if candidate_score_tuple < current_score_tuple {
                true
            } else {
                let delta = candidate_scalar - current_scalar;
                let probability = (-delta / temperature).exp().clamp(0.0, 1.0);
                rng.gen_bool(probability)
            };

            if accept {
                current = candidate_solution;
            }
        }

        performed_iterations += 1;

        if !overlaps_exist(&current) {
            let score = crate::evaluator::evaluate(&current, instance.box_size);
            if score < best_valid_score {
                best_valid = current.clone();
                best_valid_score = score;
                history.push(best_valid.clone());
            }
        }
    }

    let final_solution = repair_to_valid(instance, &best_valid);
    let final_score = crate::evaluator::evaluate(&final_solution, instance.box_size);


    let final_sig = crate::greedy::PackingSolution { bins: final_solution.bins.clone() };
    let last_sig = history.last().cloned();
    if let Some(last) = last_sig {
        let last_score = crate::evaluator::evaluate(&last, instance.box_size);
        if final_score < last_score {
            history.push(final_solution.clone());
        }
    } else {
        history.push(final_solution.clone());
    }

    OverlapResult {
        solution: final_solution,
        score: final_score,
        iterations: performed_iterations,
        history,
    }
}

pub fn run_overlap_local_search(
    instance: &Instance,
    iterations: usize,
) -> OverlapResult {
    let start = run_greedy(instance, GreedyStrategy::AreaDescending);
    run_overlap_local_search_with_start(instance, start, iterations)
}