use std::time::Instant;

use crate::evaluator::evaluate;
use crate::generator::generate_instance;
use crate::greedy::{run_greedy, GreedyStrategy};
use crate::local_search::{run_local_search_swap, run_local_search_swap_tuned, LocalSearchStart};
use crate::local_search_geometry::run_geometry_local_search;
use crate::local_search_overlap::run_overlap_local_search;

type BenchmarkCase = (usize, usize, u32, u32, u32, u32, u32);
// (num_instances, rect_count, min_w, max_w, min_h, max_h, box_size)

fn run_benchmark_cases(title: &str, test_cases: &[BenchmarkCase]) {
    println!("\n========== {} ==========", title);

    for &(num_instances, rect_count, min_w, max_w, min_h, max_h, box_size) in test_cases {
        println!(
            "\n--- Test case: instances={}, rectangles={}, min_w={}, max_w={}, min_h={}, max_h={}, box_size={} ---",
            num_instances, rect_count, min_w, max_w, min_h, max_h, box_size
        );

        let mut greedy_area_bins = 0.0;
        let mut greedy_area_waste = 0.0;
        let mut greedy_area_time = 0.0;

        let mut greedy_max_bins = 0.0;
        let mut greedy_max_waste = 0.0;
        let mut greedy_max_time = 0.0;

        let mut perm_bins = 0.0;
        let mut perm_waste = 0.0;
        let mut perm_time = 0.0;

        let mut geom_bins = 0.0;
        let mut geom_waste = 0.0;
        let mut geom_time = 0.0;

        let mut overlap_bins = 0.0;
        let mut overlap_waste = 0.0;
        let mut overlap_time = 0.0;

        for _ in 0..num_instances {
            let instance = generate_instance(
                rect_count,
                min_w,
                max_w,
                min_h,
                max_h,
                box_size,
            );

            // Greedy Area
            let start = Instant::now();
            let sol = run_greedy(&instance, GreedyStrategy::AreaDescending);
            let duration = start.elapsed();
            let score = evaluate(&sol, box_size);

            greedy_area_bins += score.bins_used as f64;
            greedy_area_waste += score.wasted_area as f64;
            greedy_area_time += duration.as_micros() as f64;

            // Greedy Max Side
            let start = Instant::now();
            let sol = run_greedy(&instance, GreedyStrategy::MaxSideDescending);
            let duration = start.elapsed();
            let score = evaluate(&sol, box_size);

            greedy_max_bins += score.bins_used as f64;
            greedy_max_waste += score.wasted_area as f64;
            greedy_max_time += duration.as_micros() as f64;

            // Local Search Permutation
            let start = Instant::now();
            let result = run_local_search_swap(
                &instance,
                50,
                LocalSearchStart::GreedyAreaDescending,
            );
            let duration = start.elapsed();

            perm_bins += result.score.bins_used as f64;
            perm_waste += result.score.wasted_area as f64;
            perm_time += duration.as_micros() as f64;

            // Local Search Geometry
            let start_sol = run_greedy(&instance, GreedyStrategy::AreaDescending);

            let start = Instant::now();
            let result = run_geometry_local_search(&instance, start_sol, 100);
            let duration = start.elapsed();

            geom_bins += result.score.bins_used as f64;
            geom_waste += result.score.wasted_area as f64;
            geom_time += duration.as_micros() as f64;

            // Local Search Overlap
            let start = Instant::now();
            let result = run_overlap_local_search(&instance, 100);
            let duration = start.elapsed();

            overlap_bins += result.score.bins_used as f64;
            overlap_waste += result.score.wasted_area as f64;
            overlap_time += duration.as_micros() as f64;
        }

        let n = num_instances as f64;

        println!("\nGreedy AreaDescending:");
        println!("  avg bins used: {:.2}", greedy_area_bins / n);
        println!("  avg wasted area: {:.2}", greedy_area_waste / n);
        println!("  avg time (µs): {:.2}", greedy_area_time / n);

        println!("\nGreedy MaxSideDescending:");
        println!("  avg bins used: {:.2}", greedy_max_bins / n);
        println!("  avg wasted area: {:.2}", greedy_max_waste / n);
        println!("  avg time (µs): {:.2}", greedy_max_time / n);

        println!("\nLocal Search Permutation:");
        println!("  avg bins used: {:.2}", perm_bins / n);
        println!("  avg wasted area: {:.2}", perm_waste / n);
        println!("  avg time (µs): {:.2}", perm_time / n);

        println!("\nLocal Search Geometry:");
        println!("  avg bins used: {:.2}", geom_bins / n);
        println!("  avg wasted area: {:.2}", geom_waste / n);
        println!("  avg time (µs): {:.2}", geom_time / n);

        println!("\nLocal Search Overlap:");
        println!("  avg bins used: {:.2}", overlap_bins / n);
        println!("  avg wasted area: {:.2}", overlap_waste / n);
        println!("  avg time (µs): {:.2}", overlap_time / n);
    }
}

