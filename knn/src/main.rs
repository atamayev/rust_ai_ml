use ndarray::Array1;
use std::error::Error;

mod utils;
mod read_knn_csv;
mod error_metrics;
mod data_normalization;

fn cycle_through_housing_data(k: usize, mut total_housing_data: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let cloned_total_housing_data = total_housing_data.clone();

    for (index, record) in total_housing_data.iter_mut().enumerate() {
        // Create spliced data from the cloned data instead of the original
        let mut spliced_housing_data = cloned_total_housing_data.clone();
        spliced_housing_data.remove(index); // Remove the current record

        let mut knn_price_array = Vec::new();

        for record_checking_against in spliced_housing_data {
            if knn_price_array.len() < k {
                if let Some(&price) = record_checking_against.last() {
                    knn_price_array.push(price);
                } else {
                    println!("No price found for the record.");
                }
            } else {
                let distance_to_other_house = find_distance(record, &record_checking_against);
                if let Some(&least_value_in_existing_array) = knn_price_array.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
                    if distance_to_other_house < least_value_in_existing_array {
                        if let Some(position) = knn_price_array.iter().position(|&x| x == least_value_in_existing_array) {
                            knn_price_array.remove(position);
                            knn_price_array.push(distance_to_other_house);
                        }
                    }
                }
            }
        }

        // Calculate the average price and push it into the record
        let average_nearest_house_price = utils::average(&knn_price_array);
        record.push(average_nearest_house_price); // Use mutable reference to modify the record
    }

    total_housing_data
}

fn find_distance(focus_house_data: &[f32], comparison_house_data: &[f32]) -> f32 {
    focus_house_data
        .iter()
        .zip(comparison_house_data.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn extract_last_two_columns(new_parsed_data: &[Vec<f32>]) -> (Array1<f32>, Array1<f32>) {
    let last_column: Vec<f32> = new_parsed_data.iter()
        .map(|row| row[row.len() - 1])
        .collect();

    let second_to_last_column: Vec<f32> = new_parsed_data.iter()
        .map(|row| row[row.len() - 2])
        .collect();
    
    // Convert Vec<f32> to Array1<f32>
    let last_array = Array1::from_vec(last_column);
    let second_to_last_array = Array1::from_vec(second_to_last_column);

    (second_to_last_array, last_array)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data/housing_dataset.csv";

    let parsed_housing_data = read_knn_csv::read_knn_csv(file_path)?;

    let normalized_data = data_normalization::min_max_normalize(&parsed_housing_data);

    // let normalized_data = data_normalization::z_score_standardize(&parsed_housing_data);

    for k in 2..10 {
        let new_parsed_data = cycle_through_housing_data(k, normalized_data.clone());  // Clone on each iteration
        
        let (y_true, y_pred) = extract_last_two_columns(&new_parsed_data);
        let mse = error_metrics::root_mean_squared_error(&y_true, &y_pred);
        let r2 = error_metrics::r2_score(&y_true, &y_pred);
        println!("k {:?}", k);
        println!("Mean Squared Error: {:?}", mse);
        println!("R² Score: {:?}", r2);
        println!("-----------------------");
    }

    Ok(())
}
