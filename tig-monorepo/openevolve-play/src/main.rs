use anyhow::Result;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::sync::{Arc, Mutex};
use tig_algorithms::knapsack::openevolve_candidate::solve_challenge;
use tig_challenges::*;
use tig_utils::dejsonify;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 || args.len() > 5 {
        eprintln!(
            "Usage: {} <num_nonces> <difficulty> <seed> [hyperparameters]",
            args[0]
        );
        std::process::exit(1);
    }

    let num_nonces: u32 = args[1].parse().expect("Invalid number for nonces");
    let difficulty: knapsack::Difficulty = dejsonify(&args[2]).unwrap();
    let seed: u64 = args[3].parse().expect("Invalid seed");
    let hyperparameters = if args.len() == 5 {
        Some(dejsonify(&args[4]).unwrap())
    } else {
        None
    };

    let mut rng = SmallRng::seed_from_u64(seed);

    let mut total_relative_time = 0.0;
    let mut total_btb = 0.0;
    let mut successful_count = 0;

    for i in 0..num_nonces {
        let seed = rng.gen();
        let c = knapsack::Challenge::generate_instance(&seed, &difficulty).unwrap();
        let solution = Arc::new(Mutex::new(knapsack::Solution::new()));
        let solution_clone = solution.clone();
        let save_solution = move |s: &knapsack::Solution| -> Result<()> {
            *solution_clone.lock().unwrap() = s.clone();
            Ok(())
        };
        let start = std::time::Instant::now();
        let result = std::panic::catch_unwind(|| {
            solve_challenge(&c, &save_solution, &hyperparameters).unwrap_or_else(|e| {
                panic!("{:?}", e);
            });
        })
        .map_err(|e| anyhow::anyhow!("{:?}", e));
        match result {
            Ok(_) => {
                let solution = solution.lock().unwrap();
                match c.calculate_total_value(&*solution) {
                    Ok(total_value) => {
                        let btb = total_value as f64 / c.baseline_value as f64 - 1.0;
                        let time_taken = start.elapsed().as_micros() as f64;
                        let relative_time = c.baseline_time as f64 / time_taken;
                        
                        total_btb += btb;
                        total_relative_time += relative_time;
                        successful_count += 1;
                    }
                    Err(e) => {
                        eprintln!("nonce {}: invalid solution - {}", i, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("nonce {}: error - {}", i, e);
            }
        }
    }

    if successful_count > 0 {
        let avg_relative_time = total_relative_time / successful_count as f64;
        let avg_btb = total_btb / successful_count as f64;
        println!(
            "Average relative time (baseline/actual): {:.6}",
            avg_relative_time
        );
        println!("Average btb: {:.8}", avg_btb);
    } else {
        eprintln!("No successful solutions");
    }
}