pub fn run_small_benchmark() {
    let test_cases: Vec<BenchmarkCase> = vec![
        (5, 10, 1, 8, 1, 8, 10),
        (5, 15, 1, 8, 1, 8, 10),
        (5, 20, 1, 8, 1, 8, 10),
    ];

    run_benchmark_cases("SMALL BENCHMARK", &test_cases);
}

pub fn run_large_benchmark() {
    let test_cases: Vec<BenchmarkCase> = vec![
        (10, 50, 1, 8, 1, 8, 10),
        (10, 80, 1, 8, 1, 8, 10),
        (10, 100, 1, 8, 1, 8, 10),
    ];

    run_benchmark_cases("LARGE BENCHMARK", &test_cases);
}

pub fn run_stress_benchmark_1000() {
    let instance = generate_instance(1000, 1, 8, 1, 8, 10);

    println!("\n========== STRESS BENCHMARK 1000 ==========");

    // Greedy Area
    let start = Instant::now();
    let sol_area = run_greedy(&instance, GreedyStrategy::AreaDescending);
    let dur_area = start.elapsed();
    let score_area = evaluate(&sol_area, instance.box_size);

    println!("\nGreedy AreaDescending:");
    println!("  bins used: {}", score_area.bins_used);
    println!("  wasted area: {}", score_area.wasted_area);
    println!("  time (ms): {:.2}", dur_area.as_secs_f64() * 1000.0);

    // Greedy Max Side
    let start = Instant::now();
    let sol_max = run_greedy(&instance, GreedyStrategy::MaxSideDescending);
    let dur_max = start.elapsed();
    let score_max = evaluate(&sol_max, instance.box_size);

    println!("\nGreedy MaxSideDescending:");
    println!("  bins used: {}", score_max.bins_used);
    println!("  wasted area: {}", score_max.wasted_area);
    println!("  time (ms): {:.2}", dur_max.as_secs_f64() * 1000.0);

    // Permutation LS, heavily tuned for the 1000-rectangle target
    let start = Instant::now();
    let perm_result = run_local_search_swap_tuned(
        &instance,
        3, // very few iterations
        LocalSearchStart::GreedyAreaDescending,
        5, // very small sampled neighborhood
    );
    let dur_perm = start.elapsed();

    println!("\nLocal Search Permutation (tuned):");
    println!("  bins used: {}", perm_result.score.bins_used);
    println!("  wasted area: {}", perm_result.score.wasted_area);
    println!("  time (ms): {:.2}", dur_perm.as_secs_f64() * 1000.0);

    // Geometry LS, tuned
    let start_solution = run_greedy(&instance, GreedyStrategy::AreaDescending);
    let start = Instant::now();
    let geom_result = run_geometry_local_search(&instance, start_solution, 10);
    let dur_geom = start.elapsed();

    println!("\nLocal Search Geometry (tuned):");
    println!("  bins used: {}", geom_result.score.bins_used);
    println!("  wasted area: {}", geom_result.score.wasted_area);
    println!("  time (ms): {:.2}", dur_geom.as_secs_f64() * 1000.0);

    // Overlap LS, tuned
    let start = Instant::now();
    let overlap_result = run_overlap_local_search(&instance, 20);
    let dur_overlap = start.elapsed();

    println!("\nLocal Search Overlap (tuned):");
    println!("  bins used: {}", overlap_result.score.bins_used);
    println!("  wasted area: {}", overlap_result.score.wasted_area);
    println!("  time (ms): {:.2}", dur_overlap.as_secs_f64() * 1000.0);
}