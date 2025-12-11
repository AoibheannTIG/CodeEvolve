use anyhow::{anyhow, Result};
use clap::{arg, Command};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_json::{Map, Value};
use std::{fs, path::PathBuf};
use tig_challenges::knapsack::*;

mod algorithm;
use algorithm::solve_challenge;

fn cli() -> Command {
    Command::new("knapsack-runner")
        .about("Executes a knapsack algorithm on a single challenge instance")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("solve")
                .arg(
                    arg!(<TRACK_ID> "Track ID for generating challenge instance")
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    arg!(<SEED> "Seed for generating challenge instance")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    arg!(<SOLUTION_FILE> "Path for algorithm to save solution as json file")
                        .value_parser(clap::value_parser!(PathBuf)),
                )
                .arg(
                    arg!(--hyperparameters [HYPERPARAMETERS] "Hyperparameters json string")
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand(
            Command::new("eval")
                .arg(
                    arg!(<TRACK_ID> "Track ID for generating challenge instance")
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    arg!(<SEED> "Seed for generating challenge instance")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    arg!(<SOLUTION_FILE> "Path to a solution json file")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
}

fn main() {
    let matches = cli().get_matches();

    if let Err(e) = (|| -> Result<()> {
        match matches.subcommand() {
            Some((cmd, sub_m)) => {
                let track_id = sub_m.get_one::<String>("TRACK_ID").unwrap();
                let track = serde_json::from_str(&format!("\"{}\"", track_id))
                    .map_err(|e| anyhow!("Failed to parse track_id: {:?}", e))?;
                let mut rng = StdRng::seed_from_u64(*sub_m.get_one::<u64>("SEED").unwrap());

                let challenge = Challenge::generate_instance(&rng.r#gen(), &track)
                    .map_err(|e| anyhow!("Failed to generate challenge: {:?}", e))?;
                match cmd {
                    "solve" => {
                        let hyperparameters: Option<Map<String, Value>> =
                            match sub_m.get_one::<String>("HYPERPARAMETERS") {
                                Some(hp_str) => {
                                    Some(serde_json::from_str(hp_str).map_err(|e| {
                                        anyhow!("Invalid JSON hyperparameters: {:?}", e)
                                    })?)
                                }
                                None => None,
                            };
                        let solution_file = sub_m.get_one::<PathBuf>("SOLUTION_FILE").unwrap();
                        let save_solution_fn = |solution: &Solution| -> Result<()> {
                            fs::write(&solution_file, serde_json::to_string(solution)?)?;
                            Ok(())
                        };
                        let start = std::time::Instant::now();
                        std::panic::catch_unwind(|| {
                            solve_challenge(&challenge, &save_solution_fn, &hyperparameters)
                        })
                        .map_err(|e| anyhow!("Panic during solve_challenge: {:?}", e))??;
                        println!("Time: {:?}", start.elapsed().as_secs_f64());
                        Ok(())
                    }
                    "eval" => {
                        let solution_file = sub_m.get_one::<PathBuf>("SOLUTION_FILE").unwrap();
                        let solution_str = fs::read_to_string(&solution_file).map_err(|e| {
                            anyhow!(
                                "Failed to read solution file {}: {:?}",
                                solution_file.display(),
                                e
                            )
                        })?;
                        let solution =
                            serde_json::from_str::<Solution>(&solution_str).map_err(|e| {
                                anyhow!(
                                    "Failed to parse solution file {}: {:?}",
                                    solution_file.display(),
                                    e
                                )
                            })?;
                        let quality = challenge
                            .evaluate_solution(&solution)
                            .map_err(|e| anyhow!("Failed to evaluate solution: {:?}", e))?;
                        println!("Quality: {}", quality);
                        Ok(())
                    }
                    _ => Err(anyhow!("Unknown command")),
                }
            }
            _ => Err(anyhow!("Invalid subcommand")),
        }
    })() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
