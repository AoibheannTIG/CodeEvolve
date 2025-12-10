// DO NOT CHANGE THESE IMPORTS
use tig_challenges::knapsack::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use rand::{rngs::SmallRng, Rng, SeedableRng};

pub fn solve_challenge(
    challenge: &Challenge,
    save_solution: &dyn Fn(&Solution) -> Result<()>,
    _hyperparameters: &Option<Map<String, Value>>,
) -> Result<()> {
    // EVOLVE-BLOCK-START    
    #[derive(Serialize, Deserialize)]
    pub struct Hyperparameters {
        // Hyperparameters are not used in this strategy.
    }

    let num_items = challenge.values.len();

    // --- 1. Greedy Construction with Interaction-Aware Heuristic ---
    let mut selected_indices = Vec::new();
    let mut current_weight: u64 = 0;
    let mut is_selected = vec![false; num_items];

    loop {
        let mut best_item_idx: Option<usize> = None;
        let mut best_ratio = f64::NEG_INFINITY;

        for i in 0..num_items {
            if is_selected[i] {
                continue;
            }

            if current_weight + challenge.weights[i] as u64 <= challenge.max_weight as u64 {
                let mut marginal_gain = challenge.values[i] as i64;
                for &j in &selected_indices {
                    marginal_gain += challenge.interaction_values[i][j] as i64;
                }

                let weight = challenge.weights[i];
                let ratio = if weight == 0 {
                    if marginal_gain > 0 { f64::INFINITY } else { f64::NEG_INFINITY }
                } else {
                    marginal_gain as f64 / weight as f64
                };

                if ratio > best_ratio {
                    best_ratio = ratio;
                    best_item_idx = Some(i);
                }
            }
        }

        if let Some(item_to_add) = best_item_idx {
            selected_indices.push(item_to_add);
            is_selected[item_to_add] = true;
            current_weight += challenge.weights[item_to_add] as u64;
        } else {
            break; // No more items can be added
        }
    }

    // --- 2. Local Search (1,1-swap) with Efficient Delta Evaluation ---
    // Pre-calculate interaction sums for each item with the current solution for O(1) delta calculation.
    let mut gain_from_interactions = vec![0i64; num_items];
    for i in 0..num_items {
        for &j in &selected_indices {
            gain_from_interactions[i] += challenge.interaction_values[i][j] as i64;
        }
    }

    loop {
        let mut best_swap: Option<(usize, usize, i64)> = None; // (in_item, out_item, delta_score)

        for item_in in 0..num_items {
            if !is_selected[item_in] { continue; }
            for item_out in 0..num_items {
                if is_selected[item_out] { continue; }

                if current_weight - challenge.weights[item_in] as u64 + challenge.weights[item_out] as u64 > challenge.max_weight as u64 {
                    continue;
                }

                // O(1) delta calculation using pre-computed sums
                let delta = (challenge.values[item_out] as i64 - challenge.values[item_in] as i64)
                          + (gain_from_interactions[item_out] - challenge.interaction_values[item_out][item_in] as i64)
                          - (gain_from_interactions[item_in]);

                if delta > 0 {
                    if best_swap.is_none() || delta > best_swap.unwrap().2 {
                        best_swap = Some((item_in, item_out, delta));
                    }
                }
            }
        }

        if let Some((item_to_remove, item_to_add, _delta)) = best_swap {
            // Perform the best swap found
            current_weight = current_weight - challenge.weights[item_to_remove] as u64 + challenge.weights[item_to_add] as u64;

            let index_in_vec = selected_indices.iter().position(|&x| x == item_to_remove).unwrap();
            selected_indices[index_in_vec] = item_to_add;
            
            is_selected[item_to_remove] = false;
            is_selected[item_to_add] = true;

            // Update the cached interaction gains efficiently (O(n))
            for i in 0..num_items {
                gain_from_interactions[i] -= challenge.interaction_values[i][item_to_remove] as i64;
                gain_from_interactions[i] += challenge.interaction_values[i][item_to_add] as i64;
            }
        } else {
            break; // Reached a local optimum
        }
    }

    let selected = selected_indices;
    // EVOLVE-BLOCK-END
    
    save_solution(&Solution { items: selected })?;
    Ok(())
}

