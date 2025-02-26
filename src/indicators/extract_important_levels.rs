use napi_derive::napi;

// Structure de résultat pour les niveaux importants
#[napi(object)]
pub struct ImportantLevels {
    pub highest_resistance: f64,
    pub lowest_support: f64,
    pub average_pivot: f64,
    pub supports: Vec<f64>,
    pub resistances: Vec<f64>,
}

// Fonction principale convertie
#[napi]
pub fn extract_important_levels(data: Vec<f64>) -> ImportantLevels {
    let mut supports = Vec::new();
    let mut resistances = Vec::new();
    let mut pivot_points = Vec::new();

    const WINDOW: usize = 5;

    for i in WINDOW..(data.len().saturating_sub(WINDOW)) {
        let current = data[i];
        let prev_window = i.saturating_sub(WINDOW);

        let is_highest = (prev_window..i).all(|j| data[j] <= current)
            && (i + 1..=i + WINDOW).all(|j| data.get(j).map_or(true, |v| *v <= current));

        let is_lowest = (prev_window..i).all(|j| data[j] >= current)
            && (i + 1..=i + WINDOW).all(|j| data.get(j).map_or(true, |v| *v >= current));

        if is_highest {
            resistances.push(current);
        }

        if is_lowest {
            supports.push(current);
        }

        if is_highest || is_lowest {
            pivot_points.push(current);
        }
    }

    // Calcul des valeurs finales avec gestion des cas vides
    let highest_resistance = resistances
        .iter()
        .copied()
        .fold(f64::NAN, f64::max)
        .max(data.iter().copied().fold(f64::NAN, f64::max));

    let lowest_support = supports
        .iter()
        .copied()
        .fold(f64::NAN, f64::min)
        .min(data.iter().copied().fold(f64::NAN, f64::min));

    let average_pivot = if !pivot_points.is_empty() {
        pivot_points.iter().sum::<f64>() / pivot_points.len() as f64
    } else {
        data.iter().sum::<f64>() / data.len() as f64
    };

    ImportantLevels {
        highest_resistance, // Directly assign the f64 value
        lowest_support,    // Directly assign the f64 value
        average_pivot,
        supports,
        resistances,
    }
}