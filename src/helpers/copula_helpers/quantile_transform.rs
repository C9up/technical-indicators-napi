/// Empirical CDF: transforms data to uniform [0,1] using rank/(n+1)
pub fn ranks_to_uniform(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    if n == 0 {
        return Vec::new();
    }

    // Create (value, original_index) pairs and sort by value
    let mut indexed: Vec<(f64, usize)> = data.iter().copied().enumerate().map(|(i, v)| (v, i)).collect();
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Assign ranks (handle ties by averaging)
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        // Find all elements with the same value
        while j < n && (indexed[j].0 - indexed[i].0).abs() < f64::EPSILON {
            j += 1;
        }
        // Average rank for ties
        let avg_rank = (i + j + 1) as f64 / 2.0; // 1-based average
        for item in indexed.iter().take(j).skip(i) {
            ranks[item.1] = avg_rank;
        }
        i = j;
    }

    // Convert ranks to uniform: rank / (n + 1)
    ranks.iter().map(|&r| r / (n as f64 + 1.0)).collect()
}

/// Inverse quantile transform: maps uniform values back to original distribution
/// Uses linear interpolation between sorted original values
pub fn uniform_to_original(uniform: &[f64], sorted_original: &[f64]) -> Vec<f64> {
    let n = sorted_original.len();
    if n == 0 {
        return vec![0.0; uniform.len()];
    }

    uniform
        .iter()
        .map(|&u| {
            let u_clamped = u.clamp(0.0, 1.0);
            let pos = u_clamped * (n as f64 - 1.0);
            let idx = pos.floor() as usize;
            let frac = pos - idx as f64;

            if idx >= n - 1 {
                sorted_original[n - 1]
            } else {
                sorted_original[idx] * (1.0 - frac) + sorted_original[idx + 1] * frac
            }
        })
        .collect()
}
