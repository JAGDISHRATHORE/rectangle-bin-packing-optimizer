mod algorithm;
mod benchmark;
mod bin;
mod evaluator;
mod framework;
mod generator;
mod generic_greedy;
mod generic_local_search;
mod greedy;
mod gui;
mod instance;
mod local_search;
mod local_search_geometry;
mod local_search_overlap;
mod packing_problem;
mod rectangle;
mod skyline;

use std::env;

use benchmark::{run_large_benchmark, run_small_benchmark, run_stress_benchmark_1000};
use gui::PackingApp;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "bench" => {
                println!("Running small benchmark...\n");
                run_small_benchmark();
                return Ok(());
            }
            "bench-large" => {
                println!("Running large benchmark...\n");
                run_large_benchmark();
                return Ok(());
            }
            "bench-1000" => {
                println!("Running 1000-rectangle stress benchmark...\n");
                run_stress_benchmark_1000();
                return Ok(());
            }
            _ => {}
        }
    }

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "OptimizeAlgoJagdish",
        options,
        Box::new(|_cc| Box::new(PackingApp::default())),
    )
}