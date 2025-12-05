// DO NOT CHANGE THESE IMPORTS
use tig_challenges::knapsack::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use crate::{seeded_hasher, HashMap, HashSet};

pub fn solve_challenge(
    challenge: &Challenge,
    save_solution: &dyn Fn(&Solution) -> Result<()>,
    _hyperparameters: &Option<Map<String, Value>>,
) -> Result<()> {
    // EVOLVE-BLOCK-START    
    // HYPERPARAMETERS: suggestions commented out here.
    
    #[derive(Serialize, Deserialize)]
    pub struct Hyperparameters {
        // Optionally define hyperparameters here
    }
   
    
    #[derive(Serialize, Deserialize)]
    pub struct Hyperparameters {
        pub max_iterations: usize,
        pub num_tries_per_iteration: usize,
    }
   
    let hyperparameters = match _hyperparameters {
        Some(params) => {
            serde_json::from_value::<Hyperparameters>(Value::Object(params.clone()))
                .map_err(|e| anyhow!("Failed to parse hyperparameters: {}", e))?
        }
        None => Hyperparameters {
            max_iterations: 100, // Number of outer loops (until no improvement for a full inner loop)
            num_tries_per_iteration: 500, // Number of random attempts per outer loop
        },
    };

    let num_items = challenge.values.len();
    let mut rng = SmallRng::from_seed(challenge.seed);
    let hasher = seeded_hasher(&challenge.seed);

    let mut selected_set: HashSet<usize> = HashSet::with_hasher(hasher.clone());
    let mut current_weight: u32 = 0;
    let mut current_value: i64 = 0; // Use i64 for value to handle negative interactions

    // --- Initial Solution: Greedy by value/weight ratio (prioritizing 0-weight items) ---
    let mut items_sorted_by_ratio: Vec<usize> = (0..num_items).collect();
    items_sorted_by_ratio.sort_by(|&a, &b| {
        let ratio_a = if challenge.weights[a] == 0 { f64::INFINITY } else { challenge.values[a] as f64 / challenge.weights[a] as f64 };
        let ratio_b = if challenge.weights[b] == 0 { f64::INFINITY } else { challenge.values[b] as f64 / challenge.weights[b] as f64 };
        ratio_b.partial_cmp(&ratio_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    for item_idx in items_sorted_by_ratio {
        if current_weight + challenge.weights[item_idx] <= challenge.max_weight {
            selected_set.insert(item_idx);
            current_weight += challenge.weights[item_idx];
        }
    }

    // Calculate initial total value including interactions for the initial selected_set
    for &i in &selected_set {
        current_value += challenge.values[i] as i64;
        for &j in &selected_set {
            if i < j { // Avoid double counting and self-interaction
                current_value += challenge.interaction_values[i][j] as i64;
            }
        }
    }

    // --- Local Search (Iterative Improvement) ---
    let all_item_indices: Vec<usize> = (0..num_items).collect();

    for _iter in 0..hyperparameters.max_iterations {
        let mut improved_in_this_iteration = false;

        // Create vectors for random selection from selected and unselected items
        // These are rebuilt each outer iteration, which is O(N).
        let selected_vec: Vec<usize> = selected_set.iter().cloned().collect();
        let unselected_vec: Vec<usize> = all_item_indices.iter()
            .filter(|&i| !selected_set.contains(i))
            .cloned()
            .collect();

        if selected_vec.is_empty() && unselected_vec.is_empty() {
            break; // No items to work with
        }

        for _try in 0..hyperparameters.num_tries_per_iteration {
            let action_type = rng.gen_range(0..3); // 0: Add, 1: Remove, 2: Swap

            match action_type {
                0 => { // Try to Add
                    if unselected_vec.is_empty() { continue; }
                    let item_out_idx = unselected_vec[rng.gen_range(0..unselected_vec.len())];

                    if current_weight + challenge.weights[item_out_idx] <= challenge.max_weight {
                        let mut gain: i64 = challenge.values[item_out_idx] as i64;
                        for &k in &selected_set {
                            gain += challenge.interaction_values[item_out_idx][k] as i64;
                        }

                        if gain > 0 {
                            selected_set.insert(item_out_idx);
                            current_weight += challenge.weights[item_out_idx];
                            current_value += gain;
                            improved_in_this_iteration = true;
                            break; // Found an improvement, restart search for this outer iteration
                        }
                    }
                },
                1 => { // Try to Remove
                    if selected_vec.is_empty() { continue; }
                    let item_in_idx = selected_vec[rng.gen_range(0..selected_vec.len())];

                    // Calculate loss if item_in_idx is removed
                    let mut loss: i64 = challenge.values[item_in_idx] as i64;
                    for &k in &selected_set {
                        if k != item_in_idx { // Interaction with other selected items
                            loss += challenge.interaction_values[item_in_idx][k] as i64;
                        }
                    }

                    if loss < 0 { // If removing it increases total value (i.e., its contribution was negative)
                        selected_set.remove(&item_in_idx);
                        current_weight -= challenge.weights[item_in_idx];
                        current_value -= loss; // Subtracting a negative loss means adding a positive value
                        improved_in_this_iteration = true;
                        break; // Found an improvement
                    }
                },
                _ => { // Try to Swap (action_type == 2)
                    if selected_vec.is_empty() || unselected_vec.is_empty() { continue; }
                    let item_in_idx = selected_vec[rng.gen_range(0..selected_vec.len())];
                    let item_out_idx = unselected_vec[rng.gen_range(0..unselected_vec.len())];

                    let delta_weight = challenge.weights[item_out_idx] as i32 - challenge.weights[item_in_idx] as i32;
                    // Check if new weight is valid (non-negative and within max_weight)
                    if (current_weight as i32 + delta_weight) >= 0 && (current_weight as i32 + delta_weight) as u32 <= challenge.max_weight {
                        let mut delta_value: i64 = challenge.values[item_out_idx] as i64 - challenge.values[item_in_idx] as i64;
                        
                        // Adjust for interactions with other items in the knapsack
                        for &k in &selected_set {
                            if k != item_in_idx { // k is an item that remains in the knapsack
                                delta_value += challenge.interaction_values[item_out_idx][k] as i64; // new interaction with item_out_idx
                                delta_value -= challenge.interaction_values[item_in_idx][k] as i64; // old interaction with item_in_idx removed
                            }
                        }

                        if delta_value > 0 {
                            selected_set.remove(&item_in_idx);
                            selected_set.insert(item_out_idx);
                            current_weight = (current_weight as i32 + delta_weight) as u32;
                            current_value += delta_value;
                            improved_in_this_iteration = true;
                            break; // Found an improvement
                        }
                    }
                }
            }
        }

        if !improved_in_this_iteration {
            break; // No improvement found in this entire outer iteration, local optimum reached
        }
    }

    let selected: Vec<usize> = selected_set.into_iter().collect();
    // EVOLVE-BLOCK-END
    
    save_solution(&Solution { items: selected })?;
    Ok(())
}