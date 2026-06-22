use crate::algorithm::PackingAlgorithm;
use crate::bin::Bin;
use crate::generic_greedy::run_generic_greedy;
use crate::instance::Instance;
use crate::packing_problem::PackingProblem;
use crate::rectangle::Rectangle;
use crate::skyline::SkylineBin;
#[derive(Debug, Clone, Copy)]
pub enum GreedyStrategy {
    AreaDescending,
    MaxSideDescending,
}
#[derive(Debug, Clone)]
pub struct PackingSolution {
    pub bins: Vec<Bin>,
}

pub fn pack_rectangles(instance: &Instance, rectangles: &[Rectangle]) -> PackingSolution {
    let mut skyline_bins: Vec<SkylineBin> = Vec::new();

    for rect in rectangles {
        let mut placed = false;
        for bin in &mut skyline_bins {
            if bin.place(rect) {
                placed = true;
                break;
            }
        }
        if !placed {
            let mut new_bin = SkylineBin::new(instance.box_size);

            if new_bin.place(rect) {
                skyline_bins.push(new_bin);
            } else {
                println!(
                    "Rectangle {} could not be placed even in a new bin!",
                    rect.id
                );
            }
        }
    }

    let bins = skyline_bins
        .into_iter()
        .enumerate()
        .map(|(id, b)| b.into_bin(id))
        .collect();

    PackingSolution { bins }
}

pub fn run_greedy(instance: &Instance, strategy: GreedyStrategy) -> PackingSolution {
    let problem = PackingProblem {
        instance: instance.clone(),
    };

    run_generic_greedy(&problem, &strategy)
}
pub struct GreedyAreaAlgorithm;
pub struct GreedyMaxSideAlgorithm;

impl PackingAlgorithm for GreedyAreaAlgorithm {
    fn name(&self) -> &str {
        "Greedy Area"
    }

    fn run(&self, instance: &Instance) -> PackingSolution {
        let problem = PackingProblem {
            instance: instance.clone(),
        };

        run_generic_greedy(&problem, &GreedyStrategy::AreaDescending)
    }
}
impl PackingAlgorithm for GreedyMaxSideAlgorithm {
    fn name(&self) -> &str {
        "Greedy Max Side"
    }

    fn run(&self, instance: &Instance) -> PackingSolution {
        let problem = PackingProblem {
            instance: instance.clone(),
        };

        run_generic_greedy(&problem, &GreedyStrategy::MaxSideDescending)
    }
}