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
        // Optionally define hyperparameters here
    }

    // This solution implements a greedy construction heuristic followed by a local search,
    // a standard and effective metaheuristic for the Quadratic Knapsack Problem (QKP).
    
    // --- Problem Data ---
    let num_items = challenge.values.len();
    let weights = &challenge.weights;
    let values = &challenge.values;
    let interactions = &challenge.interaction_values;
    let max_weight = challenge.max_weight;

    // --- Phase 1: Optimized Greedy Construction (O(N^2)) ---
    // This phase builds a strong initial solution by iteratively adding the item with the
    // best marginal gain per unit of weight.
    let mut selected_items = Vec::new();
    let mut is_selected = vec![false; num_items];
    let mut current_weight: u32 = 0;

    // `marginal_gains[k]` stores the gain of adding item `k` to the current solution.
    // gain(k) = values[k] + sum(interaction_values[k][j] for j in S)
    // Initially, S is empty, so gain(k) = values[k].
    let mut marginal_gains: Vec<i64> = values.iter().map(|&v| v as i64).collect();

    // The outer loop runs at most N times to select items.
    for _ in 0..num_items {
        let mut best_item_idx = None;
        let mut max_score = -f64::INFINITY;

        // Find the best candidate item to add in this iteration (O(N)).
        for k in 0..num_items {
            if !is_selected[k] && current_weight + weights[k] <= max_weight {
                let score = if weights[k] > 0 {
                    marginal_gains[k] as f64 / weights[k] as f64
                } else {
                    // Handle zero-weight items: infinitely good if gain is positive.
                    if marginal_gains[k] > 0 { f64::INFINITY } else { -f64::INFINITY }
                };

                if score > max_score {
                    max_score = score;
                    best_item_idx = Some(k);
                }
            }
        }

        if let Some(idx_to_add) = best_item_idx {
            // Add the best item to the solution.
            selected_items.push(idx_to_add);
            is_selected[idx_to_add] = true;
            current_weight += weights[idx_to_add];

            // Update marginal gains for all remaining candidates (O(N)).
            // The gain for item `k` increases by `interactions[k][idx_to_add]`
            // now that `idx_to_add` is in the solution.
            for k in 0..num_items {
                if !is_selected[k] {
                    marginal_gains[k] += interactions[k][idx_to_add] as i64;
                }
            }
        } else {
            // No more items can be added (either due to weight or negative scores).
            break;
        }
    }

    // --- Phase 2: Local Search with 1-1 Swaps (Best Improvement) ---
    // This phase refines the solution by repeatedly swapping an item inside the knapsack
    // with an item outside, if the swap improves the total value. It stops when no
    // such improvement can be found (a local optimum).
    loop {
        let mut best_delta: i64 = 0;
        let mut best_swap: Option<(usize, usize)> = None;

        // For efficiency, pre-calculate the sum of interactions for each item (in or out)
        // with the current solution `S`. P(k, S) = sum_{j in S} p_kj
        // This is O(N * |S|).
        let mut interaction_sums = vec![0i64; num_items];
        for k in 0..num_items {
            for &j in &selected_items {
                interaction_sums[k] += interactions[k][j] as i64;
            }
        }

        // Iterate through all possible 1-1 swaps (item_in <-> item_out). O(N^2)
        for &item_in in &selected_items {
            for item_out in 0..num_items {
                if !is_selected[item_out] {
                    // Check if the swap is valid in terms of weight, using u64 to prevent overflow.
                    let new_weight = current_weight as u64 - weights[item_in] as u64 + weights[item_out] as u64;
                    if new_weight <= max_weight as u64 {
                        // Calculate the change in value (delta) efficiently using pre-calculated sums. O(1)
                        // delta = (v_o - v_i) + sum_{j in S\{i}} (p_oj - p_ij)
                        // delta = (v_o - v_i) + (P(o,S) - p_oi) - (P(i,S) - p_ii)
                        let p_io = interactions[item_in][item_out] as i64;
                        let interaction_delta = (interaction_sums[item_out] - p_io) - interaction_sums[item_in];
                        let delta = (values[item_out] as i64 - values[item_in] as i64) + interaction_delta;

                        if delta > best_delta {
                            best_delta = delta;
                            best_swap = Some((item_in, item_out));
                        }
                    }
                }
            }
        }

        if let Some((i_to_remove, i_to_add)) = best_swap {
            // Perform the best swap found in this pass.
            let index = selected_items.iter().position(|&x| x == i_to_remove).unwrap();
            selected_items[index] = i_to_add;

            is_selected[i_to_remove] = false;
            is_selected[i_to_add] = true;

            current_weight = (current_weight as u64 - weights[i_to_remove] as u64 + weights[i_to_add] as u64) as u32;
        } else {
            // No further improvement found, local optimum is reached.
            break;
        }
    }

    let selected = selected_items;
    // EVOLVE-BLOCK-END
    
    save_solution(&Solution { items: selected })?;
    Ok(())
}

